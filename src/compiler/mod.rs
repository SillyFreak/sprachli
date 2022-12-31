mod constant;
mod error;
mod instruction;
mod writer;

use std::collections::{BTreeMap, HashMap};
use std::fmt;
use std::io::Write;
use std::iter;
use std::slice::SliceIndex;
use std::str::FromStr;

use itertools::Itertools;
use sprachli_fmt::{FormatterExt, ModuleFormat};

use crate::ast;
use crate::bytecode::instruction::{InlineConstant, Instruction, Offset};
use crate::parser::{parse_source_file, string_from_literal};
use constant::{Constant, Function, Number};
use instruction::{InstructionItem, PlaceholderKind};

pub use error::{Error, InternalError, Result};
pub use writer::write_bytecode;

pub fn compile_source_file<W: Write>(w: &mut W, source: &str) -> Result<()> {
    let ast = parse_source_file(source)?;
    compile_ast(w, ast)
}

pub fn compile_ast<W: Write>(w: &mut W, ast: ast::SourceFile) -> Result<()> {
    let module = Module::new(ast)?;
    write_bytecode(w, &module)?;
    Ok(())
}

#[derive(Clone)]
pub struct Module {
    constants: Vec<Constant>,
    globals: BTreeMap<usize, usize>,
    struct_types: BTreeMap<usize, StructType>,
}

impl Module {
    pub fn new(ast: ast::SourceFile) -> Result<Module> {
        Self::try_from(ast)
    }

    pub fn constants(&self) -> &[Constant] {
        &self.constants
    }

    pub fn globals(&self) -> &BTreeMap<usize, usize> {
        &self.globals
    }

    pub fn struct_types(&self) -> &BTreeMap<usize, StructType> {
        &self.struct_types
    }
}

impl TryFrom<ast::SourceFile<'_>> for Module {
    type Error = Error;

    fn try_from(ast: ast::SourceFile) -> Result<Module> {
        let mut c = Compiler::new();
        c.visit_source_file(ast)?;
        Ok(c.into())
    }
}

impl From<Compiler> for Module {
    fn from(compiler: Compiler) -> Self {
        let Compiler {
            constants,
            globals,
            struct_types,
            ..
        } = compiler;
        Self {
            constants,
            globals,
            struct_types,
        }
    }
}

impl ModuleFormat for Module {
    type Constant = Constant;

    fn constant(&self, index: usize) -> Option<(&Self::Constant, Option<&str>)> {
        let constant = self.constants.get(index)?;
        let string = match constant {
            Constant::String(value) => Some(value.as_str()),
            _ => None,
        };
        Some((constant, string))
    }
}

impl fmt::Debug for Module {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if f.alternate() {
            f.write_str("Module {\n")?;
            f.write_str("    constants: [\n")?;
            for (i, constant) in self.constants.iter().enumerate() {
                write!(f, "    {i:5}: ")?;
                constant.fmt_with(f, Some(self))?;
                f.write_str("\n")?;
            }
            f.write_str("    ],\n")?;
            f.write_str("    globals: {\n")?;
            for (name, index) in &self.globals {
                f.write_str("        ")?;
                let name = f.fmt_constant_ident(self, *name)?;
                match name {
                    Some(name) => {
                        write!(f, ": {index:<0$} -- ", 9usize.saturating_sub(name.len()))?
                    }
                    None => write!(f, ": {index} -- ")?,
                }
                f.fmt_constant(self, *index)?;
                f.write_str("\n")?;
            }
            f.write_str("    },\n")?;
            f.write_str("    struct_types: {\n")?;
            for (name, struct_type) in &self.struct_types {
                f.write_str("        ")?;
                f.fmt_constant_ident(self, *name)?;
                f.write_str(": ")?;
                struct_type.fmt_with(f, Some(self))?;
                f.write_str("\n")?;
            }
            f.write_str("    },\n")?;
            f.write_str("}")?;
            Ok(())
        } else {
            f.debug_struct("Module")
                .field("constants", &self.constants)
                .field("globals", &self.globals)
                .field("struct_types", &self.struct_types)
                .finish()
        }
    }
}

#[derive(Default, Debug, Clone)]
struct Compiler {
    constants: Vec<Constant>,
    constants_map: HashMap<Constant, usize>,
    struct_types: BTreeMap<usize, StructType>,
    globals: BTreeMap<usize, usize>,
}

impl Compiler {
    pub fn new() -> Self {
        Self::default()
    }

    fn add_constant<C: Into<Constant>>(&mut self, constant: C) -> usize {
        let mut add_constant = |constant: Constant| {
            if let Some(&index) = self.constants_map.get(&constant) {
                index
            } else {
                let index = self.constants.len();
                self.constants.push(constant.clone());
                self.constants_map.insert(constant, index);
                index
            }
        };

        add_constant(constant.into())
    }

    fn add_global<C: Into<Constant>>(&mut self, name: String, value: C) {
        let name = self.add_constant(name);
        let value = self.add_constant(value);
        self.globals.insert(name, value);
    }

    pub fn visit_source_file(&mut self, ast: ast::SourceFile) -> Result<()> {
        for declaration in ast.declarations {
            self.visit_declaration(declaration)?;
        }

        Ok(())
    }

    fn visit_declaration(&mut self, declaration: ast::Declaration) -> Result<()> {
        use ast::Declaration::*;

        match declaration {
            Use(_decl) => Err(Error::Unsupported("use declaration"))?,
            Fn(decl) => self.visit_fn(decl)?,
            Struct(decl) => self.visit_struct_type(decl)?,
            Mixin(_decl) => Err(Error::Unsupported("mixin"))?,
            Impl(_decl) => Err(Error::Unsupported("impl block"))?,
        }

        Ok(())
    }

    fn visit_fn(&mut self, decl: ast::FnDeclaration) -> Result<()> {
        let ast::FnDeclaration { name, trunk, .. } = decl;
        let function = InstructionCompiler::new(self).visit_fn_trunk(trunk)?;
        self.add_global(name.to_string(), function);
        Ok(())
    }

    fn visit_struct_type(&mut self, decl: ast::Struct) -> Result<()> {
        let ast::Struct { name, members, .. } = decl;
        let name = self.add_constant(name.to_string());
        let struct_type = match members {
            ast::StructMembers::Empty => StructType::Empty,
            ast::StructMembers::Positional(fields) => StructType::Positional(fields.len()),
            ast::StructMembers::Named(fields) => {
                let fields = fields
                    .iter()
                    .map(|field| self.add_constant(field.to_string()))
                    .collect();
                StructType::Named(fields)
            }
        };
        self.struct_types.insert(name, struct_type);
        Ok(())
    }
}

#[derive(Clone, PartialEq, Eq)]
pub enum StructType {
    Empty,
    Positional(usize),
    Named(Vec<usize>),
}

impl StructType {
    pub(crate) fn fmt_with<M: ModuleFormat>(
        &self,
        f: &mut fmt::Formatter<'_>,
        module: Option<&M>,
    ) -> fmt::Result {
        use StructType::*;

        match self {
            Empty => f.write_str("struct;"),
            Positional(count) => {
                f.write_str("struct(")?;
                for i in (0..*count).map(Some).intersperse(None) {
                    match i {
                        Some(i) => write!(f, "_{}", i)?,
                        None => f.write_str(", ")?,
                    }
                }
                f.write_str(");")?;
                Ok(())
            }
            Named(fields) => {
                if fields.is_empty() {
                    f.write_str("struct {}")?;
                } else {
                    f.write_str("struct { ")?;
                    for field in fields.iter().map(Some).intersperse(None) {
                        match field {
                            Some(field) => {
                                if let Some(module) = module {
                                    f.fmt_constant_ident(module, *field)?;
                                } else {
                                    write!(f, "#{field}")?;
                                }
                            }
                            None => f.write_str(", ")?,
                        }
                    }
                    f.write_str(" }")?;
                }
                Ok(())
            }
        }
    }
}

impl fmt::Debug for StructType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.fmt_with::<Module>(f, None)
    }
}

#[derive(Debug)]
struct InstructionCompiler<'a, 'input> {
    compiler: &'a mut Compiler,
    stack: Vec<Option<ast::Variable<'input>>>,
    jump_targets: Vec<JumpTarget>,
    instructions: Vec<InstructionItem>,
}

impl<'a, 'input> InstructionCompiler<'a, 'input> {
    pub fn new(compiler: &'a mut Compiler) -> Self {
        Self {
            compiler,
            stack: Default::default(),
            jump_targets: Default::default(),
            instructions: Default::default(),
        }
    }

    pub fn visit_fn_trunk(mut self, trunk: ast::FnTrunk<'input>) -> Result<Function> {
        let ast::FnTrunk {
            formal_parameters,
            body,
        } = trunk;

        self.stack
            .extend(formal_parameters.iter().copied().map(Some));
        self.visit_block(body)?;

        let instructions = self
            .instructions
            .iter()
            .map(|ins| ins.real().ok_or(InternalError::InvalidBytecode))
            .collect::<std::result::Result<_, _>>()?;

        Ok(Function::new(formal_parameters.len(), instructions))
    }

    // statements

    fn visit_statement(&mut self, stmt: ast::Statement<'input>) -> Result<()> {
        use ast::Statement::*;

        match stmt {
            Declaration(_) => {
                todo!("emit instructions");
            }
            Expression(expr) => {
                self.visit_expression(expr)?;
                self.push(Instruction::Pop)?;
                Ok(())
            }
            Jump(stmt) => self.visit_jump(stmt),
            VariableDeclaration(stmt) => self.visit_variable_declaration(stmt),
            Assignment(stmt) => self.visit_assignment(stmt),
        }
    }

    fn visit_jump(&mut self, stmt: ast::Jump<'input>) -> Result<()> {
        use ast::Jump::*;
        use PlaceholderKind::*;

        match stmt {
            Return(expr) => {
                let expr = expr.map(|expr| *expr);
                self.visit_optional(expr)?;
                self.push(Instruction::Return)?;
            }
            Break(expr) => {
                let jump_target = self.current_jump_target().ok_or(Error::NoLoopToExit)?;
                let depth = jump_target.depth();

                let expr = expr.map(|expr| *expr);
                self.visit_optional(expr)?;
                self.push(Instruction::PopScope(depth))?;

                let jump = self.push_placeholder(Jump)?;
                // if the compiler works correctly, this should be the same jump target as before
                self.current_jump_target_mut().unwrap().push_end_jump(jump);

                // despite pushing a value, break has a stack effect of zero, so negate that
                self.apply_stack_effect(-1)?;
            }
            Continue => {
                let jump_target = self.current_jump_target().ok_or(Error::NoLoopToExit)?;
                let depth = jump_target.depth();
                let start = jump_target.start();

                self.push(Instruction::InlineConstant(InlineConstant::Unit))?;
                self.push(Instruction::PopScope(depth))?;
                self.push(Instruction::Pop)?;

                self.push_placeholder(Jump)?.jump_back_to_index(self, start);
            }
        }

        Ok(())
    }

    fn visit_variable_declaration(&mut self, stmt: ast::VariableDeclaration<'input>) -> Result<()> {
        let ast::VariableDeclaration {
            variable,
            initializer,
        } = stmt;
        self.visit_optional(initializer)?;
        let var = self.stack.last_mut().unwrap();
        *var = Some(variable);
        Ok(())
    }

    fn visit_assignment(&mut self, stmt: ast::Assignment<'input>) -> Result<()> {
        use Instruction::*;

        let ast::Assignment { left, right } = stmt;

        let ast::Expression::Identifier(name) = left else {
            return Err(Error::InvalidAssignmentTarget);
        };

        self.visit_expression(right)?;

        if let Some((local, var)) = self.find_local(name) {
            if !var.mutable {
                Err(Error::ImmutableVariable)?;
            }
            self.push(StoreLocal(local))?;
        } else {
            let name = self.compiler.add_constant(name.to_string());
            self.push(StoreNamed(name))?;
        }
        Ok(())
    }

    // expressions

    fn visit_expression(&mut self, expr: ast::Expression<'input>) -> Result<()> {
        use ast::Expression::*;

        match expr {
            Number(literal) => self.visit_number(literal),
            Bool(value) => self.visit_bool(value),
            String(literal) => self.visit_string(literal),
            Identifier(name) => self.visit_identifier(name),
            Binary(expr) => self.visit_binary(expr),
            Unary(expr) => self.visit_unary(expr),
            Call(call) => self.visit_call(call),
            Block(block) => self.visit_block(block),
            Fn(expr) => self.visit_fn(expr),
            If(expr) => self.visit_if(expr),
            Loop(expr) => self.visit_loop(expr),
        }
    }

    fn visit_optional(&mut self, expr: Option<ast::Expression<'input>>) -> Result<()> {
        if let Some(expr) = expr {
            self.visit_expression(expr)?;
        } else {
            self.push(Instruction::InlineConstant(InlineConstant::Unit))?;
        }
        Ok(())
    }

    fn visit_number(&mut self, literal: &str) -> Result<()> {
        use Instruction::*;

        let number = Number::from_str(literal).map_err(InternalError::from)?;
        let constant = self.compiler.add_constant(number);
        self.push(Constant(constant))?;
        Ok(())
    }

    fn visit_bool(&mut self, value: bool) -> Result<()> {
        self.push(Instruction::InlineConstant(InlineConstant::Bool(value)))?;
        Ok(())
    }

    fn visit_string(&mut self, literal: &str) -> Result<()> {
        use Instruction::*;

        let string = string_from_literal(literal).map_err(InternalError::from)?;
        let constant = self.compiler.add_constant(string);
        self.push(Constant(constant))?;
        Ok(())
    }

    fn visit_identifier(&mut self, name: &str) -> Result<()> {
        use Instruction::*;

        if let Some((local, _)) = self.find_local(name) {
            self.push(LoadLocal(local))?;
        } else {
            let name = self.compiler.add_constant(name.to_string());
            self.push(LoadNamed(name))?;
        }
        Ok(())
    }

    fn visit_binary(&mut self, expr: ast::Binary<'input>) -> Result<()> {
        use Instruction::*;

        self.visit_expression(*expr.left)?;
        self.visit_expression(*expr.right)?;
        self.push(Binary(expr.operator))?;
        Ok(())
    }

    fn visit_unary(&mut self, expr: ast::Unary<'input>) -> Result<()> {
        use Instruction::*;

        self.visit_expression(*expr.right)?;
        self.push(Unary(expr.operator))?;
        Ok(())
    }

    fn visit_call(&mut self, call: ast::Call<'input>) -> Result<()> {
        use Instruction::*;

        self.visit_expression(*call.function)?;
        let arity = call.actual_parameters.len();
        for expr in call.actual_parameters {
            self.visit_expression(expr)?;
        }
        self.push(Call(arity))?;
        Ok(())
    }

    fn visit_block(&mut self, block: ast::Block<'input>) -> Result<()> {
        let depth = self.stack.len();
        let mut locals = 0;

        for stmt in block.statements {
            if matches!(stmt, ast::Statement::VariableDeclaration(_)) {
                locals += 1;
            }
            self.visit_statement(stmt)?;
        }
        if let Some(expr) = block.expression {
            self.visit_expression(*expr)?;
        } else {
            self.push(Instruction::InlineConstant(InlineConstant::Unit))?;
        }

        // there should be locals + 1 extra values on the stack,
        // and all but the top one should be named variables
        assert!(self.stack.len() == depth + locals + 1);
        assert!(self.stack[depth..depth + locals]
            .iter()
            .all(|local| !local.is_none()));
        assert!(self.stack[depth + locals].is_none());

        // drop the local variables
        self.push(Instruction::PopScope(depth))?;

        Ok(())
    }

    fn visit_fn(&mut self, expr: ast::Fn<'input>) -> Result<()> {
        use Instruction::*;

        let function = InstructionCompiler::new(self.compiler).visit_fn_trunk(expr.trunk)?;
        let constant = self.compiler.add_constant(function);
        self.push(Constant(constant))?;
        Ok(())
    }

    fn visit_if(&mut self, expr: ast::If<'input>) -> Result<()> {
        use ast::UnaryOperator::*;
        use Instruction::*;

        let mut end_jumps = Vec::new();

        for (condition, then_branch) in expr.then_branches {
            // jump if the condition is false
            self.visit_expression(condition)?;
            self.push(Unary(Not))?;
            let cond = self.push_placeholder(PlaceholderKind::JumpIf)?;

            let depth = self.stack.len();

            // do the then branch unless jumped
            self.visit_block(then_branch)?;
            end_jumps.push(self.push_placeholder(PlaceholderKind::Jump)?);
            cond.jump_fwd_to_current(self);

            // we have multiple branches of which only one is taken,
            // so the block's result is not really "still" on the stack.
            // after the whole if, the result (which may be unit if
            // there's no else branch) will be on the stack, so removing
            // the one from the block here is correct
            assert!(self.stack.len() == depth + 1);
            self.apply_stack_effect(-1)?;
        }
        let else_branch = expr.else_branch.map(ast::Expression::Block);
        self.visit_optional(else_branch)?;
        for end_jump in end_jumps {
            end_jump.jump_fwd_to_current(self);
        }
        Ok(())
    }

    fn visit_loop(&mut self, expr: ast::Loop<'input>) -> Result<()> {
        use Instruction::*;

        let start = self.push_jump_target().start();
        self.visit_block(expr.body)?;
        self.push(Pop)?;
        self.push_placeholder(PlaceholderKind::Jump)?
            .jump_back_to_index(self, start);
        self.pop_jump_target().unwrap();
        // we ignore the body's result, but the loop itself has a result (or diverges),
        // i.e. its stack effect is not 0 but 1.
        self.apply_stack_effect(1)?;
        Ok(())
    }

    // instruction helpers

    fn push<I: Into<InstructionItem>>(&mut self, instruction: I) -> Result<()> {
        use Instruction::*;
        use InstructionItem::*;

        let instruction = instruction.into();

        if let Some(effect) = instruction.stack_effect() {
            self.apply_stack_effect(effect)?;
        } else {
            match instruction {
                Real(PopScope(depth)) => {
                    let end = self
                        .stack
                        .len()
                        .checked_sub(1)
                        .ok_or(InternalError::InvalidStackEffect)?;
                    self.stack.drain(depth..end);
                }
                _ => unreachable!(),
            }
        }
        self.instructions.push(instruction);
        Ok(())
    }

    fn push_placeholder(&mut self, kind: PlaceholderKind) -> Result<Placeholder> {
        self.apply_stack_effect(kind.stack_effect())?;
        let index = self.instructions.len();
        self.instructions.push(InstructionItem::Placeholder(kind));
        Ok(Placeholder(index, kind))
    }

    // stack helpers

    fn apply_stack_effect(&mut self, effect: isize) -> Result<()> {
        if let Ok(effect) = usize::try_from(effect) {
            self.stack.extend(iter::repeat(None).take(effect));
        } else if let Ok(effect) = usize::try_from(-effect) {
            let len = self
                .stack
                .len()
                .checked_sub(effect)
                .ok_or(InternalError::InvalidStackEffect)?;
            self.stack.truncate(len);
        } else {
            unreachable!();
        }
        Ok(())
    }

    fn find_local(&mut self, name: &str) -> Option<(usize, ast::Variable<'input>)> {
        let mut iter = self.stack.iter().enumerate().rev();

        iter.find_map(|(i, local)| {
            let var = (*local)?;
            if var.name == name {
                Some((i, var))
            } else {
                None
            }
        })
    }

    // jump target helpers

    fn push_jump_target(&mut self) -> &JumpTarget {
        let depth = self.stack.len();
        let start = self.instructions.len();
        self.jump_targets.push(JumpTarget::new(depth, start));
        self.jump_targets.last().unwrap()
    }

    fn pop_jump_target(&mut self) -> Option<()> {
        let jump_target = self.jump_targets.pop()?;
        jump_target.fill_end_jumps(self);
        Some(())
    }

    fn current_jump_target(&self) -> Option<&JumpTarget> {
        self.jump_targets.last()
    }

    fn current_jump_target_mut(&mut self) -> Option<&mut JumpTarget> {
        self.jump_targets.last_mut()
    }
}

#[derive(Debug)]
struct Placeholder(usize, PlaceholderKind);

impl Placeholder {
    fn slot<'a>(&self, instructions: &'a mut InstructionCompiler) -> &'a mut InstructionItem {
        let Placeholder(index, kind) = *self;
        let slot = &mut instructions.instructions[index];
        assert!(*slot == InstructionItem::Placeholder(kind));
        slot
    }

    fn encoded_len<I>(instructions: &InstructionCompiler, range: I) -> usize
    where
        I: SliceIndex<[InstructionItem], Output = [InstructionItem]>,
    {
        instructions.instructions[range]
            .iter()
            .copied()
            .map(InstructionItem::encoded_len)
            .sum()
    }

    pub fn jump_fwd_to_current(self, instructions: &mut InstructionCompiler) {
        let Placeholder(index, kind) = self;

        // the offset is calculated from *after* the jump instruction (therefore +1)
        // and goes to before the target instruction. The target instruction is implicitly
        // the next one to be pushed onto the sequence, so simply calculate the length
        // until the end
        let offset = Offset::Forward(Self::encoded_len(instructions, index + 1..));

        *self.slot(instructions) = kind.jump(offset);
    }

    pub fn jump_back_to_index(self, instructions: &mut InstructionCompiler, to_index: usize) {
        let Placeholder(index, kind) = self;

        // the offset is calculated from *after* the jump instruction (therefore +1)
        // and goes back to before the target instruction, of which the index is given
        let offset = Offset::Backward(Self::encoded_len(instructions, to_index..index + 1));

        *self.slot(instructions) = kind.jump(offset);
    }
}

#[derive(Debug)]
struct JumpTarget {
    depth: usize,
    start: usize,
    end_jumps: Vec<Placeholder>,
}

impl JumpTarget {
    pub fn new(depth: usize, start: usize) -> Self {
        Self {
            depth,
            start,
            end_jumps: Default::default(),
        }
    }

    pub fn depth(&self) -> usize {
        self.depth
    }

    pub fn start(&self) -> usize {
        self.start
    }

    pub fn push_end_jump(&mut self, jump: Placeholder) {
        self.end_jumps.push(jump);
    }

    pub fn fill_end_jumps(self, instructions: &mut InstructionCompiler) {
        for jump in self.end_jumps {
            jump.jump_fwd_to_current(instructions);
        }
    }
}
