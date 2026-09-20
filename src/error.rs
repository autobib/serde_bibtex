//! # Errors for serialization and deserialization.
use core::str::Utf8Error;
use core::{ops::Range, result};
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
    inner: Box<ErrorImpl>,
}

#[derive(Debug)]
struct ErrorImpl {
    code: ErrorCode,
    span: Option<Range<usize>>,
}

/// An alias for a [`Result`](std::result::Result) with error type [`serde_bibtex::Error`](Error).
pub type Result<T> = result::Result<T, Error>;

impl Error {
    #[cfg(test)]
    pub(crate) fn inner_code(&self) -> &ErrorCode {
        &self.inner.code
    }

    /// Categorize the type of the error.
    ///
    /// ```
    /// use serde_bibtex::{from_str, error::Category};
    /// let err = from_str::<()>("@comment(unfinished").unwrap_err();
    /// assert_eq!(err.classify(), Category::Eof);
    /// assert_eq!(err.to_string(), "unclosed '('");
    /// ```
    pub fn classify(&self) -> Category {
        match &self.inner.code {
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
    pub(crate) fn new(code: ErrorCode) -> Self {
        Self {
            inner: Box::new(ErrorImpl { code, span: None }),
        }
    }

    #[inline]
    pub(crate) fn utf8(err: Utf8Error) -> Self {
        Self::new(ErrorCode::InvalidUtf8(err))
    }

    #[inline]
    pub(crate) fn ser(msg: String) -> Self {
        Self::new(ErrorCode::InvalidSerializationFormat(msg))
    }

    #[inline]
    pub(crate) fn io(err: io::Error) -> Self {
        Self::new(ErrorCode::Io(err))
    }

    #[inline]
    pub(crate) fn expected(item: &'static str, found: Option<u8>) -> Self {
        Self::new(if found.is_none() {
            ErrorCode::UnexpectedEof(item)
        } else {
            ErrorCode::Expected(item)
        })
    }

    /// The byte range in the original input associated with this error.
    ///
    /// This is the error span produced by the [`BibtexRead`](crate::parse::BibtexRead)
    /// implementation. When produced by a [`StrReader`](crate::parse::StrReader), the range is
    /// guaranteed to correspond to a valid string slice in the original input.
    ///
    /// Errors concerning an entire entry cover the input consumed so far, excluding any
    /// unconsumed closing delimiter.
    ///
    /// For errors with no source (for instance, serialization errors or IO errors), this
    /// returns `None`.
    ///
    /// Pass this range and the original input to a diagnostic renderer such as
    /// `annotate-snippets` to highlight the error. See `examples/format.rs` for an example.
    ///
    /// ```
    /// let error = serde_bibtex::from_str::<()>("@comment(unfinished").unwrap_err();
    /// assert_eq!(error.span(), Some(8..9));
    /// ```
    pub fn span(&self) -> Option<Range<usize>> {
        self.inner.span.clone()
    }

    pub(crate) fn with_span(mut self, span: Option<Range<usize>>) -> Self {
        if self.inner.span.is_none() {
            self.inner.span = span;
        }
        self
    }

    /// Refine a literal's content span to the invalid UTF-8 bytes, when applicable.
    pub(crate) fn with_utf8_span(self, content: Option<Range<usize>>) -> Self {
        if let ErrorCode::InvalidUtf8(err) = &self.inner.code {
            let span = content.map(|mut span| {
                span.start += err.valid_up_to();
                span.end = err.error_len().map_or(span.end, |len| span.start + len);
                span
            });
            self.with_span(span)
        } else {
            self
        }
    }
}

impl From<ConversionError> for Error {
    #[inline]
    fn from(value: ConversionError) -> Self {
        match value {
            ConversionError::UnexpandedMacro(s) => Self::new(ErrorCode::UnexpandedMacro(s)),
            ConversionError::InvalidUtf8(err) => Self::utf8(err),
        }
    }
}

impl From<Utf8Error> for Error {
    #[inline]
    fn from(err: Utf8Error) -> Self {
        Self::new(ErrorCode::InvalidUtf8(err))
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
        Self::new(ErrorCode::Message(msg.to_string()))
    }
}

impl serde::ser::Error for Error {
    fn custom<T: core::fmt::Display>(msg: T) -> Self {
        Self::new(ErrorCode::Message(msg.to_string()))
    }
}

impl core::fmt::Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        self.inner.code.fmt(f)
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
