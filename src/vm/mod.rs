mod error;
mod stack;
mod value;

use bigdecimal::num_bigint::{BigInt, ToBigInt};
use bigdecimal::num_traits::ToPrimitive;
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

    fn get_local(&mut self, offset: usize, index: usize) -> Result<&Value<'b>> {
        let value = self
            .stack
            .get(offset + index)
            .ok_or(InternalError::InvalidLocal(index))?;

        Ok(value)
    }

    fn get_local_mut(&mut self, offset: usize, index: usize) -> Result<&mut Value<'b>> {
        let value = self
            .stack
            .get_mut(offset + index)
            .ok_or(InternalError::InvalidLocal(index))?;

        Ok(value)
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
        let value = self.get_local(offset, index)?.clone();
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

    fn store_local(&mut self, offset: usize, index: usize) -> Result<()> {
        let value = self.stack.pop()?;
        let var = self.get_local_mut(offset, index)?;
        *var = value;
        Ok(())
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

        fn to_integer(value: &BigDecimal) -> Result<BigInt> {
            if !value.is_integer() {
                Err(Error::TypeError("integral number value".to_string()))?;
            }
            Ok(value.to_bigint().unwrap())
        }

        fn to_isize(value: &BigDecimal) -> Result<isize> {
            if !value.is_integer() {
                Err(Error::TypeError("integral number value".to_string()))?;
            }
            value
                .to_isize()
                .ok_or_else(|| Error::TypeError("small integral number value".to_string()))
        }

        let [left, right] = {
            let mut ops = self.stack.pop_multiple(2)?;
            [ops.next().unwrap(), ops.next().unwrap()]
        };

        let arithmetic = |op: fn(&BigDecimal, &BigDecimal) -> BigDecimal| {
            let result = op(left.as_number()?, right.as_number()?);
            Ok(Value::number(result))
        };

        let bitshift = |op: fn(BigInt, isize) -> BigInt| {
            let left = left.as_number().and_then(to_integer)?;
            let right = right.as_number().and_then(to_isize)?;
            let result = op(left, right);
            Ok(Value::number(result.into()))
        };

        let bitwise = |op: fn(BigInt, BigInt) -> BigInt| {
            let left = left.as_number().and_then(to_integer)?;
            let right = right.as_number().and_then(to_integer)?;
            let result = op(left, right);
            Ok(Value::number(result.into()))
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

        let value = match operator {
            Multiply => arithmetic(|a, b| a * b),
            Divide => arithmetic(|a, b| a / b),
            Modulo => arithmetic(|a, b| a % b),
            Add => arithmetic(|a, b| a + b),
            Subtract => arithmetic(|a, b| a - b),
            RightShift => bitshift(|a, b| a >> b),
            LeftShift => bitshift(|a, b| a << b),
            BitAnd => bitwise(|a, b| a & b),
            BitXor => bitwise(|a, b| a ^ b),
            BitOr => bitwise(|a, b| a | b),
            Equals => equality_comparison(true),
            NotEquals => equality_comparison(false),
            Greater => number_comparison(|a, b| a > b),
            GreaterEquals => number_comparison(|a, b| a >= b),
            Less => number_comparison(|a, b| a < b),
            LessEquals => number_comparison(|a, b| a <= b),
        }?;

        self.stack.push(value)
    }

    fn jump(&mut self, iter: &mut InstructionIter, offset: Offset) -> Result<()> {
        use InternalError::*;

        iter.jump(offset).map_err(|_| InvalidJump)?;
        Ok(())
    }

    fn jump_if(&mut self, iter: &mut InstructionIter, offset: Offset) -> Result<()> {
        use InternalError::*;

        let condition = self.stack.pop()?.as_bool()?;
        if condition {
            iter.jump(offset).map_err(|_| InvalidJump)?;
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
                Unary(operator) => self.unary(operator)?,
                Binary(operator) => self.binary(operator)?,
                LoadLocal(index) => self.load_local(offset, index)?,
                StoreLocal(index) => self.store_local(offset, index)?,
                LoadNamed(index) => self.load_named(index)?,
                StoreNamed(_index) => Err(Error::Unsupported(
                    "Tried to mutate a binding in the global scope",
                ))?,
                LoadPositionalField(_index) => todo!(),
                StorePositionalField(_index) => todo!(),
                LoadNamedField(_index) => todo!(),
                StoreNamedField(_index) => todo!(),
                Pop => self.stack.pop().map(|_| ())?,
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
