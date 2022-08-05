#[derive(thiserror::Error, Debug)]
pub enum ParseStringError {
    #[error("string literal without opening double quote")]
    MissingOpenQuote,
    #[error("unfinished escape sequence")]
    UnfinishedEscapeSequence,
    #[error("illegal escape sequence: '\\{0}'")]
    IllegalEscapeSequence(char),
    #[error("string literal without closing double quote")]
    MissingClosedQuote,
    #[error("string literal with trailing content after the closing double quote")]
    TrailingContent,
}

pub fn string_from_literal(literal: &str) -> Result<String, ParseStringError> {
    use ParseStringError::*;

    let mut string = String::with_capacity(literal.len());

    let mut iter = literal.chars();
    iter.next()
        .filter(|&ch| ch == '"')
        .ok_or(MissingOpenQuote)?;
    while let Some(ch) = iter.next() {
        match ch {
            '\\' => {
                let ch = iter.next().ok_or(UnfinishedEscapeSequence)?;
                match ch {
                    '\\' | '\"' => string.push(ch),
                    'n' => string.push('\n'),
                    'r' => string.push('\r'),
                    't' => string.push('\t'),
                    _ => Err(IllegalEscapeSequence(ch))?,
                }
            }
            '"' => {
                if iter.next().is_some() {
                    Err(TrailingContent)?;
                }

                return Ok(string);
            }
            _ => {
                string.push(ch);
            }
        }
    }

    Err(MissingClosedQuote)
}
