use std::fmt;

use super::{InternalError, Result};
use crate::bytecode::instruction::{BinaryOperator, Opcode, UnaryOperator};
use crate::bytecode::InstructionSequence;

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
    Invalid,
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
            LoadLocal(local) => write!(f, "LOAD {local}"),
            LoadNamed(index) => write!(f, "LOAD #{index}"),
            Call(arity) => write!(f, "CALL {arity}"),
            Jump(offset) => write!(f, "JUMP {offset:?}"),
            JumpIf(offset) => write!(f, "JUMP_IF {offset:?}"),
            Invalid => write!(f, "INVALID"),
        }
    }
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
            Invalid => 0,
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
            Invalid => 1,
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

impl<'a> IntoIterator for &'a InstructionSequence<'_> {
    type Item = Result<Instruction>;
    type IntoIter = InstructionIter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'b> InstructionSequence<'b> {
    #[inline]
    pub fn iter(&self) -> InstructionIter<'_> {
        InstructionIter::new(self)
    }
}

#[derive(Debug, Clone)]
pub struct InstructionIter<'b>(usize, std::slice::Iter<'b, u8>);

impl<'b> InstructionIter<'b> {
    fn new(instructions: &InstructionSequence<'b>) -> Self {
        Self(0, instructions.get().iter())
    }

    pub fn offset(&self) -> usize {
        self.0
    }

    pub fn with_offset(self) -> OffsetInstructionIter<'b> {
        OffsetInstructionIter::new(self)
    }

    fn advance(&mut self) -> Option<u8> {
        let item = self.1.next().copied()?;
        self.0 += 1;
        Some(item)
    }

    fn opcode(&mut self) -> Option<Result<Opcode>> {
        self.advance().map(|opcode| -> Result<Opcode> {
            let opcode = opcode
                .try_into()
                .map_err(|_| InternalError::InvalidBytecode)?;

            Ok(opcode)
        })
    }

    fn parameter(&mut self) -> Result<u8> {
        let parameter = self.advance().ok_or(InternalError::InvalidBytecode)?;
        Ok(parameter)
    }

    pub fn jump(&mut self, offset: Offset) -> Result<()> {
        use InternalError::*;
        use Offset::*;

        match offset {
            Forward(offset) => {
                self.0 += offset;
                if offset > 0 {
                    self.1.nth(offset - 1).ok_or(InvalidJump)?;
                }
            }
            Backward(offset) => {
                self.0 -= offset;
                if offset > 0 {
                    self.1.nth_back(offset - 1).ok_or(InvalidJump)?;
                }
            }
        }

        Ok(())
    }
}

impl Iterator for InstructionIter<'_> {
    type Item = Result<Instruction>;

    fn next(&mut self) -> Option<Self::Item> {
        use InlineConstant as Inl;
        use Instruction as In;
        use Opcode as Op;

        self.opcode().map(|opcode| {
            opcode.and_then(|opcode| {
                let ins = match opcode {
                    Op::Constant => {
                        let constant = self.parameter()?;
                        In::Constant(constant as usize)
                    }
                    Op::Unit => In::InlineConstant(Inl::Unit),
                    Op::True => In::InlineConstant(Inl::Bool(true)),
                    Op::False => In::InlineConstant(Inl::Bool(false)),
                    Op::Pop => In::Pop,
                    Op::Unary => {
                        let op = self.parameter()?;
                        let op = op.try_into().map_err(|_| InternalError::InvalidBytecode)?;
                        In::Unary(op)
                    }
                    Op::Binary => {
                        let op = self.parameter()?;
                        let op = op.try_into().map_err(|_| InternalError::InvalidBytecode)?;
                        In::Binary(op)
                    }
                    Op::LoadLocal => {
                        let local = self.parameter()?;
                        In::LoadLocal(local as usize)
                    }
                    Op::LoadNamed => {
                        let constant = self.parameter()?;
                        In::LoadNamed(constant as usize)
                    }
                    Op::Call => {
                        let arity = self.parameter()?;
                        In::Call(arity as usize)
                    }
                    Op::JumpForward => {
                        let offset = self.parameter()?;
                        In::Jump(Offset::Forward(offset as usize))
                    }
                    Op::JumpBackward => {
                        let offset = self.parameter()?;
                        In::Jump(Offset::Backward(offset as usize))
                    }
                    Op::JumpForwardIf => {
                        let offset = self.parameter()?;
                        In::JumpIf(Offset::Forward(offset as usize))
                    }
                    Op::JumpBackwardIf => {
                        let offset = self.parameter()?;
                        In::JumpIf(Offset::Backward(offset as usize))
                    }
                };

                Ok(ins)
            })
        })
    }
}

#[derive(Debug, Clone)]
pub struct OffsetInstructionIter<'b>(InstructionIter<'b>);

impl<'b> OffsetInstructionIter<'b> {
    fn new(iter: InstructionIter<'b>) -> Self {
        Self(iter)
    }

    pub fn offset(&self) -> usize {
        self.0.offset()
    }

    pub fn jump(&mut self, offset: Offset) -> Result<()> {
        self.0.jump(offset)
    }
}

impl Iterator for OffsetInstructionIter<'_> {
    type Item = (usize, Result<Instruction>);

    fn next(&mut self) -> Option<Self::Item> {
        let offset = self.offset();
        let ins = self.0.next()?;
        Some((offset, ins))
    }
}
