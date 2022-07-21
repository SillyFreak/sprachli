mod constants;

use std::collections::HashMap;
use std::str::FromStr;

pub use constants::ConstantTable;

use super::instruction::{Instruction, InstructionSequence};
use super::value;
use super::{Error, InternalError, Result, Value};
use crate::ast;
use crate::grammar::string_from_literal;
use constants::ConstantTableBuilder;

#[derive(Debug, Clone)]
pub struct AstModule {
    pub constants: ConstantTable,
    pub global_scope: HashMap<usize, usize>,
}

impl TryFrom<ast::SourceFile<'_>> for AstModule {
    type Error = Error;

    fn try_from(ast: ast::SourceFile) -> Result<Self> {
        let mut builder = AstModuleBuilder::new();
        builder.visit_source_file(ast)?;
        Ok(builder.into_module())
    }
}

impl AstModule {
    pub fn new(ast: ast::SourceFile) -> Result<Self> {
        Self::try_from(ast)
    }

    pub fn constants(&self) -> &ConstantTable {
        &self.constants
    }

    pub fn global_scope(&self) -> &HashMap<usize, usize> {
        &self.global_scope
    }
}

#[derive(Default, Debug, Clone)]
struct AstModuleBuilder {
    constants: ConstantTableBuilder,
    global_scope: HashMap<usize, usize>,
}

impl AstModuleBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn into_module(self) -> AstModule {
        AstModule {
            constants: self.constants.into_table(),
            global_scope: self.global_scope,
        }
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
        let ast::Fn {
            name,
            formal_parameters,
            body,
            ..
        } = function;

        let mut instructions = InstructionSequence::new();
        let mut locals = formal_parameters.iter().map(ToString::to_string).collect();
        self.visit_block(&mut instructions, &mut locals, body)?;

        let function = value::Function::new(formal_parameters.len(), instructions);

        let name = self.constants.insert(name.to_string().into());
        let function = self.constants.insert(function.into());
        self.global_scope.insert(name, function);

        Ok(())
    }

    fn visit_expression(
        &mut self,
        instructions: &mut InstructionSequence,
        locals: &mut Vec<String>,
        expr: ast::Expression,
    ) -> Result<()> {
        use ast::Expression::*;

        match expr {
            Number(literal) => self.visit_number(instructions, locals, literal),
            String(literal) => self.visit_string(instructions, locals, literal),
            Identifier(name) => self.visit_identifier(instructions, locals, name),
            Binary(expr) => self.visit_binary(instructions, locals, expr),
            Unary(expr) => self.visit_unary(instructions, locals, expr),
            Call(call) => self.visit_call(instructions, locals, call),
            Block(block) => self.visit_block(instructions, locals, block),
            If(expr) => self.visit_if(instructions, locals, expr),
        }
    }

    fn visit_number(
        &mut self,
        instructions: &mut InstructionSequence,
        _locals: &mut Vec<String>,
        literal: &str,
    ) -> Result<()> {
        use Instruction::*;

        let number = value::Number::from_str(literal).map_err(InternalError::from)?;
        let constant = self.constants.insert(number.into());
        instructions.push(Constant(constant));
        Ok(())
    }

    fn visit_string(
        &mut self,
        instructions: &mut InstructionSequence,
        _locals: &mut Vec<String>,
        literal: &str,
    ) -> Result<()> {
        use Instruction::*;

        let string = string_from_literal(literal).map_err(InternalError::from)?;
        let constant = self.constants.insert(string.into());
        instructions.push(Constant(constant));
        Ok(())
    }

    fn visit_identifier(
        &mut self,
        instructions: &mut InstructionSequence,
        locals: &mut Vec<String>,
        name: &str,
    ) -> Result<()> {
        use Instruction::*;

        if let Some(local) = locals
            .iter()
            .enumerate()
            .rev()
            .find_map(|(i, local)| (*local == name).then_some(i))
        {
            instructions.push(LoadLocal(local));
        } else {
            let name = self.constants.insert(name.to_string().into());
            instructions.push(LoadNamed(name));
        }
        Ok(())
    }

    fn visit_binary(
        &mut self,
        instructions: &mut InstructionSequence,
        locals: &mut Vec<String>,
        expr: ast::Binary,
    ) -> Result<()> {
        use Instruction::*;

        self.visit_expression(instructions, locals, *expr.left)?;
        self.visit_expression(instructions, locals, *expr.right)?;
        instructions.push(Binary(expr.operator));
        Ok(())
    }

    fn visit_unary(
        &mut self,
        instructions: &mut InstructionSequence,
        locals: &mut Vec<String>,
        expr: ast::Unary,
    ) -> Result<()> {
        use Instruction::*;

        self.visit_expression(instructions, locals, *expr.right)?;
        instructions.push(Unary(expr.operator));
        Ok(())
    }

    fn visit_call(
        &mut self,
        instructions: &mut InstructionSequence,
        locals: &mut Vec<String>,
        call: ast::Call,
    ) -> Result<()> {
        use Instruction::*;

        self.visit_expression(instructions, locals, *call.function)?;
        let arity = call.actual_parameters.len();
        for expr in call.actual_parameters {
            self.visit_expression(instructions, locals, expr)?;
        }
        instructions.push(Call(arity));
        Ok(())
    }

    fn visit_block(
        &mut self,
        instructions: &mut InstructionSequence,
        locals: &mut Vec<String>,
        block: ast::Block,
    ) -> Result<()> {
        use super::instruction::InlineConstant;
        use Instruction::*;

        for stmt in block.statements {
            self.visit_statement(instructions, locals, stmt)?;
        }
        if let Some(expr) = block.expression {
            self.visit_expression(instructions, locals, *expr)?;
        } else {
            instructions.push(InlineConstant(InlineConstant::Unit));
        }
        Ok(())
    }

    fn visit_statement(
        &mut self,
        instructions: &mut InstructionSequence,
        locals: &mut Vec<String>,
        stmt: ast::Statement,
    ) -> Result<()> {
        use ast::Statement::*;
        use Instruction::*;

        match stmt {
            Declaration(_) => {
                todo!("emit instructions");
            }
            Expression(expr) => {
                self.visit_expression(instructions, locals, expr)?;
                instructions.push(Pop);
                Ok(())
            }
        }
    }

    fn visit_if(
        &mut self,
        instructions: &mut InstructionSequence,
        locals: &mut Vec<String>,
        expr: ast::If,
    ) -> Result<()> {
        use super::instruction::InlineConstant;
        use ast::UnaryOperator::*;
        use Instruction::*;

        let mut end_jumps = Vec::new();

        for (condition, then_branch) in expr.then_branches {
            // jump if the condition is false
            self.visit_expression(instructions, locals, condition)?;
            instructions.push(Unary(Not));
            let cond = instructions.push_placeholder(JumpIf);
            // do the then branch unless jumped
            self.visit_block(instructions, locals, then_branch)?;
            end_jumps.push(instructions.push_placeholder(Jump));
            cond.fill(instructions);
        }
        if let Some(else_branch) = expr.else_branch {
            self.visit_block(instructions, locals, else_branch)?;
        } else {
            instructions.push(InlineConstant(InlineConstant::Unit));
        }
        for end_jump in end_jumps {
            end_jump.fill(instructions);
        }
        Ok(())
    }
}
