use std::fmt;

use super::Statement;

#[derive(Clone, PartialEq, Eq)]
pub enum Expression {
    Integer(i32),
    Identifier(String),
    Binary(Box<Binary>),
    Unary(Box<Unary>),
    Call(Call),
    Block(Box<Block>),
    If(If),
}

impl fmt::Debug for Expression {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Integer(value) => write!(f, "{value:?}"),
            Self::Identifier(name) => write!(f, "{name}"),
            Self::Binary(expr) => write!(f, "{expr:?}"),
            Self::Unary(expr) => write!(f, "{expr:?}"),
            Self::Call(expr) => write!(f, "{expr:?}"),
            Self::Block(expr) => write!(f, "{expr:?}"),
            Self::If(expr) => write!(f, "{expr:?}"),
        }
    }
}

#[derive(Clone, PartialEq, Eq)]
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

impl fmt::Debug for BinaryOperator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Equals => write!(f, "=="),
            Self::NotEquals => write!(f, "!="),
            Self::Greater => write!(f, ">"),
            Self::GreaterEquals => write!(f, ">="),
            Self::Less => write!(f, "<"),
            Self::LessEquals => write!(f, "<="),
            Self::Add => write!(f, "+"),
            Self::Subtract => write!(f, "-"),
            Self::Multiply => write!(f, "*"),
            Self::Divide => write!(f, "/"),
        }
    }
}

#[derive(Clone, PartialEq, Eq)]
pub struct Binary {
    pub operator: BinaryOperator,
    pub left: Expression,
    pub right: Expression,
}

impl fmt::Debug for Binary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("binary").field(&self.operator).field(&self.left).field(&self.right).finish()
    }
}

#[derive(Clone, PartialEq, Eq)]
pub enum UnaryOperator {
    // negation
    Negate,
    // logical inverse
    Invert,
}

impl fmt::Debug for UnaryOperator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Negate => write!(f, "-"),
            Self::Invert => write!(f, "!"),
        }
    }
}

#[derive(Clone, PartialEq, Eq)]
pub struct Unary {
    pub operator: UnaryOperator,
    pub right: Expression,
}

impl fmt::Debug for Unary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("unary").field(&self.operator).field(&self.right).finish()
    }
}

#[derive(Clone, PartialEq, Eq)]
pub struct Call {
    pub function: Box<Expression>,
    pub actual_parameters: Vec<Expression>,
}

impl fmt::Debug for Call {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut f = f.debug_tuple("call");
        f.field(&self.function);
        for actual_parameter in &self.actual_parameters {
            f.field(actual_parameter);
        }
        f.finish()
    }
}

#[derive(Clone, PartialEq, Eq)]
pub struct Block {
    pub statements: Vec<Statement>,
    pub expression: Option<Expression>,
}

impl fmt::Debug for Block {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut f = f.debug_tuple("block");
        for statement in &self.statements {
            f.field(statement);
        }
        if let Some(expression) = &self.expression {
            f.field(&expression);
        } else {
            f.field(&());
        }
        f.finish()
    }
}

#[derive(Clone, PartialEq, Eq)]
pub struct If {
    pub then_branches: Vec<(Expression, Block)>,
    pub else_branch: Option<Box<Block>>,
}

impl fmt::Debug for If {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut f = f.debug_tuple("if");
        for (condition, block) in &self.then_branches {
            f.field(condition).field(block);
        }
        if let Some(else_branch) = &self.else_branch {
            f.field(&else_branch);
        } else {
            f.field(&());
        }
        f.finish()
    }
}
