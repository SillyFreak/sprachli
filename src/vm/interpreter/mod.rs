mod stack;

use bigdecimal::BigDecimal;

use crate::ast::{UnaryOperator, BinaryOperator};
use super::{Error, InternalError, Result, Value, Vm};
use super::environment::Environment;
use super::instruction::{self, InlineConstant};
use stack::Stack;

#[derive(Debug, Clone)]
pub struct Interpreter<'a> {
    stack: Stack,
    vm: &'a Vm,
}

impl<'a> Interpreter<'a> {
    pub fn new(vm: &'a Vm) -> Self {
        Self {
            stack: Stack::new(),
            vm,
        }
    }

    pub fn main(&mut self) -> Result<Value> {
        let env = &self.vm.global_scope();
        self.do_load(env, "main")?;
        self.call(env, 0)?;

        // the call opcode checks that only one value remains on the stack
        self.stack.pop()
    }

    fn constant(&mut self, index: usize) -> Result<()> {
        let value = self.vm.constants().get(index)?;

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

    fn load(&mut self, env: &Environment, index: usize) -> Result<()> {
        let name = self.vm.constants().get(index)?
            .as_string()
            .map_err(|_| InternalError::InvalidConstantType(index, "string"))?;

        self.do_load(env, name)
    }

    fn do_load(&mut self, env: &Environment, name: &str) -> Result<()> {
        let value = env
            .get(name)
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

    fn jump(&mut self, iter: &mut instruction::Iter, offset: isize) -> Result<()> {
        iter.jump(offset)
    }

    fn jump_if(&mut self, iter: &mut instruction::Iter, offset: isize) -> Result<()> {
        let condition = self.stack.pop()?.as_bool()?;
        if condition {
            iter.jump(offset)?;
        }
        Ok(())
    }

    fn call(
        &mut self,
        env: &Environment,
        arity: usize,
    ) -> Result<()> {
        use super::instruction::Instruction::*;

        let mut ops = self.stack.pop_multiple(arity + 1)?;

        let function = ops.next().unwrap();
        let function = function.as_function()?;
        function.check_arity(arity)?;

        let mut env = Environment::child(env);
        for (name, actual_parameter) in function.formal_parameters().iter().zip(ops) {
            env.set(name.to_string(), actual_parameter)?;
        }

        let offset = self.stack.len();

        let mut instructions = function.body().iter();
        while let Some(ins) = instructions.next() {
            match ins {
                Constant(index) => self.constant(index)?,
                InlineConstant(constant) => self.inline_constant(constant)?,
                Pop => self.stack.pop().map(|_| ())?,
                Unary(operator) => self.unary(operator)?,
                Binary(operator) => self.binary(operator)?,
                Load(index) => self.load(&env, index)?,
                Call(arity) => self.call(&env, arity)?,
                Jump(offset) => self.jump(&mut instructions, offset)?,
                JumpIf(offset) => self.jump_if(&mut instructions, offset)?,
                Invalid => Err(InternalError::InvalidInstruction)?,
            }
        }

        assert_eq!(self.stack.len(), offset + 1);

        Ok(())
    }
}
