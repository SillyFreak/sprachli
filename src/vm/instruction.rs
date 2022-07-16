use crate::ast::{BinaryOperator, UnaryOperator};

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum Instruction {
    Constant(usize),
    InlineConstant(InlineConstant),
    Pop,
    Unary(UnaryOperator),
    Binary(BinaryOperator),
    Load(usize),
    Call(usize),
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

impl InstructionSequence {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn push(&mut self, instruction: Instruction) {
        self.instructions.push(instruction);
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
        Self(instructions.instructions.iter())
    }
}

impl Iterator for Iter<'_> {
    type Item = Instruction;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().copied()
    }
}
