use crate::ast::{BinaryOperator, UnaryOperator};
use super::{InternalError, Result};

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum Instruction {
    Constant(usize),
    InlineConstant(InlineConstant),
    Pop,
    Unary(UnaryOperator),
    Binary(BinaryOperator),
    Load(usize),
    Call(usize),
    Jump(isize),
    JumpIf(isize),
    Invalid,
}

impl Instruction {
    fn stack_effect(self) -> isize {
        use Instruction::*;

        match self {
            Constant(_) => 1,
            InlineConstant(_) => 1,
            Pop => -1,
            Unary(_) => 0,
            Binary(_) => -1,
            Load(_) => 1,
            Call(arity) => -isize::try_from(arity).expect("illegal arity"),
            Jump(_) => 0,
            JumpIf(_) => -1,
            Invalid => 0,
        }
    }
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum InlineConstant {
    Unit,
    Bool(bool),
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
        F: FnOnce(isize) -> Instruction,
    {
        let index = self.instructions.len();
        self.instructions.push(Instruction::Invalid);
        Placeholder(index, f)
    }

    pub fn offset_from(&self, index: usize) -> isize {
        (self.len() - index) as isize
    }

    pub fn offset_to(&self, index: usize) -> isize {
        (index - self.len()) as isize
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
    F: FnOnce(isize) -> Instruction,
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

    pub fn jump(&mut self, offset: isize) -> Result<()> {
        use InternalError::*;

        if offset > 0 {
            let offset = offset as usize;
            self.0.nth(offset - 1).ok_or(InvalidJump)?;
        } else if offset < 0 {
            let offset = -offset as usize;
            self.0.nth_back(offset - 1).ok_or(InvalidJump)?;
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
