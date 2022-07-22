use std::fmt;

use bigdecimal::BigDecimal;
use itertools::Itertools;

use super::instruction::Instruction;

pub type Number = BigDecimal;

#[derive(Clone, Hash, PartialEq, Eq)]
pub enum Constant {
    Number(Number),
    String(String),
    Function(Function),
}

impl fmt::Debug for Constant {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use Constant::*;

        match self {
            Number(value) => fmt::Display::fmt(value, f),
            String(value) => fmt::Display::fmt(value, f),
            Function(value) => value.fmt(f),
        }
    }
}

impl From<Number> for Constant {
    fn from(value: Number) -> Self {
        Self::Number(value)
    }
}

impl From<String> for Constant {
    fn from(value: String) -> Self {
        Self::String(value)
    }
}

impl From<Function> for Constant {
    fn from(value: Function) -> Self {
        Self::Function(value)
    }
}

#[derive(Clone, Hash, PartialEq, Eq)]
pub struct Function {
    arity: usize,
    body: Vec<Instruction>,
}

impl fmt::Debug for Function {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("fn (")?;
        for i in (0..self.arity).map(Some).intersperse(None) {
            match i {
                Some(i) => write!(f, "_{}", i)?,
                None => f.write_str(", ")?,
            }
        }
        f.write_str(") { ... }")
    }
}

impl Function {
    pub fn new(arity: usize, body: Vec<Instruction>) -> Self {
        Self { arity, body }
    }

    pub fn arity(&self) -> usize {
        self.arity
    }

    pub fn body(&self) -> &[Instruction] {
        &self.body
    }
}
