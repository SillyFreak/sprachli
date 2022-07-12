use crate::ast::{BinaryOperator, UnaryOperator};

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum Instruction {
    Constant(usize),
    InlineConstant(InlineConstant),
    Unary(UnaryOperator),
    Binary(BinaryOperator),
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum InlineConstant {
    Unit,
    Bool(bool),
}

#[derive(Default, Debug, Clone, Hash, PartialEq, Eq)]
pub struct InstructionSequence(Vec<Instruction>);

impl InstructionSequence {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn push(&mut self, instruction: Instruction) {
        self.0.push(instruction);
    }

    #[inline]
    pub fn iter(&self) -> Iter<'_> {
        Iter::new(self)
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
        Self(instructions.0.iter())
    }
}

impl Iterator for Iter<'_> {
    type Item = Instruction;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().copied()
    }
}
