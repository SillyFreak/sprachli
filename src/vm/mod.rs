mod constant_table;
mod environment;
mod error;
mod instruction;
mod value;

use std::str::FromStr;

use constant_table::{ConstantTable, ConstantTableBuilder};
pub use error::*;
use instruction::InstructionSequence;
pub use value::Value;

use crate::{ast, grammar::string_from_literal};

#[derive(Debug, Clone)]
pub struct Vm {
    constants: ConstantTable,
}

impl<'input> TryFrom<&ast::SourceFile<'input>> for Vm {
    type Error = Error;

    fn try_from(value: &ast::SourceFile) -> Result<Self> {
        let mut builder = VmBuilder::new();
        builder.visit_source_file(value)?;
        Ok(builder.into_vm())
    }
}

impl Vm {
    pub fn new(constants: ConstantTable) -> Self {
        Self { constants }
    }

    pub fn run(&self) -> Result<Value> {
        todo!();
    }
}

#[derive(Default, Debug, Clone)]
struct VmBuilder {
    constants: ConstantTableBuilder,
}

impl VmBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn into_vm(self) -> Vm {
        Vm::new(self.constants.into_table())
    }

    pub fn visit_source_file(&mut self, ast: &ast::SourceFile) -> Result<()> {
        for declaration in &ast.declarations {
            self.visit_declaration(declaration)?;
        }

        Ok(())
    }

    fn visit_declaration(&mut self, declaration: &ast::Declaration) -> Result<()> {
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

    fn visit_fn(&mut self, function: &ast::Fn) -> Result<()> {
        let ast::Fn { name: _name, formal_parameters, body, .. } = function;

        let formal_parameters = formal_parameters.iter().map(ToString::to_string).collect();
        let mut instructions = InstructionSequence::new();
        self.visit_block(&mut instructions, body)?;

        let _function = value::Function::new(formal_parameters, instructions);
        todo!("bind function to a name");
        // env.set(name.to_string(), Value::Function(function));
    }

    fn visit_expression(&mut self, instructions: &mut InstructionSequence, expr: &ast::Expression) -> Result<()> {
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

    fn visit_number(&mut self, instructions: &mut InstructionSequence, literal: &str) -> Result<()> {
        use instruction::Instruction::*; 

        let number = value::Number::from_str(literal).expect("number liteal is not a valid number");
        let constant = self.constants.insert(number.into());
        instructions.push(Constant(constant));
        Ok(())
    }

    fn visit_string(&mut self, instructions: &mut InstructionSequence, literal: &str) -> Result<()> {
        use instruction::Instruction::*; 

        let string = string_from_literal(literal);
        let constant = self.constants.insert(string.into());
        instructions.push(Constant(constant));
        Ok(())
    }

    fn visit_identifier(&mut self, _instructions: &mut InstructionSequence, _name: &str) -> Result<()> {
        todo!("emit instructions")
    }

    fn visit_binary(&mut self, instructions: &mut InstructionSequence, expr: &ast::Binary) -> Result<()> {
        use instruction::Instruction::*; 

        self.visit_expression(instructions, &expr.left)?;
        self.visit_expression(instructions, &expr.right)?;
        instructions.push(Binary(expr.operator));
        Ok(())
    }

    fn visit_unary(&mut self, instructions: &mut InstructionSequence, expr: &ast::Unary) -> Result<()> {
        use instruction::Instruction::*; 

        self.visit_expression(instructions, &expr.right)?;
        instructions.push(Unary(expr.operator));
        Ok(())
    }

    fn visit_call(&mut self, instructions: &mut InstructionSequence, call: &ast::Call) -> Result<()> {
        self.visit_expression(instructions, &call.function)?;
        for expr in &call.actual_parameters {
            self.visit_expression(instructions, &expr)?;
        }
        todo!("emit instructions")
    }

    fn visit_block(&mut self, instructions: &mut InstructionSequence, block: &ast::Block) -> Result<()> {
        use instruction::{InlineConstant, Instruction::*}; 

        for _stmt in &block.statements {
            todo!("emit instructions");
        }
        if let Some(expr) = &block.expression {
            self.visit_expression(instructions, expr)?;
        } else {
            instructions.push(InlineConstant(InlineConstant::Unit));
        }
        Ok(())
    }

    fn visit_if(&mut self, instructions: &mut InstructionSequence, expr: &ast::If) -> Result<()> {
        for (condition, then_branch) in &expr.then_branches {
            self.visit_expression(instructions, condition)?;
            todo!("emit instructions");
            self.visit_block(instructions, then_branch)?;
            todo!("emit instructions");
        }
        if let Some(else_branch) = &expr.else_branch {
            self.visit_block(instructions, else_branch)?;
        }
        todo!("emit instructions")
    }
}