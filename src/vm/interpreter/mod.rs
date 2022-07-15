mod stack;

use bigdecimal::BigDecimal;

use crate::ast::{UnaryOperator, BinaryOperator};
use super::{Error, Result, Value, Vm};
use super::environment::Environment;
use super::instruction::InlineConstant;
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
        let env = self.vm.global_scope();
        self.do_load(env, "main")?;
        self.call(env, 0)?;

        // the call opcode checks that only one value remains on the stack
        self.stack.pop()
    }

    fn constant(&mut self, index: usize) -> Result<()> {
        let value = self
            .vm
            .constants
            .get(index)
            .cloned()?;

        self.stack.push(value)
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
        let name = self
            .vm
            .constants
            .get(index)?
            .as_string()?;

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

        let right = self.stack.pop()?;
        let left = self.stack.pop()?;

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

    fn call(
        &mut self,
        env: &Environment,
        arity: usize,
    ) -> Result<()> {
        use super::instruction::Instruction::*;

        let (function, actual_parameters) = self.stack.pop_call(arity)?;
        let function = function.as_function().expect("pop_call returned non-function function");

        let offset = self.stack.len();

        let mut env = Environment::with_parent(env);
        for (name, actual_parameter) in function.formal_parameters().iter().zip(actual_parameters.into_iter()) {
            env.set(name.to_string(), actual_parameter);
        }

        for ins in function.body() {
            match ins {
                Constant(index) => self.constant(index)?,
                InlineConstant(constant) => self.inline_constant(constant)?,
                Pop => self.stack.pop().map(|_| ())?,
                Unary(operator) => self.unary(operator)?,
                Binary(operator) => self.binary(operator)?,
                Load(index) => self.load(&env, index)?,
                Call(arity) => self.call(&env, arity)?,
            }
        }

        assert_eq!(self.stack.len(), offset + 1);

        Ok(())
    }
}
