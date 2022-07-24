use std::fmt;

use num_enum::{IntoPrimitive, TryFromPrimitive};

use crate::ast;

pub use ast::{BinaryOperator, UnaryOperator};

#[derive(Debug, Clone, Copy, Eq, PartialEq, IntoPrimitive, TryFromPrimitive)]
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

#[derive(Clone, Copy, Hash, PartialEq, Eq)]
pub enum Instruction {
    Constant(usize),
    InlineConstant(InlineConstant),
    Pop,
    Unary(UnaryOperator),
    Binary(BinaryOperator),
    LoadLocal(usize),
    LoadNamed(usize),
    Call(usize),
    Jump(Offset),
    JumpIf(Offset),
}

impl Instruction {
    pub fn stack_effect(self) -> isize {
        use Instruction::*;

        match self {
            Constant(_) => 1,
            InlineConstant(_) => 1,
            Pop => -1,
            Unary(_) => 0,
            Binary(_) => -1,
            LoadLocal(_) => 1,
            LoadNamed(_) => 1,
            Call(arity) => -isize::try_from(arity).expect("illegal arity"),
            Jump(_) => 0,
            JumpIf(_) => -1,
        }
    }

    pub fn encoded_len(self) -> usize {
        use Instruction::*;

        match self {
            Constant(_) => 2,
            InlineConstant(_) => 2,
            Pop => 1,
            Unary(_) => 2,
            Binary(_) => 2,
            LoadLocal(_) => 2,
            LoadNamed(_) => 2,
            Call(_) => 2,
            Jump(_) => 2,
            JumpIf(_) => 2,
        }
    }
}

impl fmt::Debug for Instruction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use Instruction::*;

        match self {
            Constant(index) => write!(f, "CONST #{index}"),
            InlineConstant(value) => write!(f, "CONST {value:?}"),
            Pop => write!(f, "POP"),
            Unary(op) => write!(f, "UNARY {op:?}"),
            Binary(op) => write!(f, "BINARY {op:?}"),
            LoadLocal(local) => write!(f, "LOAD _{local}"),
            LoadNamed(index) => write!(f, "LOAD #{index}"),
            Call(arity) => write!(f, "CALL {arity}"),
            Jump(offset) => write!(f, "JUMP {offset:?}"),
            JumpIf(offset) => write!(f, "JUMP_IF {offset:?}"),
        }
    }
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum InlineConstant {
    Unit,
    Bool(bool),
}

#[derive(Clone, Copy, Hash, PartialEq, Eq)]
pub enum Offset {
    Forward(usize),
    Backward(usize),
}

impl fmt::Debug for Offset {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use Offset::*;

        match self {
            Forward(offset) => write!(f, "+{offset}"),
            Backward(offset) => write!(f, "-{offset}"),
        }
    }
}
