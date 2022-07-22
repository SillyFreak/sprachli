//! Sprachli bytecode format
//!
//! The bytecode encompasses all values derived from a source program that are
//! known at compile time and required during runtime. In particular, this
//! includes identifiers, number and string literals, and functions defined in
//! the code.

use std::fmt;

pub mod error;
pub mod instruction;
pub mod parser;

use std::collections::HashMap;

use bigdecimal::BigDecimal;
use itertools::Itertools;
use num_enum::{IntoPrimitive, TryFromPrimitive};

pub type Number = BigDecimal;

#[derive(Debug, Clone)]
pub struct Bytecode<B>(B)
where
    B: AsRef<[u8]>;

#[derive(Debug, Clone)]
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
            String(value) => fmt::Display::fmt(value, f),
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
        f.write_str(") { ... }")
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

#[derive(Debug, Clone)]
pub struct InstructionSequence<'b>(&'b [u8]);

impl<'b> InstructionSequence<'b> {
    pub fn new(instructions: &'b [u8]) -> Self {
        Self(instructions)
    }

    pub fn get(&self) -> &'b [u8] {
        self.0
    }
}
