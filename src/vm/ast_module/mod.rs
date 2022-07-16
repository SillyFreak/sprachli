mod constants;

use std::collections::HashMap;
use std::str::FromStr;

pub use constants::ConstantTable;

use crate::ast;
use crate::grammar::string_from_literal;
use super::{Error, InternalError, Result, Value};
use super::instruction::{Instruction, InstructionSequence};
use super::value;
use constants::ConstantTableBuilder;

#[derive(Debug, Clone)]
pub struct AstModule {
    constants: ConstantTable,
    global_scope: HashMap<String, Value>,
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

    pub fn global_scope(&self) -> &HashMap<String, Value> {
        &self.global_scope
    }
}


#[derive(Default, Debug, Clone)]
struct AstModuleBuilder {
    constants: ConstantTableBuilder,
    global_scope: HashMap<String, Value>,
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

        let formal_parameters = formal_parameters.iter().map(ToString::to_string).collect();

        let mut instructions = InstructionSequence::new();
        self.visit_block(&mut instructions, body)?;

        let function = value::Function::new(formal_parameters, instructions);
        self.global_scope.insert(name.to_string(), function.into());
        Ok(())
    }

    fn visit_expression(
        &mut self,
        instructions: &mut InstructionSequence,
        expr: ast::Expression,
    ) -> Result<()> {
        use ast::Expression::*;

        match expr {
            Number(literal) => self.visit_number(instructions, literal),
            String(literal) => self.visit_string(instructions, literal),
            Identifier(name) => self.visit_identifier(instructions, name),
            Binary(expr) => self.visit_binary(instructions, expr),
            Unary(expr) => self.visit_unary(instructions, expr),
            Call(expr) => self.visit_call(instructions, expr),
            Block(expr) => self.visit_block(instructions, expr),
            If(expr) => self.visit_if(instructions, expr),
        }
    }

    fn visit_number(
        &mut self,
        instructions: &mut InstructionSequence,
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
        name: &str,
    ) -> Result<()> {
        use Instruction::*;

        let name = self.constants.insert(name.to_string().into());
        instructions.push(Load(name));
        Ok(())
    }

    fn visit_binary(
        &mut self,
        instructions: &mut InstructionSequence,
        expr: ast::Binary,
    ) -> Result<()> {
        use Instruction::*;

        self.visit_expression(instructions, *expr.left)?;
        self.visit_expression(instructions, *expr.right)?;
        instructions.push(Binary(expr.operator));
        Ok(())
    }

    fn visit_unary(
        &mut self,
        instructions: &mut InstructionSequence,
        expr: ast::Unary,
    ) -> Result<()> {
        use Instruction::*;

        self.visit_expression(instructions, *expr.right)?;
        instructions.push(Unary(expr.operator));
        Ok(())
    }

    fn visit_call(
        &mut self,
        instructions: &mut InstructionSequence,
        call: ast::Call,
    ) -> Result<()> {
        use Instruction::*;

        self.visit_expression(instructions, *call.function)?;
        let arity = call.actual_parameters.len();
        for expr in call.actual_parameters {
            self.visit_expression(instructions, expr)?;
        }
        instructions.push(Call(arity));
        Ok(())
    }

    fn visit_block(
        &mut self,
        instructions: &mut InstructionSequence,
        block: ast::Block,
    ) -> Result<()> {
        use Instruction::*;
        use super::instruction::InlineConstant;

        for stmt in block.statements {
            self.visit_statement(instructions, stmt)?;
        }
        if let Some(expr) = block.expression {
            self.visit_expression(instructions, *expr)?;
        } else {
            instructions.push(InlineConstant(InlineConstant::Unit));
        }
        Ok(())
    }

    fn visit_statement(
        &mut self,
        instructions: &mut InstructionSequence,
        stmt: ast::Statement,
    ) -> Result<()> {
        use Instruction::*;
        use ast::Statement::*;

        match stmt {
            Declaration(_) => {
                todo!("emit instructions");
            }
            Expression(expr) => {
                self.visit_expression(instructions, expr)?;
                instructions.push(Pop);
                Ok(())
            }
        }
    }

    fn visit_if(
        &mut self,
        instructions: &mut InstructionSequence,
        expr: ast::If,
    ) -> Result<()> {
        for (condition, then_branch) in expr.then_branches {
            self.visit_expression(instructions, condition)?;
            todo!("emit instructions");
            self.visit_block(instructions, then_branch)?;
            todo!("emit instructions");
        }
        if let Some(else_branch) = expr.else_branch {
            self.visit_block(instructions, else_branch)?;
        }
        todo!("emit instructions")
    }
}
