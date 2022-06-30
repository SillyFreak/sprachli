use super::Statement;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Expression {
    Integer(i32),
    Identifier(String),
    Block(Box<Block>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Block {
    pub statements: Vec<Statement>,
    pub expression: Option<Expression>,
}
