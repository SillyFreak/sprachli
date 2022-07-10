use std::fmt;

use bigdecimal::BigDecimal;
use itertools::Itertools;

use crate::ast::Block;
use super::{Error, Result};

pub type Number = BigDecimal;

#[derive(Clone, PartialEq, Eq)]
pub enum Value {
    Unit,
    Bool(bool),
    Number(Number),
    Function(Function),
}

impl fmt::Debug for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Unit => f.write_str("unit"),
            Self::Bool(value) => fmt::Display::fmt(value, f),
            Self::Number(value) => fmt::Display::fmt(value, f),
            Self::Function(value) => value.fmt(f),
        }
    }
}

impl Value {
    pub fn as_bool(&self) -> Result<bool> {
        match self {
            Self::Bool(bool) => Ok(*bool),
            _ => Err(Error::TypeError("bool".to_string())),
        }
    }

    pub fn as_number(&self) -> Result<&Number> {
        match self {
            Self::Number(number) => Ok(number),
            _ => Err(Error::TypeError("number".to_string())),
        }
    }

    pub fn as_function(&self) -> Result<&Function> {
        match self {
            Self::Function(function) => Ok(function),
            _ => Err(Error::TypeError("function".to_string())),
        }
    }
}

#[derive(Clone, PartialEq, Eq)]
pub struct Function {
    formal_parameters: Vec<String>,
    body: Block,
}

impl fmt::Debug for Function {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("fn (")?;
        for str in self.formal_parameters.iter().map(String::as_str).intersperse(", ") {
            f.write_str(str)?;
        }
        f.write_str(") { ... }")
    }
}

impl Function {
    pub fn new(formal_parameters: Vec<String>, body: Block) -> Self {
        Self { formal_parameters, body }
    }

    pub fn check_arity(&self, actual_parameter_count: usize) -> Result<()> {
        let expected_parameter_count = self.formal_parameters.len();
        if actual_parameter_count != expected_parameter_count {
            Err(Error::ValueError(format!(
                "wrong parameter number; expected {}, got {}",
                expected_parameter_count,
                actual_parameter_count,
            )))?;
        }
        Ok(())
    }

    pub fn formal_parameters(&self) -> &[String] {
        &self.formal_parameters
    }

    pub fn body(&self) -> &Block {
        &self.body
    }
}
