use std::fmt;

use bigdecimal::BigDecimal;

use crate::fmt::FormatterExt;
use super::Statement;

#[derive(Clone, PartialEq, Eq)]
pub enum Expression {
    Number(BigDecimal),
    Identifier(String),
    Binary(Binary),
    Unary(Unary),
    Call(Call),
    Block(Block),
    If(If),
}

impl fmt::Debug for Expression {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Number(value) => write!(f, "{value:?}"),
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
    pub left: Box<Expression>,
    pub right: Box<Expression>,
}

impl fmt::Debug for Binary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_prefixed().item(&self.operator).item(&self.left).item(&self.right).finish()
    }
}

impl Binary {
    pub fn new(
        left: Expression,
        operator: BinaryOperator,
        right: Expression,
    ) -> Self {
        let left = Box::new(left);
        let right = Box::new(right);
        Self { operator, left, right }
    }

    pub fn new_expression(
        left: Expression,
        operator: BinaryOperator,
        right: Expression,
    ) -> Expression {
        Expression::Binary(Self::new(left, operator, right))
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
    pub right: Box<Expression>,
}

impl fmt::Debug for Unary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_prefixed().item(&self.operator).item(&self.right).finish()
    }
}

impl Unary {
    pub fn new(
        operator: UnaryOperator,
        right: Expression,
    ) -> Self {
        let right = Box::new(right);
        Self { operator, right }
    }

    pub fn new_expression(
        operator: UnaryOperator,
        right: Expression,
    ) -> Expression {
        Expression::Unary(Self::new(operator, right))
    }
}

#[derive(Clone, PartialEq, Eq)]
pub struct Call {
    pub function: Box<Expression>,
    pub actual_parameters: Vec<Expression>,
}

impl fmt::Debug for Call {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut f = f.debug_prefixed();
        f.name("call").item(&self.function).items(&self.actual_parameters).finish()
    }
}

impl Call {
    pub fn new(
        function: Expression,
        actual_parameters: Vec<Expression>,
    ) -> Self {
        let function = Box::new(function);
        Self { function, actual_parameters }
    }

    pub fn new_expression(
        function: Expression,
        actual_parameters: Vec<Expression>,
    ) -> Expression {
        Expression::Call(Self::new(function, actual_parameters))
    }
}

#[derive(Clone, PartialEq, Eq)]
pub struct Block {
    pub statements: Vec<Statement>,
    pub expression: Option<Box<Expression>>,
}

impl fmt::Debug for Block {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut f = f.debug_prefixed();
        f.name("block").items(&self.statements);
        if let Some(expression) = &self.expression {
            f.item(&expression);
        } else {
            f.item(&());
        }
        f.finish()
    }
}

impl Block {
    pub fn new(
        statements: Vec<Statement>,
        expression: Option<Expression>,
    ) -> Self {
		let expression = expression.map(Box::new);
        Self { statements, expression }
    }

    pub fn new_expression(
        statements: Vec<Statement>,
        expression: Option<Expression>,
    ) -> Expression {
        Expression::Block(Self::new(statements, expression))
    }
}

#[derive(Clone, PartialEq, Eq)]
pub struct If {
    pub then_branches: Vec<(Expression, Block)>,
    pub else_branch: Option<Block>,
}

impl fmt::Debug for If {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut f = f.debug_prefixed();
        for (condition, block) in &self.then_branches {
            f.name("if").item(condition).item(block);
        }
        if let Some(else_branch) = &self.else_branch {
            f.name("else").item(&else_branch);
        }
        f.finish()
    }
}

impl If {
    pub fn new(
        then_branches: Vec<(Expression, Block)>,
        else_branch: Option<Block>,
    ) -> Self {
        Self { then_branches, else_branch }
    }

    pub fn new_expression(
        then_branches: Vec<(Expression, Block)>,
        else_branch: Option<Block>,
    ) -> Expression {
        Expression::If(Self::new(then_branches, else_branch))
    }
}
