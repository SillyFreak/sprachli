use super::{InternalError, Result};
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

#[derive(Default, Debug, Clone, Hash, PartialEq, Eq)]
pub struct InstructionSequence {
    instructions: Vec<Instruction>,
}

#[derive(Debug)]
pub struct Placeholder<F>(usize, F);

impl InstructionSequence {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn push(&mut self, instruction: Instruction) {
        self.instructions.push(instruction);
    }

    pub fn push_placeholder<F>(&mut self, f: F) -> Placeholder<F>
    where
        F: FnOnce(Offset) -> Instruction,
    {
        let index = self.instructions.len();
        self.instructions.push(Instruction::Invalid);
        Placeholder(index, f)
    }

    pub fn offset_from(&self, index: usize) -> Offset {
        Offset::Forward(self.len() - index)
    }

    pub fn offset_to(&self, index: usize) -> Offset {
        Offset::Backward(index - self.len())
    }

    pub fn len(&self) -> usize {
        self.instructions.len()
    }

    #[inline]
    pub fn iter(&self) -> Iter<'_> {
        Iter::new(self)
    }
}

impl<F> Placeholder<F>
where
    F: FnOnce(Offset) -> Instruction,
{
    pub fn fill(self, instructions: &mut InstructionSequence) {
        let Placeholder(index, f) = self;
        let instruction = f(instructions.offset_from(index + 1));
        assert_eq!(instructions.instructions[index], Instruction::Invalid);
        instructions.instructions[index] = instruction;
    }
}

impl<'a> IntoIterator for &'a InstructionSequence {
    type Item = Instruction;
    type IntoIter = Iter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

#[derive(Debug, Clone)]
pub struct Iter<'a>(std::slice::Iter<'a, Instruction>);

impl<'a> Iter<'a> {
    fn new(instructions: &'a InstructionSequence) -> Self {
        Self(instructions.instructions.iter())
    }

    pub fn byte_offset(&self, offset: Offset) -> Result<Offset> {
        use InternalError::*;
        use Offset::*;

        let iter = self.0.clone();
        match offset {
            Forward(offset) => {
                if iter.len() < offset {
                    Err(InvalidJump)?;
                }
                let offset = iter.take(offset).copied().map(Instruction::encoded_len).sum();
                Ok(Forward(offset))
            }
            Backward(offset) => {
                let iter = iter.rev();
                if iter.len() < offset {
                    Err(InvalidJump)?;
                }
                let offset = iter.take(offset).copied().map(Instruction::encoded_len).sum();
                Ok(Backward(offset))
            }
        }
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

impl Iterator for Iter<'_> {
    type Item = Instruction;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().copied()
    }
}
