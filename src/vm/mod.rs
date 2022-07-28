mod error;
mod stack;
mod value;

use bigdecimal::BigDecimal;

use crate::ast::{BinaryOperator, UnaryOperator};
use crate::bytecode::instruction::{InlineConstant, Instruction, Offset};
use crate::bytecode::{Constant, InstructionIter, Module};
use stack::Stack;

pub use error::*;
pub use value::Value;

#[derive(Debug, Clone)]
pub struct Vm<'b> {
    module: Module<'b>,
    stack: Stack<'b>,
}

impl<'b> Vm<'b> {
    pub fn new(module: Module<'b>) -> Self {
        Self {
            module,
            stack: Stack::new(),
        }
    }

    pub fn run(mut self) -> Result<Value<'b>> {
        self.load_named_by_name("main")?;
        self.call(0)?;

        // the call opcode checks that only one value remains on the stack
        self.stack.pop()
    }

    fn get_constant(&self, index: usize) -> Result<&Constant<'b>> {
        let constant = self
            .module
            .constant(index)
            .ok_or_else(|| InternalError::InvalidConstant(index, self.module.constants().len()))?;
        Ok(constant)
    }

    fn get_global(&self, name: &str) -> Result<&Constant<'b>> {
        let value = self
            .module
            .global(name)
            .ok_or_else(|| Error::NameError(name.to_string()))?;
        Ok(value)
    }

    fn get_local(&mut self, offset: usize, index: usize) -> Result<Value<'b>> {
        let value = self
            .stack
            .get(offset + index)
            .ok_or(InternalError::InvalidLocal(index))?;

        Ok(value.clone())
    }

    fn constant(&mut self, index: usize) -> Result<()> {
        let value = self.get_constant(index).cloned()?;
        self.stack.push(Value::constant(value))
    }

    fn inline_constant(&mut self, constant: InlineConstant) -> Result<()> {
        use InlineConstant::*;

        let value = match constant {
            Unit => Value::unit(),
            Bool(bool) => Value::bool(bool),
        };

        self.stack.push(value)
    }

    fn load_local(&mut self, offset: usize, index: usize) -> Result<()> {
        let value = self.get_local(offset, index)?;
        self.stack.push(value)
    }

    fn load_named(&mut self, index: usize) -> Result<()> {
        let name = self.get_constant(index)?;
        let name = match name {
            Constant::String(name) => *name,
            _ => Err(InternalError::InvalidConstantType(index, "string"))?,
        };

        let value = self.get_global(name).cloned()?;
        self.stack.push(Value::constant(value))
    }

    fn load_named_by_name(&mut self, name: &str) -> Result<()> {
        let value = self.get_global(name).cloned()?;
        self.stack.push(Value::constant(value))
    }

    fn unary(&mut self, operator: UnaryOperator) -> Result<()> {
        use UnaryOperator::*;

        let right = self.stack.pop()?;

        let value = match operator {
            Negate => Value::number(-right.as_number()?.clone()),
            Not => Value::bool(!right.as_bool()?),
        };

        self.stack.push(value)
    }

    fn binary(&mut self, operator: BinaryOperator) -> Result<()> {
        use value::ValueRef::*;
        use BinaryOperator::*;
        use Value::*;

        let [left, right] = {
            let mut ops = self.stack.pop_multiple(2)?;
            [ops.next().unwrap(), ops.next().unwrap()]
        };

        let equality_comparison = |eq: bool| -> Result<Value> {
            let result = match (&left, &right) {
                (Unit, Unit) => true,
                (Bool(left), Bool(right)) => left == right,
                _ => match (left.get_ref().unwrap(), right.get_ref().unwrap()) {
                    (Number(left), Number(right)) => left == right,
                    (String(left), String(right)) => left == right,
                    // functions are always constants, so two values referring to the same function contain the same reference
                    (Function(left), Function(right)) => std::ptr::eq(left, right),
                    _ => false,
                },
            };

            Ok(Value::bool(result == eq))
        };

        let number_comparison = |op: fn(&BigDecimal, &BigDecimal) -> bool| {
            let result = op(left.as_number()?, right.as_number()?);
            Ok(Value::bool(result))
        };

        let arithmetic = |op: fn(&BigDecimal, &BigDecimal) -> BigDecimal| {
            let result = op(left.as_number()?, right.as_number()?);
            Ok(Value::number(result))
        };

        let value = match operator {
            Equals => equality_comparison(true),
            NotEquals => equality_comparison(false),
            Greater => number_comparison(|a, b| a > b),
            GreaterEquals => number_comparison(|a, b| a >= b),
            Less => number_comparison(|a, b| a < b),
            LessEquals => number_comparison(|a, b| a <= b),
            Add => arithmetic(|a, b| a + b),
            Subtract => arithmetic(|a, b| a - b),
            Multiply => arithmetic(|a, b| a * b),
            Divide => arithmetic(|a, b| a / b),
        }?;

        self.stack.push(value)
    }

    fn jump(&mut self, iter: &mut InstructionIter, offset: Offset) -> Result<()> {
        iter.jump(offset)
    }

    fn jump_if(&mut self, iter: &mut InstructionIter, offset: Offset) -> Result<()> {
        let condition = self.stack.pop()?.as_bool()?;
        if condition {
            iter.jump(offset)?;
        }
        Ok(())
    }

    fn call(&mut self, arity: usize) -> Result<()> {
        use Instruction::*;

        // the function & parameters are still on top of the stack
        // find the offset where this stack frame begins
        let offset = self.stack.len().checked_sub(arity + 1);
        let offset = self.stack.checked_index(offset)?;

        let function = self.stack.pop_deep(offset)?;
        let function = function.as_function()?;
        if arity != function.arity() {
            Err(Error::ValueError(format!(
                "wrong parameter number; expected {}, got {}",
                function.arity(),
                arity,
            )))?;
        }

        let mut instructions = function.body().iter();
        while let Some(ins) = instructions.next() {
            match ins.map_err(InternalError::from)? {
                Constant(index) => self.constant(index)?,
                InlineConstant(constant) => self.inline_constant(constant)?,
                Pop => self.stack.pop().map(|_| ())?,
                Unary(operator) => self.unary(operator)?,
                Binary(operator) => self.binary(operator)?,
                LoadLocal(index) => self.load_local(offset, index)?,
                LoadNamed(index) => self.load_named(index)?,
                PopScope(depth) => drop(self.stack.pop_all_under(offset + depth)?),
                Call(arity) => self.call(arity)?,
                Return => {
                    drop(self.stack.pop_all_under(offset + arity)?);
                    break;
                }
                Jump(offset) => self.jump(&mut instructions, offset)?,
                JumpIf(offset) => self.jump_if(&mut instructions, offset)?,
            }
        }

        // here the body block has finished, meaning all local variables
        // except for parameters are gone, and only the result is on top
        assert_eq!(self.stack.len(), offset + arity + 1);

        // pop the parameters from under the return value
        drop(self.stack.pop_all_under(offset)?);

        Ok(())
    }
}

impl InstructionIter<'_, '_> {
    pub fn jump(&mut self, offset: Offset) -> Result<()> {
        use InternalError::*;

        self.raw_jump(offset).map_err(|_| InvalidJump.into())
    }
}
