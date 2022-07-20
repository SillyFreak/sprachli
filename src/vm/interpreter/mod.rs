mod stack;

use bigdecimal::BigDecimal;

use crate::ast::{UnaryOperator, BinaryOperator};
use super::{Error, InternalError, Result, Value, Vm};
use super::ast_module::AstModule;
use super::instruction::{self, InlineConstant, Offset};
use stack::Stack;

#[derive(Debug, Clone)]
pub struct Interpreter<'a> {
    stack: Stack,
    module: &'a AstModule,
}

impl<'a> Interpreter<'a> {
    pub fn new(module: &'a AstModule) -> Self {
        Self {
            stack: Stack::new(),
            module,
        }
    }

    pub fn main(&mut self) -> Result<Value> {
        self.do_load_named("main")?;
        self.call(0)?;

        // the call opcode checks that only one value remains on the stack
        self.stack.pop()
    }

    fn constant(&mut self, index: usize) -> Result<()> {
        let value = self.module.constants().get(index)?;

        self.stack.push(value.clone())
    }

    fn inline_constant(&mut self, constant: InlineConstant) -> Result<()> {
        use InlineConstant::*;

        let value = match constant {
            Unit => ().into(),
            Bool(bool) => bool.into(),
        };

        self.stack.push(value)
    }

    fn load_local(&mut self, locals: &Vec<Value>, index: usize) -> Result<()> {
        let value = locals.get(index)
            .ok_or_else(|| InternalError::InvalidLocal(index))?;

        self.stack.push(value.clone())
    }

    fn load_named(&mut self, index: usize) -> Result<()> {
        let name = self.module.constants().get(index)?
            .as_string()
            .map_err(|_| InternalError::InvalidConstantType(index, "string"))?;

        self.do_load_named(name)
    }

    fn do_load_named(&mut self, name: &str) -> Result<()> {
        let value = self.module.global_scope().get(name)
            .cloned()
            .ok_or_else(|| Error::NameError(name.to_string()))?;

        self.stack.push(value)
    }

    fn unary(&mut self, operator: UnaryOperator) -> Result<()> {
        use UnaryOperator::*;

        let right = self.stack.pop()?;

        let value = match operator {
            Negate => Value::from(-right.as_number()?),
            Not => Value::from(!right.as_bool()?),
        };

        self.stack.push(value)
    }

    fn binary(&mut self, operator: BinaryOperator) -> Result<()> {
        use BinaryOperator::*;
        use super::value::RawValue::*;
        use super::value::Value::*;

        let [left, right] = {
            let mut ops = self.stack.pop_multiple(2)?;
            [ops.next().unwrap(), ops.next().unwrap()]
        };

        let equality_comparison = |eq: bool| -> Result<Value> {
            let result = match (&left, &right) {
                (Unit, Unit) => true,
                (Bool(left), Bool(right)) => left == right,
                (Boxed(left), Boxed(right)) => {
                    match (left.as_ref(), right.as_ref()) {
                        (Number(left), Number(right)) => left == right,
                        (String(left), String(right)) => left == right,
                        // TODO functions?
                        _ => false,
                    }
                },
                _ => false,
            };

            Ok(Value::from(result == eq))
        };

        let number_comparison = |op: fn(&BigDecimal, &BigDecimal) -> bool| {
            let result = op(left.as_number()?, right.as_number()?);
            Ok(Value::from(result))
        };

        let arithmetic = |op: fn(&BigDecimal, &BigDecimal) -> BigDecimal| {
            let result = op(left.as_number()?, right.as_number()?);
            Ok(Value::from(result))
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

    fn jump(&mut self, iter: &mut instruction::Iter, offset: Offset) -> Result<()> {
        iter.jump(offset)
    }

    fn jump_if(&mut self, iter: &mut instruction::Iter, offset: Offset) -> Result<()> {
        let condition = self.stack.pop()?.as_bool()?;
        if condition {
            iter.jump(offset)?;
        }
        Ok(())
    }

    fn call(
        &mut self,
        arity: usize,
    ) -> Result<()> {
        use super::instruction::Instruction::*;

        let mut ops = self.stack.pop_multiple(arity + 1)?;

        let function = ops.next().unwrap();
        let function = function.as_function()?;
        function.check_arity(arity)?;

        let locals = ops.collect();

        let offset = self.stack.len();

        let mut instructions = function.body().iter();
        while let Some(ins) = instructions.next() {
            match ins {
                Constant(index) => self.constant(index)?,
                InlineConstant(constant) => self.inline_constant(constant)?,
                Pop => self.stack.pop().map(|_| ())?,
                Unary(operator) => self.unary(operator)?,
                Binary(operator) => self.binary(operator)?,
                LoadLocal(index) => self.load_local(&locals, index)?,
                LoadNamed(index) => self.load_named(index)?,
                Call(arity) => self.call(arity)?,
                Jump(offset) => self.jump(&mut instructions, offset)?,
                JumpIf(offset) => self.jump_if(&mut instructions, offset)?,
                Invalid => Err(InternalError::InvalidInstruction)?,
            }
        }

        assert_eq!(self.stack.len(), offset + 1);

        Ok(())
    }
}
