use std::{fmt, sync::Arc};

use bigdecimal::BigDecimal;
use itertools::Itertools;

use super::instruction::InstructionSequence;
use super::{Error, Result};

pub type Number = BigDecimal;

#[derive(Clone, Hash, PartialEq, Eq)]
pub enum Value {
    Unit,
    Bool(bool),
    Boxed(Arc<RawValue>),
}

#[derive(Clone, Hash, PartialEq, Eq)]
pub enum RawValue {
    Number(Number),
    String(String),
    Function(Function),
}

impl fmt::Debug for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use Value::*;

        match self {
            Unit => f.write_str("unit"),
            Bool(value) => fmt::Display::fmt(value, f),
            Boxed(value) => value.fmt(f),
        }
    }
}

impl fmt::Debug for RawValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use RawValue::*;

        match self {
            Number(value) => fmt::Display::fmt(value, f),
            String(value) => fmt::Display::fmt(value, f),
            Function(value) => value.fmt(f),
        }
    }
}

impl Default for Value {
    fn default() -> Self {
        Self::unit()
    }
}

impl From<()> for Value {
    fn from(_: ()) -> Self {
        Self::unit()
    }
}

impl From<bool> for Value {
    fn from(value: bool) -> Self {
        Self::bool(value)
    }
}

impl From<Number> for Value {
    fn from(value: Number) -> Self {
        Self::number(value)
    }
}

impl From<String> for Value {
    fn from(value: String) -> Self {
        Self::string(value)
    }
}

impl From<Function> for Value {
    fn from(value: Function) -> Self {
        Self::function(value)
    }
}

impl Value {
    pub fn unit() -> Self {
        Self::Unit
    }

    pub fn bool(value: bool) -> Self {
        Self::Bool(value)
    }

    fn boxed(value: RawValue) -> Self {
        Self::Boxed(Arc::new(value))
    }

    pub fn number(value: Number) -> Self {
        Self::boxed(RawValue::Number(value))
    }

    pub fn string(value: String) -> Self {
        Self::boxed(RawValue::String(value))
    }

    pub fn function(value: Function) -> Self {
        Self::boxed(RawValue::Function(value))
    }

    pub fn is_unit(&self) -> bool {
        self == &Self::Unit
    }

    pub fn as_bool(&self) -> Result<bool> {
        use Value::*;

        if let Bool(value) = self {
            return Ok(*value);
        }

        Err(Error::TypeError("bool".to_string()))
    }

    pub fn as_number(&self) -> Result<&Number> {
        use RawValue::*;
        use Value::*;

        if let Boxed(value) = self {
            if let Number(value) = value.as_ref() {
                return Ok(value);
            }
        }

        Err(Error::TypeError("bool".to_string()))
    }

    pub fn as_string(&self) -> Result<&str> {
        use RawValue::*;
        use Value::*;

        if let Boxed(value) = self {
            if let String(value) = value.as_ref() {
                return Ok(value);
            }
        }

        Err(Error::TypeError("string".to_string()))
    }

    pub fn as_function(&self) -> Result<&Function> {
        use RawValue::*;
        use Value::*;

        if let Boxed(value) = self {
            if let Function(value) = value.as_ref() {
                return Ok(value);
            }
        }

        Err(Error::TypeError("function".to_string()))
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

    pub fn arity(&self) -> usize {
        self.formal_parameters.len()
    }

    pub fn check_arity(&self, actual_parameter_count: usize) -> Result<()> {
        if actual_parameter_count != self.arity() {
            Err(Error::ValueError(format!(
                "wrong parameter number; expected {}, got {}",
                self.arity(), actual_parameter_count,
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
