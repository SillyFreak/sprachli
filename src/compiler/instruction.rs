use std::fmt;

use super::Module;
use crate::bytecode::instruction::{Instruction, Offset};
use crate::fmt::ModuleFormat;

#[derive(Clone, Copy, Hash, PartialEq, Eq)]
pub enum InstructionItem {
    Real(Instruction),
    Placeholder(PlaceholderKind),
}

impl InstructionItem {
    pub fn real(self) -> Option<Instruction> {
        use InstructionItem::*;

        match self {
            Real(ins) => Some(ins),
            _ => None,
        }
    }

    pub fn stack_effect(self) -> Option<isize> {
        use InstructionItem::*;

        match self {
            Real(ins) => ins.stack_effect(),
            Placeholder(pl) => Some(pl.stack_effect()),
        }
    }

    pub fn encoded_len(self) -> usize {
        use InstructionItem::*;

        match self {
            Real(ins) => ins.encoded_len(),
            Placeholder(pl) => pl.encoded_len(),
        }
    }

    pub(crate) fn fmt_with<M: ModuleFormat>(
        &self,
        f: &mut fmt::Formatter<'_>,
        module: Option<&M>,
    ) -> fmt::Result {
        use InstructionItem::*;

        match self {
            Real(ins) => ins.fmt_with(f, module),
            Placeholder(kind) => kind.fmt_with(f, module),
        }
    }
}

impl From<Instruction> for InstructionItem {
    fn from(ins: Instruction) -> Self {
        InstructionItem::Real(ins)
    }
}

impl fmt::Debug for InstructionItem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.fmt_with::<Module>(f, None)
    }
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum PlaceholderKind {
    Jump,
    JumpIf,
}

impl PlaceholderKind {
    pub fn stack_effect(self) -> isize {
        use PlaceholderKind::*;

        match self {
            Jump => 0,
            JumpIf => -1,
        }
    }

    pub fn encoded_len(self) -> usize {
        use PlaceholderKind::*;

        match self {
            Jump | JumpIf => 2,
        }
    }

    pub fn jump(self, offset: Offset) -> InstructionItem {
        use PlaceholderKind::*;

        match self {
            Jump => Instruction::Jump(offset).into(),
            JumpIf => Instruction::JumpIf(offset).into(),
        }
    }

    pub(crate) fn fmt_with<M: ModuleFormat>(
        &self,
        f: &mut fmt::Formatter<'_>,
        _module: Option<&M>,
    ) -> fmt::Result {
        use PlaceholderKind::*;

        match self {
            Jump => write!(f, "JUMP PLACEHOLDER"),
            JumpIf => write!(f, "JUMP_IF PLACEHOLDER"),
        }
    }
}
