use bigdecimal::BigDecimal;

use super::{environment::Environment, value::Function, Error, Result, Value, Vm};

#[derive(Debug, Clone, Copy)]
pub struct Interpreter<'a> {
    vm: &'a Vm,
}

impl<'a> Interpreter<'a> {
    pub fn new(vm: &'a Vm) -> Self {
        Self { vm }
    }

    pub fn main(&self) -> Result<Value> {
        let env = self.vm.global_scope();
        let main = self.load(env, "main")?;
        let main = main.as_function()?;
        self.call(env, main, &[])
    }

    fn constant(&self, index: usize) -> Result<Value> {
        let constant = self
            .vm
            .constants
            .get(index)
            .cloned()
            .expect("constant not in constant table");
        Ok(constant)
    }

    fn load(&self, env: &Environment, name: &str) -> Result<Value> {
        env.get(name)
            .cloned()
            .ok_or_else(|| Error::NameError(name.to_string()))
    }

    fn call(
        &self,
        env: &Environment,
        function: &Function,
        actual_parameters: &[Value],
    ) -> Result<Value> {
        use super::{
            instruction::Instruction::*,
            value::RawValue::*,
            value::Value::*,
        };
        function.check_arity(actual_parameters.len())?;

        let mut env = Environment::with_parent(env);
        for (name, value) in function.formal_parameters().iter().zip(actual_parameters) {
            env.set(name.to_string(), value.clone());
        }

        let mut stack = Vec::new();

        stack.reserve(function.body().stack_size());
        for ins in function.body() {
            match ins {
                Constant(index) => {
                    let constant = self.constant(index)?;
                    stack.push(constant);
                }
                InlineConstant(constant) => {
                    use super::instruction::InlineConstant;

                    let constant = match constant {
                        InlineConstant::Unit => ().into(),
                        InlineConstant::Bool(bool) => bool.into(),
                    };
                    stack.push(constant);
                }
                Unary(operator) => {
                    use super::ast::UnaryOperator::*;

                    let right = stack.pop().expect("empty stack");

                    let result = match operator {
                        Negate => Value::from(-right.as_number()?),
                        Not => Value::from(!right.as_bool()?),
                    };

                    stack.push(result);
                }
                Binary(operator) => {
                    use super::ast::BinaryOperator::*;

                    let right = stack.pop().expect("empty stack");
                    let left = stack.pop().expect("empty stack");

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

                    let result = match operator {
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

                    stack.push(result);
                }
            }
        }

        let result = stack.pop().expect("empty stack");
        assert!(stack.is_empty(), "stack not empty after execution");

        Ok(result)
    }
}
