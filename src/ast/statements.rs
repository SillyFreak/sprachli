use std::fmt;

use super::{Declaration, Expression};

#[derive(Clone, PartialEq, Eq)]
pub enum Statement<'input> {
    Declaration(Declaration<'input>),
    Expression(Expression<'input>),
}

impl fmt::Debug for Statement<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Declaration(stmt) => stmt.fmt(f),
            Self::Expression(stmt) => stmt.fmt(f),
        }
    }
}
