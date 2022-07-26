use std::fmt;

use num_enum::{IntoPrimitive, TryFromPrimitive};
use sprachli_fmt::{FormatterExt, ModuleFormat};
use sprachli_parser::ast;

use super::Module;

pub use ast::{BinaryOperator, UnaryOperator};

#[derive(Debug, Clone, Copy, Eq, PartialEq, IntoPrimitive, TryFromPrimitive)]
#[repr(u8)]
pub enum Opcode {
    // constants
    Constant,
    Unit,
    True,
    False,

    // calculations
    Unary,
    Binary,

    // memory
    LoadLocal,
    StoreLocal,
    LoadNamed,
    StoreNamed,
    LoadPositionalField,
    StorePositionalField,
    LoadNamedField,
    StoreNamedField,

    // stack management
    Pop,
    PopScope,
    Call,
    Return,

    // jumps
    JumpForward,
    JumpBackward,
    JumpForwardIf,
    JumpBackwardIf,
}

#[derive(Clone, Copy, Hash, PartialEq, Eq)]
pub enum Instruction {
    // constants
    Constant(usize),
    InlineConstant(InlineConstant),

    // calculations
    Unary(UnaryOperator),
    Binary(BinaryOperator),

    // memory
    LoadLocal(usize),
    StoreLocal(usize),
    LoadNamed(usize),
    StoreNamed(usize),
    LoadPositionalField(usize),
    StorePositionalField(usize),
    LoadNamedField(usize),
    StoreNamedField(usize),

    // stack management
    Pop,
    PopScope(usize),

    // functions
    Call(usize),
    Return,

    // jumps
    Jump(Offset),
    JumpIf(Offset),
}

impl Instruction {
    pub fn stack_effect(self) -> Option<isize> {
        use Instruction::*;

        let effect = match self {
            Constant(_) => 1,
            InlineConstant(_) => 1,
            Unary(_) => 0,
            Binary(_) => -1,
            LoadLocal(_) => 1,
            StoreLocal(_) => -1,
            LoadNamed(_) => 1,
            StoreNamed(_) => -1,
            LoadPositionalField(_) => 1,
            StorePositionalField(_) => -1,
            LoadNamedField(_) => 1,
            StoreNamedField(_) => -1,
            Pop => -1,
            PopScope(_depth) => return None,
            Call(arity) => -isize::try_from(arity).expect("illegal arity"),
            // Return diverges, but it (conceptually) pops one value off the stack before the function ends
            Return => -1,
            Jump(_) => 0,
            JumpIf(_) => -1,
        };

        Some(effect)
    }

    pub fn encoded_len(self) -> usize {
        use Instruction::*;

        match self {
            Constant(_) => 2,
            InlineConstant(_) => 1,
            Unary(_) => 2,
            Binary(_) => 2,
            LoadLocal(_) => 2,
            StoreLocal(_) => 2,
            LoadNamed(_) => 2,
            StoreNamed(_) => 2,
            LoadPositionalField(_) => 2,
            StorePositionalField(_) => 2,
            LoadNamedField(_) => 2,
            StoreNamedField(_) => 2,
            Pop => 1,
            PopScope(_) => 2,
            Call(_) => 2,
            Return => 1,
            Jump(_) => 2,
            JumpIf(_) => 2,
        }
    }

    pub fn fmt_with<M: ModuleFormat>(
        &self,
        f: &mut fmt::Formatter<'_>,
        module: Option<&M>,
    ) -> fmt::Result {
        use Instruction::*;

        match self {
            Constant(index) => {
                if let Some(module) = module {
                    write!(f, "CONST #{index:<8} -- ")?;
                    f.fmt_constant(module, *index)?;
                } else {
                    write!(f, "CONST #{index}")?;
                }
                Ok(())
            }
            InlineConstant(value) => write!(f, "CONST {value:?}"),
            Pop => write!(f, "POP"),
            Unary(op) => write!(f, "UNARY {op:?}"),
            Binary(op) => write!(f, "BINARY {op:?}"),
            LoadLocal(local) => write!(f, "LOAD _{local}"),
            StoreLocal(local) => write!(f, "STORE _{local}"),
            LoadNamed(index) => {
                if let Some(module) = module {
                    write!(f, "LOAD #{index:<9} -- ")?;
                    f.fmt_constant_ident(module, *index)?;
                } else {
                    write!(f, "LOAD #{index}")?;
                }
                Ok(())
            }
            StoreNamed(index) => {
                if let Some(module) = module {
                    write!(f, "STORE #{index:<8} -- ")?;
                    f.fmt_constant_ident(module, *index)?;
                } else {
                    write!(f, "STORE #{index}")?;
                }
                Ok(())
            }
            LoadPositionalField(index) => write!(f, "LOAD FIELD _{index}"),
            StorePositionalField(index) => write!(f, "STORE FIELD _{index}"),
            LoadNamedField(index) => {
                if let Some(module) = module {
                    write!(f, "LOAD FIELD #{index:<3} -- ")?;
                    f.fmt_constant_ident(module, *index)?;
                } else {
                    write!(f, "LOAD FIELD #{index}")?;
                }
                Ok(())
            }
            StoreNamedField(index) => {
                if let Some(module) = module {
                    write!(f, "STORE FIELD #{index:<2} -- ")?;
                    f.fmt_constant_ident(module, *index)?;
                } else {
                    write!(f, "STORE FIELD #{index}")?;
                }
                Ok(())
            }
            PopScope(depth) => write!(f, "POP SCOPE {depth}"),
            Call(arity) => write!(f, "CALL {arity}"),
            Return => write!(f, "RETURN"),
            Jump(offset) => write!(f, "JUMP {offset:?}"),
            JumpIf(offset) => write!(f, "JUMP_IF {offset:?}"),
        }
    }
}

impl fmt::Debug for Instruction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.fmt_with::<Module>(f, None)
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
