//! Sprachli bytecode format
//!
//! The bytecode encompasses all values derived from a source program that are
//! known at compile time and required during runtime. In particular, this
//! includes identifiers, number and string literals, and functions defined in
//! the code.

mod error;
mod instructions;
pub mod parser;
pub mod writer;

use std::collections::HashMap;

use bigdecimal::BigDecimal;
use num_enum::{IntoPrimitive, TryFromPrimitive};

pub use instructions::InstructionSequence;

pub type Number = BigDecimal;

#[derive(Debug, Clone)]
pub struct Bytecode<B>(B)
where
    B: AsRef<[u8]>;

#[derive(Debug, Clone)]
pub struct Module<'b> {
    constants: Vec<Constant<'b>>,
    globals: HashMap<usize, usize>,
}

impl<'b> Module<'b> {
    pub fn new(constants: Vec<Constant<'b>>, globals: HashMap<usize, usize>) -> Self {
        Self { constants, globals }
    }
}

#[derive(Debug, Eq, PartialEq, IntoPrimitive, TryFromPrimitive)]
#[repr(u8)]
pub enum ConstantType {
    Number,
    String,
    Function,
}

#[derive(Debug, Clone)]
pub enum Constant<'b> {
    Number(Number),
    String(&'b str),
    Function(Function<'b>),
}

#[derive(Debug, Clone)]
pub struct Function<'b> {
    arity: usize,
    body: InstructionSequence<'b>,
}

impl<'b> Function<'b> {
    pub fn new(arity: usize, body: InstructionSequence<'b>) -> Self {
        Self { arity, body }
    }
}
