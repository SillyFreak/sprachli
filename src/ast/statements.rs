use std::fmt;

use super::{Declaration, Expression};

#[derive(Clone, PartialEq, Eq)]
pub enum Statement<'input> {
    Declaration(Declaration<'input>),
    Expression(Expression<'input>),
}

impl Statement<'_> {
    pub(super) fn is_simple(&self) -> bool {
        use Statement::*;

        match self {
            Expression(expr) => expr.is_simple(),
            _ => false,
        }
    }
}

impl fmt::Debug for Statement<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Declaration(stmt) => stmt.fmt(f),
            Self::Expression(stmt) => stmt.fmt(f),
        }
    }
}
