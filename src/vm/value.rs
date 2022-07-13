use std::{fmt, sync::Arc};

use bigdecimal::BigDecimal;
use itertools::Itertools;

use super::instruction::InstructionSequence;
use super::{Error, Result};

pub type Number = BigDecimal;

#[derive(Clone, Hash, PartialEq, Eq)]
pub enum RawValue {
    Unit,
    Bool(bool),
    Number(Number),
    String(String),
    Function(Function),
}

#[derive(Clone, Hash, PartialEq, Eq)]
pub struct Value(Arc<RawValue>);

impl fmt::Debug for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use RawValue::*;

        match self.get() {
            Unit => f.write_str("unit"),
            Bool(value) => fmt::Display::fmt(value, f),
            Number(value) => fmt::Display::fmt(value, f),
            String(value) => fmt::Display::fmt(value, f),
            Function(value) => value.fmt(f),
        }
    }
}

impl From<()> for Value {
    fn from(_: ()) -> Self {
        Self::new(RawValue::Unit)
    }
}

impl From<bool> for Value {
    fn from(value: bool) -> Self {
        Self::new(RawValue::Bool(value))
    }
}

impl From<Number> for Value {
    fn from(value: Number) -> Self {
        Self::new(RawValue::Number(value))
    }
}

impl From<String> for Value {
    fn from(value: String) -> Self {
        Self::new(RawValue::String(value))
    }
}

impl From<Function> for Value {
    fn from(value: Function) -> Self {
        Self::new(RawValue::Function(value))
    }
}

impl Value {
    pub fn new(value: RawValue) -> Self {
        Self(Arc::new(value))
    }

    pub fn get(&self) -> &RawValue {
        &self.0
    }

    pub fn as_bool(&self) -> Result<bool> {
        use RawValue::*;

        match self.get() {
            Bool(bool) => Ok(*bool),
            _ => Err(Error::TypeError("bool".to_string())),
        }
    }

    pub fn as_number(&self) -> Result<&Number> {
        use RawValue::*;

        match self.get() {
            Number(number) => Ok(number),
            _ => Err(Error::TypeError("number".to_string())),
        }
    }

    pub fn as_string(&self) -> Result<&str> {
        use RawValue::*;

        match self.get() {
            String(string) => Ok(string),
            _ => Err(Error::TypeError("string".to_string())),
        }
    }

    pub fn as_function(&self) -> Result<&Function> {
        use RawValue::*;

        match self.get() {
            Function(function) => Ok(function),
            _ => Err(Error::TypeError("function".to_string())),
        }
    }
}

#[derive(Clone, Hash, PartialEq, Eq)]
pub struct Function {
    formal_parameters: Vec<String>,
    body: InstructionSequence,
}

impl fmt::Debug for Function {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("fn (")?;
        for str in self
            .formal_parameters
            .iter()
            .map(String::as_str)
            .intersperse(", ")
        {
            f.write_str(str)?;
        }
        f.write_str(") { ... }")
    }
}

impl Function {
    pub fn new(formal_parameters: Vec<String>, body: InstructionSequence) -> Self {
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

    pub fn formal_parameters(&self) -> &[String] {
        &self.formal_parameters
    }

    pub fn body(&self) -> &InstructionSequence {
        &self.body
    }
}
