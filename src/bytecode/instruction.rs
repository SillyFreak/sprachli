use num_enum::{IntoPrimitive, TryFromPrimitive};

use crate::ast;

pub use ast::{BinaryOperator, UnaryOperator};

#[derive(Debug, Eq, PartialEq, IntoPrimitive, TryFromPrimitive)]
#[repr(u8)]
pub enum Opcode {
    // zero is reserved for intentionally invalid opcodes,
    // but all nontaken opcodes are of course invalid as well
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
