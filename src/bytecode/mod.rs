//! Sprachli bytecode format
//!
//! The bytecode encompasses all values derived from a source program that are
//! known at compile time and required during runtime. In particular, this
//! includes identifiers, number and string literals, and functions defined in
//! the code.

use std::fmt;

mod error;
pub mod instruction;
pub mod parser;

use std::collections::HashMap;

use bigdecimal::BigDecimal;
use itertools::Itertools;
use num_enum::{IntoPrimitive, TryFromPrimitive};

use crate::fmt::{FormatterExt, ModuleFormat};
use instruction::{InlineConstant, Instruction, Offset, Opcode};

pub use error::*;
pub use parser::parse_bytecode;

pub type Number = BigDecimal;

#[derive(Debug, Clone)]
pub struct Bytecode<B>(B)
where
    B: AsRef<[u8]>;

#[derive(Clone)]
pub struct Module<'b> {
    constants: Vec<Constant<'b>>,
    globals: HashMap<&'b str, usize>,
}

impl<'b> Module<'b> {
    pub fn new(constants: Vec<Constant<'b>>, globals: HashMap<&'b str, usize>) -> Self {
        Self { constants, globals }
    }

    pub fn constants(&self) -> &Vec<Constant<'b>> {
        &self.constants
    }

    pub fn constant(&self, index: usize) -> Option<&Constant<'b>> {
        self.constants.get(index)
    }

    pub fn globals(&self) -> &HashMap<&'b str, usize> {
        &self.globals
    }

    pub fn global(&self, name: &str) -> Option<&Constant<'b>> {
        let index = *self.globals.get(name)?;
        self.constant(index)
    }
}

impl<'b> ModuleFormat for Module<'b> {
    type Constant = Constant<'b>;

    fn constant(&self, index: usize) -> Option<(&Self::Constant, Option<&str>)> {
        let constant = self.constants.get(index)?;
        let string = match *constant {
            Constant::String(value) => Some(value),
            _ => None,
        };
        Some((constant, string))
    }
}

impl fmt::Debug for Module<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if f.alternate() {
            f.write_str("Module {\n")?;
            f.write_str("    constants: [\n")?;
            for (i, constant) in self.constants.iter().enumerate() {
                write!(f, "    {i:5}: ")?;
                constant.fmt_with(f, Some(self))?;
                f.write_str("\n")?;
            }
            f.write_str("    ],\n")?;
            f.write_str("    globals: {\n")?;
            for (name, index) in &self.globals {
                f.write_str("        ")?;
                f.write_str(name)?;
                write!(f, ": {index:<0$} -- ", 9usize.saturating_sub(name.len()))?;
                f.fmt_constant(self, *index)?;
                f.write_str("\n")?;
            }
            f.write_str("    },\n")?;
            f.write_str("}")?;
            Ok(())
        } else {
            f.debug_struct("Module")
                .field("constants", &self.constants)
                .field("globals", &self.globals)
                .finish()
        }
    }
}

#[derive(Debug, Eq, PartialEq, IntoPrimitive, TryFromPrimitive)]
#[repr(u8)]
pub enum ConstantType {
    Number,
    String,
    Function,
}

#[derive(Clone)]
pub enum Constant<'b> {
    Number(Number),
    String(&'b str),
    Function(Function<'b>),
}

impl<'b> Constant<'b> {
    pub(crate) fn fmt_with<M: ModuleFormat>(
        &self,
        f: &mut fmt::Formatter<'_>,
        module: Option<&M>,
    ) -> fmt::Result {
        use fmt::Debug;
        use Constant::*;

        match self {
            Number(value) => fmt::Display::fmt(value, f),
            String(value) => value.fmt(f),
            Function(value) => value.fmt_with(f, module),
        }
    }
}

impl fmt::Debug for Constant<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.fmt_with::<Module>(f, None)
    }
}

#[derive(Clone)]
pub struct Function<'b> {
    arity: usize,
    body: InstructionSequence<'b>,
}

impl<'b> Function<'b> {
    pub fn new(arity: usize, body: InstructionSequence<'b>) -> Self {
        Self { arity, body }
    }

    pub fn arity(&self) -> usize {
        self.arity
    }

    pub fn body(&self) -> &InstructionSequence {
        &self.body
    }

    pub(crate) fn fmt_with<M: ModuleFormat>(
        &self,
        f: &mut fmt::Formatter<'_>,
        module: Option<&M>,
    ) -> fmt::Result {
        f.write_str("fn (")?;
        for i in (0..self.arity).map(Some).intersperse(None) {
            match i {
                Some(i) => write!(f, "_{}", i)?,
                None => f.write_str(", ")?,
            }
        }

        if f.alternate() {
            f.write_str(") {\n")?;
            self.body.fmt_with(f, module)?;
            f.write_str("\n           }")?;
        } else {
            f.write_str(") { ... }")?;
        }
        Ok(())
    }
}

impl fmt::Debug for Function<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.fmt_with::<Module>(f, None)
    }
}

#[derive(Clone)]
pub struct InstructionSequence<'b>(&'b [u8]);

impl<'b> InstructionSequence<'b> {
    pub fn new(instructions: &'b [u8]) -> Self {
        Self(instructions)
    }

    pub fn get(&self) -> &'b [u8] {
        self.0
    }

    #[inline]
    pub fn iter(&self) -> InstructionIter<'_, '_> {
        InstructionIter::new(self)
    }
}

impl<'a, 'b> IntoIterator for &'a InstructionSequence<'b>
where
    'a: 'b,
{
    type Item = Result<Instruction>;
    type IntoIter = InstructionIter<'a, 'b>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'b> InstructionSequence<'b> {
    pub(crate) fn fmt_with<M: ModuleFormat>(
        &self,
        f: &mut fmt::Formatter<'_>,
        module: Option<&M>,
    ) -> fmt::Result {
        use fmt::Debug;

        if f.alternate() {
            for ins in self
                .iter()
                .with_offset()
                .map(Some)
                .intersperse_with(|| None)
            {
                if let Some((offset, ins)) = ins {
                    match ins {
                        Ok(ins) => {
                            write!(f, "           {offset:5}  ")?;
                            ins.fmt_with(f, module)?;
                        }
                        Err(_error) => write!(f, "           {offset:5}  ...")?,
                    }
                } else {
                    f.write_str("\n")?;
                }
            }
            Ok(())
        } else {
            self.0.fmt(f)
        }
    }
}

impl fmt::Debug for InstructionSequence<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.fmt_with::<Module>(f, None)
    }
}

#[derive(Debug, Clone)]
pub struct InstructionIter<'a, 'b>
where
    'a: 'b,
{
    instructions: &'a InstructionSequence<'b>,
    offset: usize,
    iter: std::slice::Iter<'b, u8>,
}

impl<'a, 'b> InstructionIter<'a, 'b> {
    fn new(instructions: &'a InstructionSequence<'b>) -> Self {
        Self {
            instructions,
            offset: 0,
            iter: instructions.get().iter(),
        }
    }

    pub fn offset(&self) -> usize {
        self.offset
    }

    pub fn with_offset(self) -> OffsetInstructionIter<'a, 'b> {
        OffsetInstructionIter::new(self)
    }

    fn advance(&mut self) -> Option<u8> {
        let item = self.iter.next().copied()?;
        self.offset += 1;
        Some(item)
    }

    fn opcode(&mut self) -> Option<Result<Opcode>> {
        self.advance().map(|opcode| -> Result<Opcode> {
            let opcode = opcode
                .try_into()
                .map_err(|_| Error::InvalidOpcode(opcode))?;

            Ok(opcode)
        })
    }

    fn parameter(&mut self, opcode: Opcode) -> Result<u8> {
        let parameter = self.advance().ok_or(Error::IncompleteInstruction(opcode))?;
        Ok(parameter)
    }

    pub(crate) fn raw_jump(&mut self, offset: Offset) -> std::result::Result<(), ()> {
        use Offset::*;

        match offset {
            Forward(offset) => self.offset += offset,
            Backward(offset) => self.offset -= offset,
        }
        self.iter = self.instructions.get()[self.offset..].iter();

        Ok(())
    }
}

impl Iterator for InstructionIter<'_, '_> {
    type Item = Result<Instruction>;

    fn next(&mut self) -> Option<Self::Item> {
        use InlineConstant as Inl;
        use Instruction as In;
        use Opcode as Op;

        self.opcode().map(|opcode| {
            opcode.and_then(|opcode| {
                let ins = match opcode {
                    Op::Constant => {
                        let constant = self.parameter(opcode)?;
                        In::Constant(constant as usize)
                    }
                    Op::Unit => In::InlineConstant(Inl::Unit),
                    Op::True => In::InlineConstant(Inl::Bool(true)),
                    Op::False => In::InlineConstant(Inl::Bool(false)),
                    Op::Pop => In::Pop,
                    Op::Unary => {
                        let op = self.parameter(opcode)?;
                        let op = op
                            .try_into()
                            .map_err(|_| Error::InvalidInstruction(opcode))?;
                        In::Unary(op)
                    }
                    Op::Binary => {
                        let op = self.parameter(opcode)?;
                        let op = op
                            .try_into()
                            .map_err(|_| Error::InvalidInstruction(opcode))?;
                        In::Binary(op)
                    }
                    Op::LoadLocal => {
                        let local = self.parameter(opcode)?;
                        In::LoadLocal(local as usize)
                    }
                    Op::LoadNamed => {
                        let constant = self.parameter(opcode)?;
                        In::LoadNamed(constant as usize)
                    }
                    Op::StoreLocal => {
                        let local = self.parameter(opcode)?;
                        In::StoreLocal(local as usize)
                    }
                    Op::StoreNamed => {
                        let constant = self.parameter(opcode)?;
                        In::StoreNamed(constant as usize)
                    }
                    Op::PopScope => {
                        let depth = self.parameter(opcode)?;
                        In::PopScope(depth as usize)
                    }
                    Op::Call => {
                        let arity = self.parameter(opcode)?;
                        In::Call(arity as usize)
                    }
                    Op::Return => In::Return,
                    Op::JumpForward => {
                        let offset = self.parameter(opcode)?;
                        In::Jump(Offset::Forward(offset as usize))
                    }
                    Op::JumpBackward => {
                        let offset = self.parameter(opcode)?;
                        In::Jump(Offset::Backward(offset as usize))
                    }
                    Op::JumpForwardIf => {
                        let offset = self.parameter(opcode)?;
                        In::JumpIf(Offset::Forward(offset as usize))
                    }
                    Op::JumpBackwardIf => {
                        let offset = self.parameter(opcode)?;
                        In::JumpIf(Offset::Backward(offset as usize))
                    }
                };

                Ok(ins)
            })
        })
    }
}

#[derive(Debug, Clone)]
pub struct OffsetInstructionIter<'a, 'b>(InstructionIter<'a, 'b>);

impl<'a, 'b> OffsetInstructionIter<'a, 'b> {
    fn new(iter: InstructionIter<'a, 'b>) -> Self {
        Self(iter)
    }

    pub fn offset(&self) -> usize {
        self.0.offset()
    }

    pub(crate) fn raw_jump(&mut self, offset: Offset) -> std::result::Result<(), ()> {
        self.0.raw_jump(offset)
    }
}

impl Iterator for OffsetInstructionIter<'_, '_> {
    type Item = (usize, Result<Instruction>);

    fn next(&mut self) -> Option<Self::Item> {
        let offset = self.offset();
        let ins = self.0.next()?;
        Some((offset, ins))
    }
}
