mod environment;
mod value;

use crate::ast;

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

    pub fn visit_source_file(&self, source: &ast::SourceFile) -> Result<Value> {
        use ast::Declaration::*;
        
        let mut env = Environment::new();

        for decl in &source.declarations {
            match decl {
                Use(_decl) => Err(Error::Unsupported("use declaration"))?,
                Fn(ast::Fn { name, formal_parameters, body, .. }) => {
                    let function = value::Function::new(formal_parameters.clone(), body.clone());
                    env.set(name.clone(), Value::Function(function));
                },
                Struct(_decl) => Err(Error::Unsupported("struct"))?,
                Mixin(_decl) => Err(Error::Unsupported("mixin"))?,
                Impl(_decl) => Err(Error::Unsupported("impl block"))?,
            }
        }

        let main = self.visit_identifier(&env, "main")?;
        let main = main.as_function()?;
        self.visit_invoke(&env, main, &[])
    }

    pub fn visit_expression(&self, env: &Environment, expr: &ast::Expression) -> Result<Value> {
        use ast::Expression::*;

        match expr {
            Number(value) => self.visit_number(env, value),
            Identifier(name) => self.visit_identifier(env, name),
            Binary(expr) => self.visit_binary(env, expr),
            Unary(expr) => self.visit_unary(env, expr),
            Call(expr) => self.visit_call(env, expr),
            Block(expr) => self.visit_block(env, expr),
            // If(expr) => self.visit_expr(env, expr),
            _ => Err(Error::Unsupported("expression that is not an number literal")),
        }
    }

    fn visit_number(&self, _env: &Environment, value: &value::Number) -> Result<Value> {
        Ok(Value::Number(value.clone()))
    }

    fn visit_identifier(&self, env: &Environment, name: &str) -> Result<Value> {
        env.get(name).cloned().ok_or_else(|| Error::NameError(name.to_string()))
    }

    fn visit_binary(&self, env: &Environment, expr: &ast::Binary) -> Result<Value> {
        use ast::BinaryOperator::*;
        use Value::*;

        let equality_comparison = |eq: bool| -> Result<Value> {
            let left = self.visit_expression(env, &expr.left)?;
            let right = self.visit_expression(env, &expr.right)?;

            let result = match (left, right) {
                (Unit, Unit) => true,
                (Bool(left), Bool(right)) => left == right,
                (Number(left), Number(right)) => left == right,
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

    fn visit_unary(&self, env: &Environment, expr: &ast::Unary) -> Result<Value> {
        use ast::UnaryOperator::*;

        match expr.operator {
            Negate => {
                let right = self.visit_expression(env, &expr.right)?;
                let result = -right.as_number()?;
                Ok(Value::Number(result))
            },
            Not => {
                let right = self.visit_expression(env, &expr.right)?;
                let result = !right.as_bool()?;
                Ok(Value::Bool(result))
            },
        }
    }

    fn visit_call(&self, env: &Environment, call: &ast::Call) -> Result<Value> {
        let function = self.visit_expression(env, &call.function)?;
        let function = function.as_function()?;
        let actual_parameters = call.actual_parameters.iter()
                .map(|expr| self.visit_expression(env, expr))
                .collect::<Result<Vec<_>>>()?;

        self.visit_invoke(&env, function, &actual_parameters)
    }

    fn visit_invoke(&self, env: &Environment, function: &value::Function, actual_parameters: &[Value]) -> Result<Value> {
        function.check_arity(actual_parameters.len())?;

        let mut env = Environment::with_parent(env);
        for (name, value) in function.formal_parameters().iter().zip(actual_parameters) {
            env.set(name.clone(), value.clone());
        }

        self.visit_block(&env, function.body())
    }

    fn visit_block(&self, env: &Environment, block: &ast::Block) -> Result<Value> {
        if !block.statements.is_empty() {
            Err(Error::Unsupported("statements in block"))?;
        }

        if let Some(expr) = &block.expression {
            self.visit_expression(&env, expr)
        } else {
            Ok(Value::Unit)
        }
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
        assert_eq!(run("fn main() { 21 * 2 }").unwrap(), Value::Number(42.into()));
        assert_eq!(run("fn main() { 22 + 20 }").unwrap(), Value::Number(42.into()));
        assert_eq!(run("fn main() { 22 >= 20 }").unwrap(), Value::Bool(true));
        assert_eq!(run("fn main() { -22 < 20 }").unwrap(), Value::Bool(true));
        assert_eq!(run("fn main() { 5 == 10 }").unwrap(), Value::Bool(false));
        assert_eq!(run("fn foo() { 42 } fn main() { foo() }").unwrap(), Value::Number(42.into()));
        assert_eq!(run("fn foo(a) { -a } fn main() { foo(-42) }").unwrap(), Value::Number(42.into()));
        assert_eq!(run("fn main() { 5 == { 10 } }").unwrap(), Value::Bool(false));
    }
}