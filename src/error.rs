//! # Errors for serialization and deserialization.
use std::io::Error as IoError;
use std::result;
use std::str::Utf8Error;

use serde::ser::Error as SeError;

use crate::token::ConversionError;

#[derive(Debug, PartialEq)]
pub struct Error {
    code: ErrorCode,
}

pub type Result<T> = result::Result<T, Error>;

impl Error {
    pub(crate) fn syntax(code: ErrorCode) -> Self {
        Self { code }
    }

    pub(crate) fn utf8(err: Utf8Error) -> Self {
        Self {
            code: ErrorCode::InvalidUtf8(err),
        }
    }

    pub(crate) fn io(err: IoError) -> Self {
        Self {
            code: ErrorCode::Io(err.to_string()),
        }
    }

    pub(crate) fn eof() -> Self {
        Self {
            code: ErrorCode::UnexpectedEof,
        }
    }

    pub(crate) fn only_seq() -> Self {
        Self::custom("bibliography must be a sequence")
    }

    pub(crate) fn only_enum() -> Self {
        Self::custom("entry must be an enum")
    }
}

impl From<ConversionError> for Error {
    fn from(value: ConversionError) -> Self {
        match value {
            ConversionError::UnresolvedMacro(s) => Self {
                code: ErrorCode::UnresolvedMacro(s),
            },
            ConversionError::InvalidUtf8(err) => Self::utf8(err),
        }
    }
}

impl From<Utf8Error> for Error {
    fn from(err: Utf8Error) -> Self {
        Self {
            code: ErrorCode::InvalidUtf8(err),
        }
    }
}

impl From<IoError> for Error {
    fn from(err: IoError) -> Self {
        Self::io(err)
    }
}

impl std::error::Error for Error {}

impl serde::de::Error for Error {
    fn custom<T: std::fmt::Display>(msg: T) -> Self {
        Self::syntax(ErrorCode::Message(msg.to_string()))
    }
}

impl SeError for Error {
    fn custom<T: std::fmt::Display>(msg: T) -> Self {
        Self::syntax(ErrorCode::Message(msg.to_string()))
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.code.fmt(f)
    }
}

#[derive(Debug, PartialEq)]
pub(crate) enum ErrorCode {
    Message(String),
    VariableStartsWithDigit,
    UnexpectedClosingBracket,
    ExpectedNextTokenOrEndOfField,
    UnterminatedTextToken,
    InvalidStartOfEntry,
    ExpectedEndOfEntry,
    UnresolvedMacro(String),
    UnclosedBracket,
    UnclosedQuote,
    UnexpectedEof,
    ExpectedFieldSep,
    InvalidUtf8(Utf8Error),
    Io(String),
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
            Self::UnresolvedMacro(s) => write!(f, "expected text, got unresolved macro {s}"),
        }
    }
}
