//! # Errors for serialization and deserialization.
use std::io::Error as IoError;
use std::result;
use std::str::Utf8Error;

use serde::ser::Error as SeError;

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

impl From<Utf8Error> for Error {
    fn from(err: Utf8Error) -> Self {
        Self::utf8(err)
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
    DisallowedChar(char),
    VariableStartsWithDigit,
    UnexpectedClosingBracket,
    ExpectedNextTokenOrEndOfField,
    UnterminatedTextToken,
    InvalidStartOfEntry,
    ExpectedEndOfEntry,
    UnclosedBracket,
    UnclosedQuote,
    UnexpectedEof,
    ExpectedFieldSep,
    UnresolvedMacro(String),
    InvalidUtf8(Utf8Error),
    Io(String),
    Empty,
}

impl std::fmt::Display for ErrorCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::DisallowedChar(ch) => {
                write!(f, "invalid char {:?}", ch)
            }
            Self::ExpectedFieldSep => f.write_str("expected field separator '='"),
            Self::InvalidStartOfEntry => f.write_str("expected start of entry '{' or '('"),
            Self::VariableStartsWithDigit => f.write_str("identifier starts with ASCII digit"),
            Self::UnexpectedClosingBracket => f.write_str("unmatched closing bracket"),
            Self::UnterminatedTextToken => f.write_str("unmatched opening bracket"),
            Self::UnresolvedMacro(s) => write!(f, "unresolved macro: {s}"),
            Self::InvalidUtf8(err) => err.fmt(f),
            Self::Empty => f.write_str("identifier missing or length 0"),
            Self::Message(msg) => f.write_str(msg),
            Self::UnexpectedEof => f.write_str("TODO"),
            Self::ExpectedNextTokenOrEndOfField => f.write_str("TODO"),
            Self::UnclosedBracket => f.write_str("unclosed '{' in token"),
            Self::UnclosedQuote => f.write_str("unclosed '\"' in token"),
            Self::ExpectedEndOfEntry => f.write_str("expected end of entry"),
            Self::Io(err) => write!(f, "IO error: {err}"),
        }
    }
}
