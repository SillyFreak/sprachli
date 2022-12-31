use std::fmt;

use num_enum::{IntoPrimitive, TryFromPrimitive};
use sprachli_fmt::FormatterExt;

use super::{FnTrunk, Statement};

#[derive(Clone, PartialEq, Eq)]
pub enum Expression<'input> {
    Number(&'input str),
    Bool(bool),
    String(&'input str),
    Identifier(&'input str),
    Binary(Binary<'input>),
    Unary(Unary<'input>),
    Call(Call<'input>),
    Block(Block<'input>),
    Fn(Fn<'input>),
    If(If<'input>),
    Loop(Loop<'input>),
}

impl Expression<'_> {
    pub(super) fn is_simple(&self) -> bool {
        use Expression::*;

        matches!(self, Number(_) | Bool(_) | String(_) | Identifier(_))
    }
}

impl fmt::Debug for Expression<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use Expression::*;

        match self {
            Number(value) => fmt::Display::fmt(value, f),
            Bool(value) => fmt::Display::fmt(value, f),
            String(value) => fmt::Display::fmt(value, f),
            Identifier(name) => f.write_str(name),
            Binary(expr) => expr.fmt(f),
            Unary(expr) => expr.fmt(f),
            Call(expr) => expr.fmt(f),
            Block(expr) => expr.fmt(f),
            Fn(expr) => expr.fmt(f),
            If(expr) => expr.fmt(f),
            Loop(expr) => expr.fmt(f),
        }
    }
}

#[derive(Clone, Copy, Hash, PartialEq, Eq, IntoPrimitive, TryFromPrimitive)]
#[repr(u8)]
pub enum BinaryOperator {
    // multiplicative
    Multiply,
    Divide,
    Modulo,
    // additive
    Add,
    Subtract,
    // shift
    RightShift,
    LeftShift,
    // bitwise
    BitAnd,
    BitXor,
    BitOr,
    // comparison
    Equals,
    NotEquals,
    Greater,
    GreaterEquals,
    Less,
    LessEquals,
}

impl fmt::Debug for BinaryOperator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use BinaryOperator::*;

        match self {
            Multiply => f.write_str("*"),
            Divide => f.write_str("/"),
            Modulo => f.write_str("%"),
            Add => f.write_str("+"),
            Subtract => f.write_str("-"),
            RightShift => f.write_str(">>"),
            LeftShift => f.write_str("<<"),
            BitAnd => f.write_str("&"),
            BitXor => f.write_str("^"),
            BitOr => f.write_str("|"),
            Equals => f.write_str("=="),
            NotEquals => f.write_str("!="),
            Greater => f.write_str(">"),
            GreaterEquals => f.write_str(">="),
            Less => f.write_str("<"),
            LessEquals => f.write_str("<="),
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
        let compact = self.left.is_simple() && self.right.is_simple();
        f.debug_sexpr_compact(compact)
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
        use UnaryOperator::*;

        match self {
            Negate => f.write_str("-"),
            Not => f.write_str("!"),
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
        let compact = self.right.is_simple();
        f.debug_sexpr_compact(compact)
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
        let compact =
            self.function.is_simple() && self.actual_parameters.iter().all(Expression::is_simple);
        f.debug_sexpr_compact(compact)
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
        let compact = self.statements.iter().all(Statement::is_simple)
            && self
                .expression
                .as_deref()
                .map_or(true, Expression::is_simple);
        let mut f = f.debug_sexpr_compact(compact);
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
pub struct Fn<'input> {
    pub trunk: FnTrunk<'input>,
}

impl<'input> Fn<'input> {
    pub fn new(trunk: FnTrunk<'input>) -> Self {
        Self { trunk }
    }
}

impl<'input> From<Fn<'input>> for Expression<'input> {
    fn from(value: Fn<'input>) -> Self {
        Expression::Fn(value)
    }
}

impl fmt::Debug for Fn<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut f = f.debug_sexpr();
        f.name("fn");
        self.trunk.fmt(&mut f);
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
