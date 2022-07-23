//! Sprachli bytecode format
//!
//! The bytecode encompasses all values derived from a source program that are
//! known at compile time and required during runtime. In particular, this
//! includes identifiers, number and string literals, and functions defined in
//! the code.

use std::fmt;

mod error;
mod fmt_helpers;
pub mod instruction;
pub mod parser;

use std::collections::HashMap;

use bigdecimal::BigDecimal;
use itertools::Itertools;
use num_enum::{IntoPrimitive, TryFromPrimitive};

use fmt_helpers::*;
use instruction::{InlineConstant, Instruction, Offset, Opcode};

pub use error::*;
pub use parser::parse_bytecode;

pub type Number = BigDecimal;

#[derive(Debug, Clone)]
pub struct Bytecode<B>(B)
where
    B: AsRef<[u8]>;

#[derive(Debug, Clone)]
pub struct Module<'b> {
    constants: Constants<'b>,
    globals: HashMap<&'b str, usize>,
}

impl<'b> Module<'b> {
    pub fn new(constants: Vec<Constant<'b>>, globals: HashMap<&'b str, usize>) -> Self {
        let constants = Constants::new(constants);
        Self { constants, globals }
    }

    pub fn constants(&self) -> &Vec<Constant<'b>> {
        &self.constants.0
    }

    pub fn constant(&self, index: usize) -> Option<&Constant<'b>> {
        self.constants.0.get(index)
    }

    pub fn globals(&self) -> &HashMap<&'b str, usize> {
        &self.globals
    }

    pub fn global(&self, name: &str) -> Option<&Constant<'b>> {
        let index = *self.globals.get(name)?;
        self.constant(index)
    }
}

#[derive(Clone)]
struct Constants<'b>(Vec<Constant<'b>>);

impl fmt::Debug for Constants<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if f.alternate() {
            f.write_str("[\n")?;
            for constant in self.0.iter().enumerate().map(Some).intersperse(None) {
                match constant {
                    Some((i, constant)) => {
                        let constant = FmtConstant::new(&self.0, constant);
                        write!(f, "{i:5} ")?;
                        constant.fmt(f)?;
                    }
                    None => f.write_str("\n")?,
                }
            }
            f.write_str("\n]")?;
            Ok(())
        } else {
            self.0.fmt(f)
        }
    }
}

impl<'b> Constants<'b> {
    pub fn new(items: Vec<Constant<'b>>) -> Self {
        Self(items)
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

impl fmt::Debug for Constant<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use Constant::*;

        match self {
            Number(value) => fmt::Display::fmt(value, f),
            String(value) => value.fmt(f),
            Function(value) => value.fmt(f),
        }
    }
}

#[derive(Clone)]
pub struct Function<'b> {
    arity: usize,
    body: InstructionSequence<'b>,
}

impl fmt::Debug for Function<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("fn (")?;
        for i in (0..self.arity).map(Some).intersperse(None) {
            match i {
                Some(i) => write!(f, "_{}", i)?,
                None => f.write_str(", ")?,
            }
        }

        if f.alternate() {
            f.write_str(") {\n")?;
            self.body.fmt(f)?;
            f.write_str("\n}")?;
        } else {
            f.write_str(") { ... }")?;
        }
        Ok(())
    }
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
    pub fn iter(&self) -> InstructionIter<'_> {
        InstructionIter::new(self)
    }
}

impl fmt::Debug for InstructionSequence<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if f.alternate() {
            for ins in self
                .iter()
                .with_offset()
                .map(Some)
                .intersperse_with(|| None)
            {
                if let Some((offset, ins)) = ins {
                    match ins {
                        Ok(ins) => write!(f, "{offset:5} {ins:?}")?,
                        Err(_error) => write!(f, "{offset:5} ...")?,
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

impl<'a> IntoIterator for &'a InstructionSequence<'_> {
    type Item = Result<Instruction>;
    type IntoIter = InstructionIter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
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
            Forward(offset) => {
                self.0 += offset;
                if offset > 0 {
                    self.1.nth(offset - 1).ok_or(())?;
                }
            }
            Backward(offset) => {
                self.0 -= offset;
                if offset > 0 {
                    self.1.nth_back(offset - 1).ok_or(())?;
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
                    Op::Call => {
                        let arity = self.parameter(opcode)?;
                        In::Call(arity as usize)
                    }
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
pub struct OffsetInstructionIter<'b>(InstructionIter<'b>);

impl<'b> OffsetInstructionIter<'b> {
    fn new(iter: InstructionIter<'b>) -> Self {
        Self(iter)
    }

    pub fn offset(&self) -> usize {
        self.0.offset()
    }

    pub(crate) fn raw_jump(&mut self, offset: Offset) -> std::result::Result<(), ()> {
        self.0.raw_jump(offset)
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
