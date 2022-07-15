mod environment;
mod value;

use std::str::FromStr;

use crate::{ast, grammar::string_from_literal};

use bigdecimal::BigDecimal;
pub use environment::Environment;
pub use value::Value;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Name not known: {0}")]
    NameError(String),
    #[error("Type error, expected: {0}")]
    TypeError(String),
    #[error("Value error: {0}")]
    ValueError(String),
    #[error("Unsupported language construct: {0}")]
    Unsupported(&'static str),
}

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Default, Debug, Clone)]
pub struct Interpreter;

impl Interpreter {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn visit_source_file<'input>(
        &self,
        source: &ast::SourceFile<'input>,
    ) -> Result<Value<'input>> {
        use ast::Declaration::*;

        let mut env = Environment::new();

        for decl in &source.declarations {
            match decl {
                Use(_decl) => Err(Error::Unsupported("use declaration"))?,
                Fn(ast::Fn {
                    name,
                    formal_parameters,
                    body,
                    ..
                }) => {
                    let function = value::Function::new(formal_parameters.clone(), body.clone());
                    env.set(name.to_string(), Value::Function(function));
                }
                Struct(_decl) => Err(Error::Unsupported("struct"))?,
                Mixin(_decl) => Err(Error::Unsupported("mixin"))?,
                Impl(_decl) => Err(Error::Unsupported("impl block"))?,
            }
        }

        let main = self.visit_identifier(&env, "main")?;
        let main = main.as_function()?;
        self.visit_invoke(&env, main, &[])
    }

    pub fn visit_expression<'input>(
        &self,
        env: &Environment<'input, '_>,
        expr: &ast::Expression,
    ) -> Result<Value<'input>> {
        use ast::Expression::*;

        match expr {
            Number(literal) => self.visit_number(env, literal),
            String(literal) => self.visit_string(env, literal),
            Identifier(name) => self.visit_identifier(env, name),
            Binary(expr) => self.visit_binary(env, expr),
            Unary(expr) => self.visit_unary(env, expr),
            Call(expr) => self.visit_call(env, expr),
            Block(expr) => self.visit_block(env, expr),
            If(expr) => self.visit_if(env, expr),
        }
    }

    fn visit_number<'input>(
        &self,
        _env: &Environment<'input, '_>,
        literal: &str,
    ) -> Result<Value<'input>> {
        let number = value::Number::from_str(literal).expect("number liteal is not a valid number");
        Ok(Value::Number(number))
    }

    fn visit_string<'input>(
        &self,
        _env: &Environment<'input, '_>,
        literal: &str,
    ) -> Result<Value<'input>> {
        let string = string_from_literal(literal).expect("string liteal is not a valid string");
        Ok(Value::String(string))
    }

    fn visit_identifier<'input>(
        &self,
        env: &Environment<'input, '_>,
        name: &str,
    ) -> Result<Value<'input>> {
        let x = env.get(name).cloned();
        x.ok_or_else(|| Error::NameError(name.to_string()))
    }

    fn visit_binary<'input>(
        &self,
        env: &Environment<'input, '_>,
        expr: &ast::Binary,
    ) -> Result<Value<'input>> {
        use ast::BinaryOperator::*;
        use Value::*;

        let equality_comparison = |eq: bool| -> Result<Value<'input>> {
            let left = self.visit_expression(env, &expr.left)?;
            let right = self.visit_expression(env, &expr.right)?;

            let result = match (left, right) {
                (Unit, Unit) => true,
                (Bool(left), Bool(right)) => left == right,
                (Number(left), Number(right)) => left == right,
                (String(left), String(right)) => left == right,
                // TODO functions?
                _ => false,
            };

            Ok(Value::Bool(result == eq))
        };

        let number_comparison = |op: fn(&BigDecimal, &BigDecimal) -> bool| {
            let left = self.visit_expression(env, &expr.left)?;
            let right = self.visit_expression(env, &expr.right)?;

            let result = op(left.as_number()?, right.as_number()?);
            Ok(Value::Bool(result))
        };

        let arithmetic = |op: fn(&BigDecimal, &BigDecimal) -> BigDecimal| {
            let left = self.visit_expression(env, &expr.left)?;
            let right = self.visit_expression(env, &expr.right)?;

            let result = op(left.as_number()?, right.as_number()?);
            Ok(Value::Number(result))
        };

        match expr.operator {
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
        }
    }

    fn visit_unary<'input>(
        &self,
        env: &Environment<'input, '_>,
        expr: &ast::Unary,
    ) -> Result<Value<'input>> {
        use ast::UnaryOperator::*;

        match expr.operator {
            Negate => {
                let right = self.visit_expression(env, &expr.right)?;
                let result = -right.as_number()?;
                Ok(Value::Number(result))
            }
            Not => {
                let right = self.visit_expression(env, &expr.right)?;
                let result = !right.as_bool()?;
                Ok(Value::Bool(result))
            }
        }
    }

    fn visit_call<'input>(
        &self,
        env: &Environment<'input, '_>,
        call: &ast::Call,
    ) -> Result<Value<'input>> {
        let function = self.visit_expression(env, &call.function)?;
        let function = function.as_function()?;
        let actual_parameters = call
            .actual_parameters
            .iter()
            .map(|expr| self.visit_expression(env, expr))
            .collect::<Result<Vec<_>>>()?;

        self.visit_invoke(env, function, &actual_parameters)
    }

    fn visit_invoke<'input>(
        &self,
        env: &Environment<'input, '_>,
        function: &value::Function,
        actual_parameters: &[Value<'input>],
    ) -> Result<Value<'input>> {
        function.check_arity(actual_parameters.len())?;

        let mut env = Environment::with_parent(env);
        for (name, value) in function.formal_parameters().iter().zip(actual_parameters) {
            env.set(name.to_string(), value.clone());
        }

        self.visit_block(&env, function.body())
    }

    fn visit_block<'input>(
        &self,
        env: &Environment<'input, '_>,
        block: &ast::Block,
    ) -> Result<Value<'input>> {
        if !block.statements.is_empty() {
            Err(Error::Unsupported("statements in block"))?;
        }

        if let Some(expr) = &block.expression {
            self.visit_expression(env, expr)
        } else {
            Ok(Value::Unit)
        }
    }

    fn visit_if<'input>(
        &self,
        env: &Environment<'input, '_>,
        expr: &ast::If,
    ) -> Result<Value<'input>> {
        for (condition, then_branch) in &expr.then_branches {
            let condition = self.visit_expression(env, condition)?;
            let condition = condition.as_bool()?;

            if condition {
                return self.visit_block(env, then_branch);
            }
        }

        if let Some(else_branch) = &expr.else_branch {
            return self.visit_block(env, else_branch);
        }

        Ok(Value::Unit)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::grammar::SourceFileParser;

    fn run(source: &str) -> Result<Value> {
        let source = SourceFileParser::new().parse(source).unwrap();
        Interpreter::new().visit_source_file(&source)
    }

    #[test]
    fn test_interpreter() {
        assert_eq!(run("fn main() {}").unwrap(), Value::Unit);
        assert_eq!(run("fn main() { 42 }").unwrap(), Value::Number(42.into()));
        assert_eq!(
            run("fn main() { 21 * 2 }").unwrap(),
            Value::Number(42.into())
        );
        assert_eq!(
            run("fn main() { 22 + 20 }").unwrap(),
            Value::Number(42.into())
        );
        assert_eq!(run("fn main() { 22 >= 20 }").unwrap(), Value::Bool(true));
        assert_eq!(run("fn main() { -22 < 20 }").unwrap(), Value::Bool(true));
        assert_eq!(run("fn main() { 5 == 10 }").unwrap(), Value::Bool(false));
        assert_eq!(
            run("fn foo() { 42 } fn main() { foo() }").unwrap(),
            Value::Number(42.into())
        );
        assert_eq!(
            run("fn foo(a) { -a } fn main() { foo(-42) }").unwrap(),
            Value::Number(42.into())
        );
        assert_eq!(
            run("fn main() { 5 == { 10 } }").unwrap(),
            Value::Bool(false)
        );

        let source = "
        fn max(a, b) {
            if a > b { a } else { b }
        }

        fn main() {
            max(2, 42)
        }
        ";
        assert_eq!(run(source).unwrap(), Value::Number(42.into()));

        let source = "
        fn is_even(x) {
            if x >= 2 { is_even(x - 2) } else { x == 0 }
        }

        fn main() {
            is_even(42)
        }
        ";
        assert_eq!(run(source).unwrap(), Value::Bool(true));
    }
}
