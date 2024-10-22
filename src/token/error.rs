use std::error::Error;
use std::fmt;
use std::str::Utf8Error;

/// Possible syntax errors in BibTeX tokens and identifiers.
#[derive(Debug, PartialEq)]
pub enum TokenError {
    /// Expected to be non-empty.
    Empty,
    /// Contains an invalid char.
    InvalidChar(char),
    /// Expected to start with a non-ASCII digit.
    StartsWithDigit,
    /// Has an extra closing bracket.
    ExtraClosingBracket,
    /// Has too many opening brackets.
    ExtraOpeningBracket,
}

/// An error which results when converting between text and variable tokens.
pub enum ConversionError {
    /// Expected a text token; got macro.
    UnexpandedMacro(String),
    /// Text contains invalid bytes.
    InvalidUtf8(Utf8Error),
}

impl From<Utf8Error> for ConversionError {
    fn from(err: Utf8Error) -> Self {
        Self::InvalidUtf8(err)
    }
}

impl fmt::Display for TokenError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TokenError::Empty => f.write_str("identifier must be non-empty"),
            TokenError::InvalidChar(ch) => {
                write!(f, "identifier contains invalid character '{ch}'")
            }
            TokenError::StartsWithDigit => f.write_str("variable cannot start with digit"),
            TokenError::ExtraClosingBracket => f.write_str("text token has extra closing bracket"),
            TokenError::ExtraOpeningBracket => {
                f.write_str("text token has unclosed opening bracket")
            }
        }
    }
}

impl Error for TokenError {}

/// Errors which result while attempting to construct a token type from an input.
#[derive(Debug, PartialEq)]
pub struct TokenParseError<S> {
    /// The input object.
    pub input: S,
    /// The syntax error which occurred.
    pub error: TokenError,
}

impl<S> From<TokenParseError<S>> for TokenError {
    fn from(value: TokenParseError<S>) -> Self {
        value.error
    }
}
