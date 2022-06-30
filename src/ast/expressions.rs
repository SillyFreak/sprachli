use super::Statement;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Expression {
    Integer(i32),
    Identifier(String),
    Block(Box<Block>),
    If(If),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Block {
    pub statements: Vec<Statement>,
    pub expression: Option<Expression>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct If {
    pub then_branches: Vec<(Expression, Block)>,
    pub else_branch: Option<Box<Block>>,
}
