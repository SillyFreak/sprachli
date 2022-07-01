use std::fmt;

use super::{Declaration, Expression};

#[derive(Clone, PartialEq, Eq)]
pub enum Statement {
    Declaration(Declaration),
    Expression(Expression),
}

impl fmt::Debug for Statement {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Declaration(stmt) => write!(f, "{stmt:?}"),
            Self::Expression(stmt) => write!(f, "{stmt:?}"),
        }
    }
}
