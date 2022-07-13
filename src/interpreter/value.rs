use std::fmt;

use bigdecimal::BigDecimal;
use itertools::Itertools;

use super::{Error, Result};
use crate::ast::Block;

pub type Number = BigDecimal;

#[derive(Clone, PartialEq, Eq)]
pub enum Value<'input> {
    Unit,
    Bool(bool),
    Number(Number),
    String(String),
    Function(Function<'input>),
}

impl fmt::Debug for Value<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Unit => f.write_str("unit"),
            Self::Bool(value) => fmt::Display::fmt(value, f),
            Self::Number(value) => fmt::Display::fmt(value, f),
            Self::String(value) => fmt::Display::fmt(value, f),
            Self::Function(value) => value.fmt(f),
        }
    }
}

impl<'input> Value<'input> {
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

    pub fn as_string(&self) -> Result<&str> {
        match self {
            Self::String(string) => Ok(string),
            _ => Err(Error::TypeError("string".to_string())),
        }
    }

    pub fn as_function(&self) -> Result<&Function<'input>> {
        match self {
            Self::Function(function) => Ok(function),
            _ => Err(Error::TypeError("function".to_string())),
        }
    }
}

#[derive(Clone, PartialEq, Eq)]
pub struct Function<'input> {
    formal_parameters: Vec<&'input str>,
    body: Block<'input>,
}

impl fmt::Debug for Function<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("fn (")?;
        for str in self.formal_parameters.iter().copied().intersperse(", ") {
            f.write_str(str)?;
        }
        f.write_str(") { ... }")
    }
}

impl<'input> Function<'input> {
    pub fn new(formal_parameters: Vec<&'input str>, body: Block<'input>) -> Self {
        Self {
            formal_parameters,
            body,
        }
    }

    pub fn check_arity(&self, actual_parameter_count: usize) -> Result<()> {
        let expected_parameter_count = self.formal_parameters.len();
        if actual_parameter_count != expected_parameter_count {
            Err(Error::ValueError(format!(
                "wrong parameter number; expected {}, got {}",
                expected_parameter_count, actual_parameter_count,
            )))?;
        }
        Ok(())
    }

    pub fn formal_parameters(&self) -> &[&'input str] {
        &self.formal_parameters
    }

    pub fn body(&self) -> &Block<'input> {
        &self.body
    }
}
