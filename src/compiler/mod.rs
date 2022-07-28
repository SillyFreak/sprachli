mod constant;
mod error;
mod instruction;
mod writer;

use std::collections::HashMap;
use std::fmt;
use std::io::Write;
use std::iter;
use std::str::FromStr;

use crate::ast;
use crate::parser::{parse_source_file, string_from_literal};
use constant::{Constant, Function, Number};
use instruction::{Instruction, Offset};

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
    globals: HashMap<usize, usize>,
}

impl Module {
    pub fn new(ast: ast::SourceFile) -> Result<Module> {
        Self::try_from(ast)
    }

    pub fn constants(&self) -> &[Constant] {
        &self.constants
    }

    pub fn globals(&self) -> &HashMap<usize, usize> {
        &self.globals
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
            constants, globals, ..
        } = compiler;
        Self { constants, globals }
    }
}

impl Module {
    pub(crate) fn fmt_constant(&self, f: &mut fmt::Formatter<'_>, index: usize) -> fmt::Result {
        match self.constants.get(index) {
            Some(constant) => write!(f, "{constant:?}"),
            _ => f.write_str("illegal constant"),
        }
    }

    pub(crate) fn fmt_constant_ident(
        &self,
        f: &mut fmt::Formatter<'_>,
        index: usize,
    ) -> std::result::Result<Option<&str>, fmt::Error> {
        match self.constants.get(index) {
            Some(Constant::String(value)) => {
                f.write_str(value)?;
                return Ok(Some(value));
            }
            Some(constant) => write!(f, "{constant:?} (invalid identifier)")?,
            _ => f.write_str("illegal constant")?,
        }
        Ok(None)
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
                let name = self.fmt_constant_ident(f, *name)?;
                match name {
                    Some(name) => {
                        write!(f, ": {index:<0$} -- ", 9usize.saturating_sub(name.len()))?
                    }
                    None => write!(f, ": {index} -- ")?,
                }
                self.fmt_constant(f, *index)?;
                f.write_str("\n")?;
            }
            f.write_str("    },\n")?;
            f.write_str("}")?;
            Ok(())
        } else {
            f.debug_struct("Module")
                .field("constants", &self.constants)
                .field("globals", &self.globals)
                .finish()
        }
    }
}

#[derive(Default, Debug, Clone)]
struct Compiler {
    constants: Vec<Constant>,
    constants_map: HashMap<Constant, usize>,
    globals: HashMap<usize, usize>,
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
            Fn(function) => self.visit_fn(function)?,
            Struct(_decl) => Err(Error::Unsupported("struct"))?,
            Mixin(_decl) => Err(Error::Unsupported("mixin"))?,
            Impl(_decl) => Err(Error::Unsupported("impl block"))?,
        }

        Ok(())
    }

    fn visit_fn(&mut self, function: ast::Fn) -> Result<()> {
        let name = function.name;
        let function = InstructionCompiler::new(self).visit_fn(function)?;
        self.add_global(name.to_string(), function);
        Ok(())
    }
}

#[derive(Debug)]
struct InstructionCompiler<'a, 'input> {
    compiler: &'a mut Compiler,
    stack: Vec<&'input str>,
    instructions: Vec<Instruction>,
}

impl<'a, 'input> InstructionCompiler<'a, 'input> {
    pub fn new(compiler: &'a mut Compiler) -> Self {
        Self {
            compiler,
            stack: Default::default(),
            instructions: Default::default(),
        }
    }

    fn apply_stack_effect(&mut self, effect: isize) -> Result<()> {
        if let Ok(effect) = usize::try_from(effect) {
            let empty_vars = iter::repeat("");
            self.stack.extend(empty_vars.take(effect));
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

    fn push(&mut self, instruction: Instruction) -> Result<()> {
        use Instruction::*;

        if let Some(effect) = instruction.stack_effect() {
            self.apply_stack_effect(effect)?;
        } else {
            match instruction {
                PopScope(depth) => {
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

    fn push_placeholder<F>(&mut self, dummy: Instruction, f: F) -> Result<Placeholder<F>>
    where
        F: FnOnce(Offset) -> Instruction,
    {
        self.apply_stack_effect(dummy.stack_effect().unwrap())?;
        let index = self.instructions.len();
        self.instructions.push(Instruction::JumpPlaceholder);
        Ok(Placeholder(index, f))
    }

    fn push_jump_placeholder(&mut self) -> Result<Placeholder<impl FnOnce(Offset) -> Instruction>> {
        use Instruction::*;
        use Offset::*;
        self.push_placeholder(Jump(Forward(0)), Jump)
    }

    fn push_jump_if_placeholder(
        &mut self,
    ) -> Result<Placeholder<impl FnOnce(Offset) -> Instruction>> {
        use Instruction::*;
        use Offset::*;
        self.push_placeholder(JumpIf(Forward(0)), JumpIf)
    }

    fn offset_from(&self, index: usize) -> Offset {
        let skipped = &self.instructions[index..];
        let offset = skipped.iter().copied().map(Instruction::encoded_len).sum();
        Offset::Forward(offset)
    }

    fn offset_to(&self, index: usize) -> Offset {
        let skipped = &self.instructions[index..];
        let offset = skipped.iter().copied().map(Instruction::encoded_len).sum();
        Offset::Backward(offset)
    }

    pub fn visit_fn(mut self, function: ast::Fn<'input>) -> Result<Function> {
        let ast::Fn {
            formal_parameters,
            body,
            ..
        } = function;

        self.stack.extend(&formal_parameters);
        self.visit_block(body)?;

        Ok(Function::new(formal_parameters.len(), self.instructions))
    }

    fn visit_expression(&mut self, expr: ast::Expression<'input>) -> Result<()> {
        use ast::Expression::*;

        match expr {
            Number(literal) => self.visit_number(literal),
            String(literal) => self.visit_string(literal),
            Identifier(name) => self.visit_identifier(name),
            Binary(expr) => self.visit_binary(expr),
            Unary(expr) => self.visit_unary(expr),
            Call(call) => self.visit_call(call),
            Block(block) => self.visit_block(block),
            If(expr) => self.visit_if(expr),
            Loop(expr) => self.visit_loop(expr),
        }
    }

    fn visit_optional(&mut self, expr: Option<ast::Expression<'input>>) -> Result<()> {
        use instruction::InlineConstant;
        use Instruction::*;

        if let Some(expr) = expr {
            self.visit_expression(expr)?;
        } else {
            self.push(InlineConstant(InlineConstant::Unit))?;
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

    fn visit_string(&mut self, literal: &str) -> Result<()> {
        use Instruction::*;

        let string = string_from_literal(literal).map_err(InternalError::from)?;
        let constant = self.compiler.add_constant(string);
        self.push(Constant(constant))?;
        Ok(())
    }

    fn visit_identifier(&mut self, name: &str) -> Result<()> {
        use Instruction::*;

        if let Some(local) = self
            .stack
            .iter()
            .enumerate()
            .rev()
            .find_map(|(i, local)| (*local == name).then_some(i))
        {
            self.push(LoadLocal(local))?;
        } else {
            let name = self.compiler.add_constant(name.to_string());
            self.push(LoadNamed(name))?;
        }
        Ok(())
    }

    fn visit_jump(&mut self, stmt: ast::Jump<'input>) -> Result<()> {
        use ast::Jump::*;

        match stmt {
            Return(expr) => {
                let expr = expr.map(|expr| *expr);
                self.visit_optional(expr)?;
                self.push(Instruction::Return)?;
            }
            Break(_expr) => todo!(),
            Continue => todo!(),
        }

        Ok(())
    }

    fn visit_variable_declaration(&mut self, stmt: ast::VariableDeclaration<'input>) -> Result<()> {
        self.visit_optional(stmt.initializer)?;
        let name = self.stack.last_mut().unwrap();
        *name = stmt.name;
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
        use instruction::InlineConstant;
        use Instruction::*;

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
            self.push(InlineConstant(InlineConstant::Unit))?;
        }

        // there should be locals + 1 extra values on the stack,
        // and all but the top one should be named variables
        assert!(self.stack.len() == depth + locals + 1);
        assert!(self.stack[depth..depth + locals]
            .iter()
            .all(|local| !local.is_empty()));
        assert!(self.stack[depth + locals].is_empty());

        // drop the local variables
        self.push(PopScope(depth))?;

        Ok(())
    }

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
        }
    }

    fn visit_if(&mut self, expr: ast::If<'input>) -> Result<()> {
        use ast::UnaryOperator::*;
        use Instruction::*;

        let mut end_jumps = Vec::new();

        for (condition, then_branch) in expr.then_branches {
            // jump if the condition is false
            self.visit_expression(condition)?;
            self.push(Unary(Not))?;
            let cond = self.push_jump_if_placeholder()?;

            let depth = self.stack.len();

            // do the then branch unless jumped
            self.visit_block(then_branch)?;
            end_jumps.push(self.push_jump_placeholder()?);
            cond.fill(self);

            // we have multiple branches of which only one is taken,
            // so the block's result is not really "still" on the stack.
            // after the whole if, the result (which may be unit if
            // there's no else branch) will be on the stack, so removing
            // the one from the block here is correct
            assert!(self.stack.len() == depth + 1);
            let name = self.stack.pop().unwrap();
            // this is an expression result, so it can't have a name
            assert!(name.is_empty());
        }
        let else_branch = expr.else_branch.map(ast::Expression::Block);
        self.visit_optional(else_branch)?;
        for end_jump in end_jumps {
            end_jump.fill(self);
        }
        Ok(())
    }

    fn visit_loop(&mut self, expr: ast::Loop<'input>) -> Result<()> {
        use Instruction::*;

        let start = self.instructions.len();
        self.visit_block(expr.body)?;
        self.push(Pop)?;
        self.push_jump_placeholder()?.fill_to(self, start);
        // we ignore the body's result, but the loop itself has a result (or diverges),
        // i.e. its stack effect is not 0 but 1.
        self.apply_stack_effect(1)?;
        Ok(())
    }
}

#[derive(Debug)]
struct Placeholder<F>(usize, F);

impl<F> Placeholder<F>
where
    F: FnOnce(Offset) -> Instruction,
{
    pub fn fill(self, instructions: &mut InstructionCompiler) {
        let Placeholder(index, f) = self;
        let instruction = f(instructions.offset_from(index + 1));
        assert_eq!(
            instructions.instructions[index],
            Instruction::JumpPlaceholder,
        );
        instructions.instructions[index] = instruction;
    }

    pub fn fill_to(self, instructions: &mut InstructionCompiler, to_index: usize) {
        let Placeholder(index, f) = self;
        let instruction = f(instructions.offset_to(to_index));
        assert_eq!(
            instructions.instructions[index],
            Instruction::JumpPlaceholder,
        );
        instructions.instructions[index] = instruction;
    }
}
