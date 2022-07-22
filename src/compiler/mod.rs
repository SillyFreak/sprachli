mod constant;
mod error;
mod instruction;
mod writer;

use std::collections::HashMap;
use std::io::Write;
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

#[derive(Debug, Clone)]
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
struct InstructionCompiler<'a> {
    compiler: &'a mut Compiler,
    locals: Vec<String>,
    instructions: Vec<Instruction>,
}

impl<'a> InstructionCompiler<'a> {
    pub fn new(compiler: &'a mut Compiler) -> Self {
        Self {
            compiler,
            locals: Default::default(),
            instructions: Default::default(),
        }
    }

    fn push(&mut self, instruction: Instruction) {
        self.instructions.push(instruction);
    }

    fn push_placeholder<F>(&mut self, f: F) -> Placeholder<F>
    where
        F: FnOnce(Offset) -> Instruction,
    {
        let index = self.instructions.len();
        self.instructions.push(Instruction::JumpPlaceholder);
        Placeholder(index, f)
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

    pub fn visit_fn(mut self, function: ast::Fn) -> Result<Function> {
        let ast::Fn {
            formal_parameters,
            body,
            ..
        } = function;

        self.locals = formal_parameters.iter().map(ToString::to_string).collect();
        self.visit_block(body)?;

        Ok(Function::new(formal_parameters.len(), self.instructions))
    }

    fn visit_expression(&mut self, expr: ast::Expression) -> Result<()> {
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
        }
    }

    fn visit_number(&mut self, literal: &str) -> Result<()> {
        use Instruction::*;

        let number = Number::from_str(literal).map_err(InternalError::from)?;
        let constant = self.compiler.add_constant(number);
        self.push(Constant(constant));
        Ok(())
    }

    fn visit_string(&mut self, literal: &str) -> Result<()> {
        use Instruction::*;

        let string = string_from_literal(literal).map_err(InternalError::from)?;
        let constant = self.compiler.add_constant(string);
        self.push(Constant(constant));
        Ok(())
    }

    fn visit_identifier(&mut self, name: &str) -> Result<()> {
        use Instruction::*;

        if let Some(local) = self
            .locals
            .iter()
            .enumerate()
            .rev()
            .find_map(|(i, local)| (*local == name).then_some(i))
        {
            self.push(LoadLocal(local));
        } else {
            let name = self.compiler.add_constant(name.to_string());
            self.push(LoadNamed(name));
        }
        Ok(())
    }

    fn visit_binary(&mut self, expr: ast::Binary) -> Result<()> {
        use Instruction::*;

        self.visit_expression(*expr.left)?;
        self.visit_expression(*expr.right)?;
        self.push(Binary(expr.operator));
        Ok(())
    }

    fn visit_unary(&mut self, expr: ast::Unary) -> Result<()> {
        use Instruction::*;

        self.visit_expression(*expr.right)?;
        self.push(Unary(expr.operator));
        Ok(())
    }

    fn visit_call(&mut self, call: ast::Call) -> Result<()> {
        use Instruction::*;

        self.visit_expression(*call.function)?;
        let arity = call.actual_parameters.len();
        for expr in call.actual_parameters {
            self.visit_expression(expr)?;
        }
        self.push(Call(arity));
        Ok(())
    }

    fn visit_block(&mut self, block: ast::Block) -> Result<()> {
        use instruction::InlineConstant;
        use Instruction::*;

        for stmt in block.statements {
            self.visit_statement(stmt)?;
        }
        if let Some(expr) = block.expression {
            self.visit_expression(*expr)?;
        } else {
            self.push(InlineConstant(InlineConstant::Unit));
        }
        Ok(())
    }

    fn visit_statement(&mut self, stmt: ast::Statement) -> Result<()> {
        use ast::Statement::*;
        use Instruction::*;

        match stmt {
            Declaration(_) => {
                todo!("emit instructions");
            }
            Expression(expr) => {
                self.visit_expression(expr)?;
                self.push(Pop);
                Ok(())
            }
        }
    }

    fn visit_if(&mut self, expr: ast::If) -> Result<()> {
        use ast::UnaryOperator::*;
        use instruction::InlineConstant;
        use Instruction::*;

        let mut end_jumps = Vec::new();

        for (condition, then_branch) in expr.then_branches {
            // jump if the condition is false
            self.visit_expression(condition)?;
            self.push(Unary(Not));
            let cond = self.push_placeholder(JumpIf);
            // do the then branch unless jumped
            self.visit_block(then_branch)?;
            end_jumps.push(self.push_placeholder(Jump));
            cond.fill(self);
        }
        if let Some(else_branch) = expr.else_branch {
            self.visit_block(else_branch)?;
        } else {
            self.push(InlineConstant(InlineConstant::Unit));
        }
        for end_jump in end_jumps {
            end_jump.fill(self);
        }
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
}