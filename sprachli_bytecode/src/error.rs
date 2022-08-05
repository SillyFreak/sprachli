use std::fmt;

use nom::error::ParseError;

use super::instruction::Opcode;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("ParseError: {0}")]
    ParseError(String),
    #[error("Invalid constant pool entry: unknown type")]
    InvalidConstantType,
    #[error("Invalid constant pool entry: invalid utf8 string")]
    InvalidStringConstant,
    #[error("Invalid constant pool entry: invalid number string")]
    InvalidNumberConstant,
    #[error("Invalid opcode: {0}")]
    InvalidOpcode(u8),
    #[error("Incomplete instruction: {0:?}")]
    IncompleteInstruction(Opcode),
    #[error("Invalid Instruction: {0:?}")]
    InvalidInstruction(Opcode),
    #[error("Constant #{0} not in constant table of len {1}")]
    InvalidConstantRef(usize, usize),
    #[error("Constant #{0} was not a {1}")]
    InvalidConstantRefType(usize, &'static str),
    #[error("Invalid struct: unknown type")]
    InvalidStructType,
}

impl<I: fmt::Debug> From<nom::error::Error<I>> for Error {
    fn from(error: nom::error::Error<I>) -> Self {
        Self::ParseError(format!("{:?}", error))
    }
}

impl<I: fmt::Debug> ParseError<I> for Error {
    fn from_error_kind(input: I, kind: nom::error::ErrorKind) -> Self {
        nom::error::Error::from_error_kind(input, kind).into()
    }

    fn append(_input: I, _kind: nom::error::ErrorKind, other: Self) -> Self {
        other
    }
}

pub type Result<T> = std::result::Result<T, Error>;
