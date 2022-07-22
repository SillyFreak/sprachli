use num_enum::{IntoPrimitive, TryFromPrimitive};

use crate::ast;

#[derive(Debug, Eq, PartialEq, IntoPrimitive, TryFromPrimitive)]
#[repr(u8)]
pub enum Opcode {
    Constant = 1,
    // InlineConstant
    Unit,
    True,
    False,

    Pop,
    Unary,
    Binary,
    LoadLocal,
    LoadNamed,
    Call,
    // Jump & JumpIf
    JumpForward,
    JumpBackward,
    JumpForwardIf,
    JumpBackwardIf,
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

impl From<BinaryOperator> for ast::BinaryOperator {
    fn from(op: BinaryOperator) -> Self {
        use ast::BinaryOperator as Ast;
        use BinaryOperator as Op;

        match op {
            Op::Equals => Ast::Equals,
            Op::NotEquals => Ast::NotEquals,
            Op::Greater => Ast::Greater,
            Op::GreaterEquals => Ast::GreaterEquals,
            Op::Less => Ast::Less,
            Op::LessEquals => Ast::LessEquals,
            Op::Add => Ast::Add,
            Op::Subtract => Ast::Subtract,
            Op::Multiply => Ast::Multiply,
            Op::Divide => Ast::Divide,
        }
    }
}

impl From<ast::BinaryOperator> for BinaryOperator {
    fn from(op: ast::BinaryOperator) -> Self {
        use ast::BinaryOperator as Ast;
        use BinaryOperator as Op;

        match op {
            Ast::Equals => Op::Equals,
            Ast::NotEquals => Op::NotEquals,
            Ast::Greater => Op::Greater,
            Ast::GreaterEquals => Op::GreaterEquals,
            Ast::Less => Op::Less,
            Ast::LessEquals => Op::LessEquals,
            Ast::Add => Op::Add,
            Ast::Subtract => Op::Subtract,
            Ast::Multiply => Op::Multiply,
            Ast::Divide => Op::Divide,
        }
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

impl From<UnaryOperator> for ast::UnaryOperator {
    fn from(op: UnaryOperator) -> Self {
        use ast::UnaryOperator as Ast;
        use UnaryOperator as Op;

        match op {
            Op::Negate => Ast::Negate,
            Op::Not => Ast::Not,
        }
    }
}

impl From<ast::UnaryOperator> for UnaryOperator {
    fn from(op: ast::UnaryOperator) -> Self {
        use ast::UnaryOperator as Ast;
        use UnaryOperator as Op;

        match op {
            Ast::Negate => Op::Negate,
            Ast::Not => Op::Not,
        }
    }
}
