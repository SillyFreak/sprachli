use nom::error::ParseError;

#[derive(thiserror::Error, Debug)]
pub enum Error<I> {
    #[error("ParseError: {0}")]
    ParseError(#[from] nom::error::Error<I>),
    #[error("Invalid constant pool entry: unknown type")]
    InvalidConstantType,
    #[error("Invalid constant pool entry: invalid utf8 string")]
    InvalidStringConstant,
    #[error("Invalid constant pool entry: invalid number string")]
    InvalidNumberConstant,
    #[error("Constant #{0} not in constant table of len {1}")]
    InvalidConstantRef(usize, usize),
    #[error("Constant #{0} was not a {1}")]
    InvalidConstantRefType(usize, &'static str),
}

impl<I> ParseError<I> for Error<I> {
    fn from_error_kind(input: I, kind: nom::error::ErrorKind) -> Self {
        nom::error::Error::from_error_kind(input, kind).into()
    }

    fn append(_input: I, _kind: nom::error::ErrorKind, other: Self) -> Self {
        other
    }
}
