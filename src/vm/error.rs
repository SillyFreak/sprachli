use bigdecimal::ParseBigDecimalError;

use crate::bytecode::Error as BytecodeError;
use crate::parser::ParseStringError;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Name not known: {0}")]
    NameError(String),
    #[error("Type error, expected: {0}")]
    TypeError(String),
    #[error("Value error: {0}")]
    ValueError(String),
    #[error("Unsupported language construct: {0}")]
    Unsupported(&'static str),
    #[error("Internal Error: {0}")]
    Internal(#[from] InternalError),
}

pub type Result<T> = std::result::Result<T, Error>;

#[derive(thiserror::Error, Debug)]
pub enum InternalError {
    #[error("Constant #{0} not in constant table of len {1}")]
    InvalidConstant(usize, usize),
    #[error("Invalid local variable #{0}")]
    InvalidLocal(usize),
    #[error("Constant #{0} was not a {1}")]
    InvalidConstantType(usize, &'static str),
    #[error("Invalid number literal: {0}")]
    InvalidNumberLiteral(#[from] ParseBigDecimalError),
    #[error("Invalid string literal: {0}")]
    InvalidStringLiteral(#[from] ParseStringError),
    #[error("Tried to pop from an empty stack")]
    EmptyStack,
    #[error("Invalid bytecode sequence: {0}")]
    InvalidBytecode(#[from] BytecodeError),
    #[error("Tried to jump to nonexistent instruction")]
    InvalidJump,
}
