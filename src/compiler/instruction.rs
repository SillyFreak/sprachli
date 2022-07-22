use crate::ast::{BinaryOperator, UnaryOperator};

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
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
    JumpPlaceholder,
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
            JumpPlaceholder => 0,
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
            JumpPlaceholder => 2,
        }
    }
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum InlineConstant {
    Unit,
    Bool(bool),
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum Offset {
    Forward(usize),
    Backward(usize),
}
