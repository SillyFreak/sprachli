use super::Statement;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Expression {
    Integer(i32),
    Identifier(String),
    Binary(Box<Binary>),
    Unary(Box<Unary>),
    Block(Box<Block>),
    If(If),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BinaryOperator {
    // equality
    Equals,
    NotEquals,
    // comparison
    Greater,
    GreaterEquals,
    Less,
    LessEquals,
    // additive
    Add,
    Subtract,
    // multiplicative
    Multiply,
    Divide,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Binary {
    pub operator: BinaryOperator,
    pub left: Expression,
    pub right: Expression,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UnaryOperator {
    // negation
    Negate,
    // logical inverse
    Invert,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Unary {
    pub operator: UnaryOperator,
    pub right: Expression,
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
