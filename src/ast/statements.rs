use std::fmt;

use super::{Declaration, Expression};
use crate::fmt::FormatterExt;

#[derive(Clone, PartialEq, Eq)]
pub enum Statement<'input> {
    Declaration(Declaration<'input>),
    Expression(Expression<'input>),
    Jump(Jump<'input>),
    VariableDeclaration(VariableDeclaration<'input>),
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
            Self::Jump(stmt) => stmt.fmt(f),
            Self::VariableDeclaration(stmt) => stmt.fmt(f),
        }
    }
}

#[derive(Clone, PartialEq, Eq)]
pub enum Jump<'input> {
    Return(Option<Box<Expression<'input>>>),
    Break(Option<Box<Expression<'input>>>),
    Continue,
}

impl<'input> Jump<'input> {
    pub fn new_return(right: Option<Expression<'input>>) -> Self {
        let right = right.map(Box::new);
        Self::Return(right)
    }

    pub fn new_break(right: Option<Expression<'input>>) -> Self {
        let right = right.map(Box::new);
        Self::Break(right)
    }
}

impl<'input> From<Jump<'input>> for Statement<'input> {
    fn from(value: Jump<'input>) -> Self {
        Statement::Jump(value)
    }
}

impl fmt::Debug for Jump<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use Jump::*;

        match self {
            Return(expr) => {
                let compact = expr.as_deref().map_or(true, Expression::is_simple);
                f.debug_sexpr_compact(compact)
                    .name("return")
                    .items(expr.iter())
                    .finish()
            }
            Break(expr) => {
                let compact = expr.as_deref().map_or(true, Expression::is_simple);
                f.debug_sexpr_compact(compact)
                    .name("break")
                    .items(expr.iter())
                    .finish()
            }
            Continue => f.debug_sexpr_compact(true).name("continue").finish(),
        }
    }
}

#[derive(Clone, PartialEq, Eq)]
pub struct VariableDeclaration<'input> {
    pub mutable: bool,
    pub name: &'input str,
    pub initializer: Option<Expression<'input>>,
}

impl<'input> VariableDeclaration<'input> {
    pub fn new(mutable: bool, name: &'input str, initializer: Option<Expression<'input>>) -> Self {
        Self {
            mutable,
            name,
            initializer,
        }
    }
}

impl<'input> From<VariableDeclaration<'input>> for Statement<'input> {
    fn from(value: VariableDeclaration<'input>) -> Self {
        Statement::VariableDeclaration(value)
    }
}

impl fmt::Debug for VariableDeclaration<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let compact = self
            .initializer
            .as_ref()
            .map_or(true, Expression::is_simple);
        let mut f = f.debug_sexpr_compact(compact);
        f.name("let");
        if self.mutable {
            f.compact_name("mut");
        }
        f.compact_name(self.name)
            .items(self.initializer.iter())
            .finish()
    }
}
