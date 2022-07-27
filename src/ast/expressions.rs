use std::fmt;

use num_enum::{IntoPrimitive, TryFromPrimitive};

use super::Statement;
use crate::fmt::FormatterExt;

#[derive(Clone, PartialEq, Eq)]
pub enum Expression<'input> {
    Number(&'input str),
    String(&'input str),
    Identifier(&'input str),
    Jump(Jump<'input>),
    Binary(Binary<'input>),
    Unary(Unary<'input>),
    Call(Call<'input>),
    Block(Block<'input>),
    If(If<'input>),
    Loop(Loop<'input>),
}

impl fmt::Debug for Expression<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Number(value) => fmt::Display::fmt(value, f),
            Self::String(value) => fmt::Display::fmt(value, f),
            Self::Identifier(name) => f.write_str(name),
            Self::Jump(expr) => expr.fmt(f),
            Self::Binary(expr) => expr.fmt(f),
            Self::Unary(expr) => expr.fmt(f),
            Self::Call(expr) => expr.fmt(f),
            Self::Block(expr) => expr.fmt(f),
            Self::If(expr) => expr.fmt(f),
            Self::Loop(expr) => expr.fmt(f),
        }
    }
}

#[derive(Clone, PartialEq, Eq)]
pub enum Jump<'input> {
    Return(Option<Box<Expression<'input>>>),
}

impl<'input> Jump<'input> {
    pub fn new_return(right: Option<Expression<'input>>) -> Self {
        let right = right.map(Box::new);
        Self::Return(right)
    }
}

impl<'input> From<Jump<'input>> for Expression<'input> {
    fn from(value: Jump<'input>) -> Self {
        Expression::Jump(value)
    }
}
impl fmt::Debug for Jump<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use Jump::*;

        match self {
            Return(expr) => f.debug_sexpr().name("return").items(expr.iter()).finish(),
        }
    }
}

#[derive(Clone, Copy, Hash, PartialEq, Eq, IntoPrimitive, TryFromPrimitive)]
#[repr(u8)]
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
            Self::Equals => f.write_str("=="),
            Self::NotEquals => f.write_str("!="),
            Self::Greater => f.write_str(">"),
            Self::GreaterEquals => f.write_str(">="),
            Self::Less => f.write_str("<"),
            Self::LessEquals => f.write_str("<="),
            Self::Add => f.write_str("+"),
            Self::Subtract => f.write_str("-"),
            Self::Multiply => f.write_str("*"),
            Self::Divide => f.write_str("/"),
        }
    }
}

#[derive(Clone, PartialEq, Eq)]
pub struct Binary<'input> {
    pub operator: BinaryOperator,
    pub left: Box<Expression<'input>>,
    pub right: Box<Expression<'input>>,
}

impl<'input> Binary<'input> {
    pub fn new(
        left: Expression<'input>,
        operator: BinaryOperator,
        right: Expression<'input>,
    ) -> Self {
        let left = Box::new(left);
        let right = Box::new(right);
        Self {
            operator,
            left,
            right,
        }
    }
}

impl<'input> From<Binary<'input>> for Expression<'input> {
    fn from(value: Binary<'input>) -> Self {
        Expression::Binary(value)
    }
}

impl fmt::Debug for Binary<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_sexpr()
            .item(&self.operator)
            .item(&self.left)
            .item(&self.right)
            .finish()
    }
}

#[derive(Clone, Copy, Hash, PartialEq, Eq, IntoPrimitive, TryFromPrimitive)]
#[repr(u8)]
pub enum UnaryOperator {
    // negation
    Negate,
    // logical inverse
    Not,
}

impl fmt::Debug for UnaryOperator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Negate => f.write_str("-"),
            Self::Not => f.write_str("!"),
        }
    }
}

#[derive(Clone, PartialEq, Eq)]
pub struct Unary<'input> {
    pub operator: UnaryOperator,
    pub right: Box<Expression<'input>>,
}

impl<'input> Unary<'input> {
    pub fn new(operator: UnaryOperator, right: Expression<'input>) -> Self {
        let right = Box::new(right);
        Self { operator, right }
    }
}

impl<'input> From<Unary<'input>> for Expression<'input> {
    fn from(value: Unary<'input>) -> Self {
        Expression::Unary(value)
    }
}

impl fmt::Debug for Unary<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_sexpr()
            .item(&self.operator)
            .item(&self.right)
            .finish()
    }
}

#[derive(Clone, PartialEq, Eq)]
pub struct Call<'input> {
    pub function: Box<Expression<'input>>,
    pub actual_parameters: Vec<Expression<'input>>,
}

impl<'input> Call<'input> {
    pub fn new(function: Expression<'input>, actual_parameters: Vec<Expression<'input>>) -> Self {
        let function = Box::new(function);
        Self {
            function,
            actual_parameters,
        }
    }
}

impl<'input> From<Call<'input>> for Expression<'input> {
    fn from(value: Call<'input>) -> Self {
        Expression::Call(value)
    }
}

impl fmt::Debug for Call<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_sexpr()
            .name("call")
            .item(&self.function)
            .items(&self.actual_parameters)
            .finish()
    }
}

#[derive(Clone, PartialEq, Eq)]
pub struct Block<'input> {
    pub statements: Vec<Statement<'input>>,
    pub expression: Option<Box<Expression<'input>>>,
}

impl<'input> Block<'input> {
    pub fn new(statements: Vec<Statement<'input>>, expression: Option<Expression<'input>>) -> Self {
        let expression = expression.map(Box::new);
        Self {
            statements,
            expression,
        }
    }
}

impl<'input> From<Block<'input>> for Expression<'input> {
    fn from(value: Block<'input>) -> Self {
        Expression::Block(value)
    }
}

impl fmt::Debug for Block<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut f = f.debug_sexpr();
        f.name("block").items(&self.statements);
        if let Some(expression) = &self.expression {
            f.item(&expression);
        } else {
            f.item(&());
        }
        f.finish()
    }
}

#[derive(Clone, PartialEq, Eq)]
pub struct If<'input> {
    pub then_branches: Vec<(Expression<'input>, Block<'input>)>,
    pub else_branch: Option<Block<'input>>,
}

impl<'input> If<'input> {
    pub fn new(
        then_branches: Vec<(Expression<'input>, Block<'input>)>,
        else_branch: Option<Block<'input>>,
    ) -> Self {
        Self {
            then_branches,
            else_branch,
        }
    }
}

impl<'input> From<If<'input>> for Expression<'input> {
    fn from(value: If<'input>) -> Self {
        Expression::If(value)
    }
}

impl fmt::Debug for If<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut f = f.debug_sexpr();
        for (condition, block) in &self.then_branches {
            f.name("if").item(condition).item(block);
        }
        if let Some(else_branch) = &self.else_branch {
            f.name("else").item(&else_branch);
        }
        f.finish()
    }
}

#[derive(Clone, PartialEq, Eq)]
pub struct Loop<'input> {
    pub body: Block<'input>,
}

impl<'input> Loop<'input> {
    pub fn new(body: Block<'input>) -> Self {
        Self { body }
    }
}

impl<'input> From<Loop<'input>> for Expression<'input> {
    fn from(value: Loop<'input>) -> Self {
        Expression::Loop(value)
    }
}

impl fmt::Debug for Loop<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_sexpr().name("loop").item(&self.body).finish()
    }
}
