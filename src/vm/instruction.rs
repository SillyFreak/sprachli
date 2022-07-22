use super::{InternalError, Result};
use crate::ast::{BinaryOperator, UnaryOperator};
use crate::bytecode::instruction::{self, Opcode};
use crate::bytecode::InstructionSequence;

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
    Invalid,
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

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum Offset {
    Forward(usize),
    Backward(usize),
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
pub struct InstructionIter<'b>(std::slice::Iter<'b, u8>);

impl<'b> InstructionIter<'b> {
    fn new(instructions: &InstructionSequence<'b>) -> Self {
        Self(instructions.get().iter())
    }

    fn opcode(&mut self) -> Option<Result<Opcode>> {
        self.0.next().copied().map(|opcode| -> Result<Opcode> {
            let opcode = opcode
                .try_into()
                .map_err(|_| InternalError::InvalidBytecode)?;

            Ok(opcode)
        })
    }

    fn parameter(&mut self) -> Result<u8> {
        let parameter = *self.0.next().ok_or(InternalError::InvalidBytecode)?;
        Ok(parameter)
    }

    pub fn jump(&mut self, offset: Offset) -> Result<()> {
        use InternalError::*;
        use Offset::*;

        match offset {
            Forward(offset) => {
                if offset > 0 {
                    self.0.nth(offset - 1).ok_or(InvalidJump)?;
                }
            }
            Backward(offset) => {
                if offset > 0 {
                    self.0.nth_back(offset - 1).ok_or(InvalidJump)?;
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
                        let op = instruction::UnaryOperator::try_from(op)
                            .map_err(|_| InternalError::InvalidBytecode)?;
                        In::Unary(op.into())
                    }
                    Op::Binary => {
                        let op = self.parameter()?;
                        let op = instruction::BinaryOperator::try_from(op)
                            .map_err(|_| InternalError::InvalidBytecode)?;
                        In::Binary(op.into())
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
