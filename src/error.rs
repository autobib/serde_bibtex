//! # Errors for serialization and deserialization.
use core::result;
use core::str::Utf8Error;
use std::io;

use crate::token::ConversionError;

/// The error category of an [`Error`].
#[derive(Debug, PartialEq)]
pub enum Category {
    /// Error while handling IO.
    Io,
    /// Syntax error during deserialization.
    Syntax,
    /// Data representation or conversion errors (such as Serde visitor errors,
    /// unexpanded macros, invalid UTF-8, etc.)
    Data,
    /// Incomplete input.
    Eof,
}

/// The main error type as used by [`de::Deserializer`](crate::de::Deserializer) and
/// [`ser::Serializer`](crate::ser::Serializer).
#[derive(Debug)]
pub struct Error {
    /// The underlying error type.
    pub(crate) code: ErrorCode,
}

/// An alias for a [`Result`](std::result::Result) with error type [`serde_bibtex::Error`](Error).
pub type Result<T> = result::Result<T, Error>;

impl Error {
    /// Categorize the type of the error.
    ///
    /// ```
    /// use serde_bibtex::{from_str, error::Category};
    /// let err = from_str::<()>("@comment(unfinished").unwrap_err();
    /// assert_eq!(err.classify(), Category::Eof);
    /// assert_eq!(err.to_string(), "unclosed '('");
    /// ```
    pub fn classify(&self) -> Category {
        match &self.code {
            ErrorCode::VariableStartsWithDigit
            | ErrorCode::UnexpectedClosingBracket
            | ErrorCode::Expected(_)
            | ErrorCode::ExpectedTextToken
            | ErrorCode::ExpectedEndOfEntry { .. } => Category::Syntax,
            ErrorCode::UnclosedDelimiter(_) | ErrorCode::UnexpectedEof(_) => Category::Eof,
            ErrorCode::Message(_)
            | ErrorCode::InvalidUtf8(_)
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
    pub(crate) fn expected(item: &'static str, found: Option<u8>) -> Self {
        Self::syntax(if found.is_none() {
            ErrorCode::UnexpectedEof(item)
        } else {
            ErrorCode::Expected(item)
        })
    }

    /// Add an entry's delimiter context only to otherwise context-free EOF errors.
    /// Inner scanner diagnostics and visitor errors must survive unchanged.
    pub(crate) fn in_entry(self, closing: u8) -> Self {
        if matches!(self.code, ErrorCode::UnexpectedEof(_)) {
            Self::syntax(ErrorCode::UnclosedDelimiter(match closing {
                b')' => b'(',
                _ => b'{',
            }))
        } else {
            self
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

impl core::error::Error for Error {}

impl serde::de::Error for Error {
    fn custom<T: core::fmt::Display>(msg: T) -> Self {
        Self::syntax(ErrorCode::Message(msg.to_string()))
    }
}

impl serde::ser::Error for Error {
    fn custom<T: core::fmt::Display>(msg: T) -> Self {
        Self::syntax(ErrorCode::Message(msg.to_string()))
    }
}

impl core::fmt::Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        self.code.fmt(f)
    }
}

#[derive(Debug)]
pub(crate) enum ErrorCode {
    Message(String),
    VariableStartsWithDigit,
    UnexpectedClosingBracket,
    Expected(&'static str),
    InvalidSerializationFormat(String),
    ExpectedEndOfEntry { expected: u8, found: u8 },
    UnexpandedMacro(String),
    UnclosedDelimiter(u8),
    UnexpectedEof(&'static str),
    ExpectedTextToken,
    InvalidUtf8(Utf8Error),
    Io(io::Error),
}

impl core::fmt::Display for ErrorCode {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Expected(item) => write!(f, "expected {item}"),
            Self::VariableStartsWithDigit => f.write_str("identifier starts with ASCII digit"),
            Self::UnexpectedClosingBracket => f.write_str("unmatched closing '}'"),
            Self::InvalidUtf8(err) => err.fmt(f),
            Self::Message(msg) => f.write_str(msg),
            Self::UnexpectedEof(item) => write!(f, "unexpected end of input; expected {item}"),
            Self::UnclosedDelimiter(opening) => write!(f, "unclosed '{}'", char::from(*opening)),
            Self::ExpectedEndOfEntry { expected, found } => write!(
                f,
                "expected end of entry '{}', found '{}'",
                char::from(*expected),
                char::from(*found).escape_default(),
            ),
            Self::Io(err) => write!(f, "IO error: {err}"),
            Self::ExpectedTextToken => f.write_str("expected text token, found variable"),
            Self::UnexpandedMacro(s) => write!(f, "expected text, got unresolved macro {s}"),
            Self::InvalidSerializationFormat(msg) => {
                write!(f, "invalid serialization format: {msg}")
            }
        }
    }
}
