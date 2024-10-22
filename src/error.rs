//! # Errors for serialization and deserialization.
use std::io;
use std::result;
use std::str::Utf8Error;

use crate::token::ConversionError;

/// The error category of a given [`bibtex::Error`](Error).
#[derive(Debug, PartialEq)]
pub enum Category {
    /// Error while handling IO.
    Io,
    /// Syntax error during deserialization.
    Syntax,
    /// Data error, such as unexpanded macros or invalid serialization format.
    Data,
    /// Unexpected end of input.
    Eof,
}

/// The main error type as used by [`de::Deserializer`](crate::de::Deserializer) and
/// [`ser::Serializer`](crate::ser::Serializer).
#[derive(Debug)]
pub struct Error {
    /// The underlying error type.
    pub(crate) code: ErrorCode,
}

/// Alias for a [`Result`](std::result::Result) with the error type [`bibtex::Error`](crate::error::Error).
pub type Result<T> = result::Result<T, Error>;

impl Error {
    pub fn classify(&self) -> Category {
        match &self.code {
            ErrorCode::Message(_)
            | ErrorCode::VariableStartsWithDigit
            | ErrorCode::UnexpectedClosingBracket
            | ErrorCode::ExpectedNextTokenOrEndOfField
            | ErrorCode::UnterminatedTextToken
            | ErrorCode::InvalidStartOfEntry
            | ErrorCode::ExpectedFieldSep
            | ErrorCode::Empty
            | ErrorCode::ExpectedEndOfEntry => Category::Syntax,
            ErrorCode::UnclosedQuote | ErrorCode::UnexpectedEof | ErrorCode::UnclosedBracket => {
                Category::Eof
            }
            ErrorCode::InvalidUtf8(_)
            | ErrorCode::UnexpandedMacro(_)
            | ErrorCode::InvalidSerializationFormat(_) => Category::Data,
            ErrorCode::Io(_) => Category::Io,
        }
    }

    #[inline]
    pub(crate) fn syntax(code: ErrorCode) -> Self {
        Self { code }
    }

    #[inline]
    pub(crate) fn utf8(err: Utf8Error) -> Self {
        Self {
            code: ErrorCode::InvalidUtf8(err),
        }
    }

    #[inline]
    pub(crate) fn ser(msg: String) -> Self {
        Self {
            code: ErrorCode::InvalidSerializationFormat(msg),
        }
    }

    #[inline]
    pub(crate) fn io(err: io::Error) -> Self {
        Self {
            code: ErrorCode::Io(err),
        }
    }

    #[inline]
    pub(crate) fn eof() -> Self {
        Self {
            code: ErrorCode::UnexpectedEof,
        }
    }
}

impl From<ConversionError> for Error {
    #[inline]
    fn from(value: ConversionError) -> Self {
        match value {
            ConversionError::UnexpandedMacro(s) => Self {
                code: ErrorCode::UnexpandedMacro(s),
            },
            ConversionError::InvalidUtf8(err) => Self::utf8(err),
        }
    }
}

impl From<Utf8Error> for Error {
    #[inline]
    fn from(err: Utf8Error) -> Self {
        Self {
            code: ErrorCode::InvalidUtf8(err),
        }
    }
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Self {
        Self::io(err)
    }
}

impl std::error::Error for Error {}

impl serde::de::Error for Error {
    fn custom<T: std::fmt::Display>(msg: T) -> Self {
        Self::syntax(ErrorCode::Message(msg.to_string()))
    }
}

impl serde::ser::Error for Error {
    fn custom<T: std::fmt::Display>(msg: T) -> Self {
        Self::syntax(ErrorCode::Message(msg.to_string()))
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.code.fmt(f)
    }
}

#[derive(Debug)]
pub(crate) enum ErrorCode {
    Message(String),
    VariableStartsWithDigit,
    UnexpectedClosingBracket,
    ExpectedNextTokenOrEndOfField,
    InvalidSerializationFormat(String),
    UnterminatedTextToken,
    InvalidStartOfEntry,
    ExpectedEndOfEntry,
    UnexpandedMacro(String),
    UnclosedBracket,
    UnclosedQuote,
    UnexpectedEof,
    ExpectedFieldSep,
    InvalidUtf8(Utf8Error),
    Io(io::Error),
    Empty,
}

impl std::fmt::Display for ErrorCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ExpectedFieldSep => f.write_str("expected field separator '='"),
            Self::InvalidStartOfEntry => f.write_str("expected start of entry '{' or '('"),
            Self::VariableStartsWithDigit => f.write_str("identifier starts with ASCII digit"),
            Self::UnexpectedClosingBracket => f.write_str("unmatched closing bracket"),
            Self::UnterminatedTextToken => f.write_str("unmatched opening bracket"),
            Self::InvalidUtf8(err) => err.fmt(f),
            Self::Empty => f.write_str("identifier missing or length 0"),
            Self::Message(msg) => f.write_str(msg),
            Self::UnexpectedEof => f.write_str("unexpected end of input"),
            Self::ExpectedNextTokenOrEndOfField => {
                f.write_str("expected another token or a field terminator")
            }
            Self::UnclosedBracket => f.write_str("unclosed '{' in token"),
            Self::UnclosedQuote => f.write_str("unclosed '\"' in token"),
            Self::ExpectedEndOfEntry => f.write_str("expected end of entry"),
            Self::Io(err) => write!(f, "IO error: {err}"),
            Self::UnexpandedMacro(s) => write!(f, "expected text, got unresolved macro {s}"),
            Self::InvalidSerializationFormat(msg) => {
                write!(f, "invalid serialization format: {msg}")
            }
        }
    }
}
