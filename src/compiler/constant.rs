use std::fmt;

use bigdecimal::BigDecimal;
use itertools::Itertools;

use super::instruction::Instruction;
use super::Module;

pub type Number = BigDecimal;

#[derive(Clone, Hash, PartialEq, Eq)]
pub enum Constant {
    Number(Number),
    String(String),
    Function(Function),
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

impl Constant {
    pub(crate) fn fmt_with(
        &self,
        f: &mut fmt::Formatter<'_>,
        module: Option<&Module>,
    ) -> fmt::Result {
        use fmt::Debug;
        use Constant::*;

        match self {
            Number(value) => fmt::Display::fmt(value, f),
            String(value) => value.fmt(f),
            Function(value) => value.fmt_with(f, module),
        }
    }
}

impl fmt::Debug for Constant {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.fmt_with(f, None)
    }
}

#[derive(Clone, Hash, PartialEq, Eq)]
pub struct Function {
    arity: usize,
    body: Vec<Instruction>,
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

    pub(crate) fn fmt_with(
        &self,
        f: &mut fmt::Formatter<'_>,
        module: Option<&Module>,
    ) -> fmt::Result {
        f.write_str("fn (")?;
        for i in (0..self.arity).map(Some).intersperse(None) {
            match i {
                Some(i) => write!(f, "_{}", i)?,
                None => f.write_str(", ")?,
            }
        }

        if f.alternate() {
            f.write_str(") {\n")?;
            self.fmt_body_with(f, module)?;
            f.write_str("\n           }")?;
        } else {
            f.write_str(") { ... }")?;
        }
        Ok(())
    }

    pub(crate) fn fmt_body_with(
        &self,
        f: &mut fmt::Formatter<'_>,
        module: Option<&Module>,
    ) -> fmt::Result {
        use fmt::Debug;

        let mut offset = 0;
        if f.alternate() {
            for ins in self.body.iter().map(Some).intersperse_with(|| None) {
                if let Some(ins) = ins {
                    write!(f, "           {offset:5}  ")?;
                    ins.fmt_with(f, module)?;
                    offset += ins.encoded_len();
                } else {
                    f.write_str("\n")?;
                }
            }
            Ok(())
        } else {
            self.body.fmt(f)
        }
    }
}

impl fmt::Debug for Function {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.fmt_with(f, None)
    }
}
