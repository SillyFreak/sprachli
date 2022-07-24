use std::{fmt, sync::Arc};

use super::{Error, Result};
use crate::bytecode::Constant;

pub use crate::bytecode::{Function, Number};

#[derive(Clone)]
pub enum Value<'b> {
    Unit,
    Bool(bool),
    Constant(Constant<'b>),
    Boxed(Arc<BoxedValue>),
}

#[derive(Clone)]
pub enum BoxedValue {
    Number(Number),
    String(String),
}

#[derive(Clone)]
pub enum ValueRef<'a, 'b> {
    Number(&'a Number),
    String(&'a str),
    Function(&'a Function<'b>),
}

impl<'b> Value<'b> {
    pub fn unit() -> Self {
        Self::Unit
    }

    pub fn bool(value: bool) -> Self {
        Self::Bool(value)
    }

    pub fn constant(value: Constant<'b>) -> Self {
        Self::Constant(value)
    }

    fn boxed(value: BoxedValue) -> Self {
        Self::Boxed(Arc::new(value))
    }

    pub fn number(value: Number) -> Self {
        Self::boxed(BoxedValue::Number(value))
    }

    pub fn string(value: String) -> Self {
        Self::boxed(BoxedValue::String(value))
    }

    pub fn get_ref<'a>(&'a self) -> Option<ValueRef<'a, 'b>>
    where
        'a: 'b,
    {
        use self::Constant as C;
        use BoxedValue as B;
        use Value::*;
        use ValueRef as R;

        let result = match self {
            Constant(C::Number(value)) => R::Number(value),
            Constant(C::String(value)) => R::String(value),
            Constant(C::Function(value)) => R::Function(value),
            Boxed(arc) => match arc.as_ref() {
                B::Number(value) => R::Number(value),
                B::String(value) => R::String(value),
            },
            _ => None?,
        };

        Some(result)
    }

    pub fn is_unit(&self) -> bool {
        matches!(self, Self::Unit)
    }

    pub fn as_bool(&self) -> Result<bool> {
        use Value::*;

        if let Bool(value) = self {
            Ok(*value)
        } else {
            Err(Error::TypeError("bool".to_string()))
        }
    }

    pub fn as_number(&self) -> Result<&Number> {
        use ValueRef::*;

        if let Some(Number(value)) = self.get_ref() {
            Ok(value)
        } else {
            Err(Error::TypeError("number".to_string()))
        }
    }

    pub fn as_string(&self) -> Result<&str> {
        use ValueRef::*;

        if let Some(String(value)) = self.get_ref() {
            Ok(value)
        } else {
            Err(Error::TypeError("string".to_string()))
        }
    }

    pub fn as_function(&self) -> Result<&Function> {
        use ValueRef::*;

        if let Some(Function(value)) = self.get_ref() {
            Ok(value)
        } else {
            Err(Error::TypeError("function".to_string()))
        }
    }
}

impl fmt::Debug for Value<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use Value::*;

        match self {
            Unit => f.write_str("unit"),
            Bool(value) => fmt::Display::fmt(value, f),
            Constant(value) => value.fmt(f),
            Boxed(value) => value.fmt(f),
        }
    }
}

impl fmt::Debug for BoxedValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use BoxedValue::*;

        match self {
            Number(value) => fmt::Display::fmt(value, f),
            String(value) => fmt::Display::fmt(value, f),
        }
    }
}

// pub fn check_arity(&self, actual_parameter_count: usize) -> Result<()> {
//     if actual_parameter_count != self.arity {
//         Err(Error::ValueError(format!(
//             "wrong parameter number; expected {}, got {}",
//             self.arity, actual_parameter_count,
//         )))?;
//     }
//     Ok(())
// }
