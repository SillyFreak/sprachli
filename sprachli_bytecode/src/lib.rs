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

use std::collections::BTreeMap;

use bigdecimal::BigDecimal;
use itertools::Itertools;
use num_enum::{IntoPrimitive, TryFromPrimitive};
use sprachli_fmt::{FormatterExt, ModuleFormat};

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
    globals: BTreeMap<&'b str, usize>,
    structs: BTreeMap<&'b str, Struct<'b>>,
}

impl<'b> Module<'b> {
    pub fn new(
        constants: Vec<Constant<'b>>,
        globals: BTreeMap<&'b str, usize>,
        structs: BTreeMap<&'b str, Struct<'b>>,
    ) -> Self {
        Self {
            constants,
            globals,
            structs,
        }
    }

    pub fn constants(&self) -> &Vec<Constant<'b>> {
        &self.constants
    }

    pub fn constant(&self, index: usize) -> Option<&Constant<'b>> {
        self.constants.get(index)
    }

    pub fn globals(&self) -> &BTreeMap<&'b str, usize> {
        &self.globals
    }

    pub fn global(&self, name: &str) -> Option<&Constant<'b>> {
        let index = *self.globals.get(name)?;
        self.constant(index)
    }

    pub fn structs(&self) -> &BTreeMap<&'b str, Struct<'b>> {
        &self.structs
    }

    pub fn strucct(&self, name: &str) -> Option<&Struct<'b>> {
        self.structs.get(name)
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
            f.write_str("    structs: {\n")?;
            for (name, decl) in &self.structs {
                f.write_str("        ")?;
                f.write_str(name)?;
                f.write_str(": ")?;
                decl.fmt(f)?;
                f.write_str("\n")?;
            }
            f.write_str("    },\n")?;
            f.write_str("}")?;
            Ok(())
        } else {
            f.debug_struct("Module")
                .field("constants", &self.constants)
                .field("globals", &self.globals)
                .field("structs", &self.structs)
                .finish()
        }
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, IntoPrimitive, TryFromPrimitive)]
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

#[derive(Debug, Clone, Copy, Eq, PartialEq, IntoPrimitive, TryFromPrimitive)]
#[repr(u8)]
pub enum StructType {
    Empty,
    Positional,
    Named,
}

#[derive(Clone, PartialEq, Eq)]
pub enum Struct<'b> {
    Empty,
    Positional(usize),
    Named(Vec<&'b str>),
}

impl fmt::Debug for Struct<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use Struct::*;

        match self {
            Empty => f.write_str("struct;"),
            Positional(count) => {
                f.write_str("struct(")?;
                for i in (0..*count).map(Some).intersperse(None) {
                    match i {
                        Some(i) => write!(f, "_{}", i)?,
                        None => f.write_str(", ")?,
                    }
                }
                f.write_str(");")?;
                Ok(())
            }
            Named(members) => {
                if members.is_empty() {
                    f.write_str("struct {}")?;
                } else {
                    f.write_str("struct { ")?;
                    for member in members.iter().map(Some).intersperse(None) {
                        match member {
                            Some(member) => f.write_str(member)?,
                            None => f.write_str(", ")?,
                        }
                    }
                    f.write_str(" }")?;
                }
                Ok(())
            }
        }
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

    fn parameter_u8(&mut self, opcode: Opcode) -> Result<u8> {
        let parameter = self.advance().ok_or(Error::IncompleteInstruction(opcode))?;
        Ok(parameter)
    }

    fn instruction_u8<F>(&mut self, opcode: Opcode, f: F) -> Result<Instruction>
    where
        F: FnOnce(usize) -> Instruction,
    {
        let parameter = self.parameter_u8(opcode)?;
        Ok(f(parameter as usize))
    }

    pub fn jump(&mut self, offset: Offset) -> std::result::Result<(), ()> {
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
                    Op::Constant => self.instruction_u8(opcode, In::Constant)?,
                    Op::Unit => In::InlineConstant(Inl::Unit),
                    Op::True => In::InlineConstant(Inl::Bool(true)),
                    Op::False => In::InlineConstant(Inl::Bool(false)),
                    Op::Unary => {
                        let op = self.parameter_u8(opcode)?;
                        let op = op
                            .try_into()
                            .map_err(|_| Error::InvalidInstruction(opcode))?;
                        In::Unary(op)
                    }
                    Op::Binary => {
                        let op = self.parameter_u8(opcode)?;
                        let op = op
                            .try_into()
                            .map_err(|_| Error::InvalidInstruction(opcode))?;
                        In::Binary(op)
                    }
                    Op::LoadLocal => self.instruction_u8(opcode, In::LoadLocal)?,
                    Op::StoreLocal => self.instruction_u8(opcode, In::StoreLocal)?,
                    Op::LoadNamed => self.instruction_u8(opcode, In::LoadNamed)?,
                    Op::StoreNamed => self.instruction_u8(opcode, In::StoreNamed)?,
                    Op::LoadPositionalField => {
                        self.instruction_u8(opcode, In::LoadPositionalField)?
                    }
                    Op::StorePositionalField => {
                        self.instruction_u8(opcode, In::StorePositionalField)?
                    }
                    Op::LoadNamedField => self.instruction_u8(opcode, In::LoadNamedField)?,
                    Op::StoreNamedField => self.instruction_u8(opcode, In::StoreNamedField)?,
                    Op::Pop => In::Pop,
                    Op::PopScope => self.instruction_u8(opcode, In::PopScope)?,
                    Op::Call => self.instruction_u8(opcode, In::Call)?,
                    Op::Return => In::Return,
                    Op::JumpForward => {
                        self.instruction_u8(opcode, |off| In::Jump(Offset::Forward(off)))?
                    }
                    Op::JumpBackward => {
                        self.instruction_u8(opcode, |off| In::Jump(Offset::Backward(off)))?
                    }
                    Op::JumpForwardIf => {
                        self.instruction_u8(opcode, |off| In::JumpIf(Offset::Forward(off)))?
                    }
                    Op::JumpBackwardIf => {
                        self.instruction_u8(opcode, |off| In::JumpIf(Offset::Backward(off)))?
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

    pub fn jump(&mut self, offset: Offset) -> std::result::Result<(), ()> {
        self.0.jump(offset)
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
