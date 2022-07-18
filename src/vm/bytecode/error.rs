#[derive(thiserror::Error, Debug)]
pub enum ParseError<I> {
    #[error("ParseError: {0}")]
    ParseError(#[from] nom::error::Error<I>),
    #[error("Invalid constant pool entry: unknown type")]
    InvalidConstantType,
    #[error("Invalid constant pool entry: invalid utf8 string")]
    InvalidStringConstant,
    #[error("Invalid constant pool entry: invalid number string")]
    InvalidNumberConstant,
}

impl<I> nom::error::ParseError<I> for ParseError<I> {
    fn from_error_kind(input: I, kind: nom::error::ErrorKind) -> Self {
        nom::error::Error::from_error_kind(input, kind).into()
    }

    fn append(input: I, kind: nom::error::ErrorKind, other: Self) -> Self {
        other
    }
}