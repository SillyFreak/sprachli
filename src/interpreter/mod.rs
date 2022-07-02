mod environment;
mod value;

use crate::ast;

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

    pub fn visit_source_file(&mut self, source: &ast::SourceFile) -> Result<Value> {
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

        let main = env.get("main")
            .ok_or_else(|| Error::NameError("main".to_string()))?
            .as_function()?;

        self.visit_call(&env, main, &[])
    }

    pub fn visit_call(&mut self, env: &Environment, function: &value::Function, actual_parameters: &[Value]) -> Result<Value> {
        function.check_arity(actual_parameters.len())?;

        let mut env = Environment::new();
        for (name, value) in function.formal_parameters().iter().zip(actual_parameters) {
            env.set(name.clone(), value.clone());
        }

        let body = function.body();

        if !body.statements.is_empty() {
            Err(Error::Unsupported("statements in function body"))?;
        }

        if let Some(expr) = &body.expression {
            self.visit_expression(&env, expr)
        } else {
            Ok(Value::Unit)
        }
    }

    pub fn visit_expression(&self, env: &Environment, expr: &ast::Expression) -> Result<Value> {
        let result = match expr {
            ast::Expression::Integer(value) => Ok(*value),
            _ => Err(Error::Unsupported("expression that is not an integer literal")),
        }?;
        Ok(Value::Number(result.into()))
    }
}