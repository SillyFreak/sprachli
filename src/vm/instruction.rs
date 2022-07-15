use crate::ast::{BinaryOperator, UnaryOperator};

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum Instruction {
    Constant(usize),
    InlineConstant(InlineConstant),
    Unary(UnaryOperator),
    Binary(BinaryOperator),
    Load(usize),
    Call(usize),
}

impl Instruction {
    fn stack_deltas(self) -> (isize, isize) {
        use Instruction::*;

        match self {
            Constant(_) => (1, 1),
            InlineConstant(_) => (1, 1),
            Unary(_) => (0, 0),
            Binary(_) => (0, -1),
            Load(_) => (1, 1),
            Call(arity) => (0, -isize::try_from(arity).expect("illegal arity")),
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
    stack_size: usize,
    instructions: Vec<Instruction>,
}

impl InstructionSequence {
    pub fn new(stack_size: usize, instructions: Vec<Instruction>) -> Self {
        Self {
            stack_size,
            instructions,
        }
    }

    pub fn stack_size(&self) -> usize {
        self.stack_size
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

#[derive(Default, Debug, Clone)]
pub struct InstructionSequenceBuilder {
    stack_size: usize,
    current_stack_size: usize,
    instructions: Vec<Instruction>,
}

impl InstructionSequenceBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn into_instruction_sequence(self) -> InstructionSequence {
        InstructionSequence::new(self.stack_size, self.instructions)
    }

    pub fn push(&mut self, instruction: Instruction) {
        let (size, delta) = instruction.stack_deltas();

        let max_size = self.current_stack_size.wrapping_add(size as usize);
        if max_size > self.stack_size {
            self.stack_size = max_size;
        }

        let new_size = self.current_stack_size.wrapping_add(delta as usize);
        self.current_stack_size = new_size;

        self.instructions.push(instruction);
    }
}
