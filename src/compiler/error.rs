use std::fmt;
use std::io::Error as IoError;

use bigdecimal::ParseBigDecimalError;
use lalrpop_util::ParseError as LalrpopParseError;

use crate::parser::{Error as ParseError, ParseStringError};

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("IO Error: {0}")]
    Io(#[from] IoError),
    #[error("Parse Error: {0}")]
    Parse(LalrpopParseError<usize, Token, &'static str>),
    #[error("Invalid assignment target")]
    InvalidAssignmentTarget,
    #[error("Assignment to immutable variable")]
    ImmutableVariable,
    #[error("break/continue statement without enclosing loop")]
    NoLoopToExit,
    #[error("Unsupported language construct: {0}")]
    Unsupported(&'static str),
    #[error("Internal Error: {0}")]
    Internal(#[from] InternalError),
}

impl From<ParseError<'_>> for Error {
    fn from(error: ParseError<'_>) -> Self {
        Error::Parse(error.map_token(|t| Token(t.0, t.1.to_string())))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Token(pub usize, pub String);

impl fmt::Display for Token {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(&self.1, formatter)
    }
}

pub type Result<T> = std::result::Result<T, Error>;

#[derive(thiserror::Error, Debug)]
pub enum InternalError {
    #[error("Invalid number literal: {0}")]
    InvalidNumberLiteral(#[from] ParseBigDecimalError),
    #[error("Invalid string literal: {0}")]
    InvalidStringLiteral(#[from] ParseStringError),
    #[error("Invalid stack effect: an instruction would pop from an empty stack")]
    InvalidStackEffect,
    #[error("Invalid bytecode: an invalid instruction sequence was generated")]
    InvalidBytecode,
}
