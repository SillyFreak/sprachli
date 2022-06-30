use super::{Declaration, Expression};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Statement {
    Declaration(Declaration),
    Expression(Expression),
}
