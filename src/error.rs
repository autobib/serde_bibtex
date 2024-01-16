use std::str::Utf8Error;

#[derive(Debug, PartialEq)]
pub enum TokenConversionError {
    UnresolvedMacro(String),
    InvalidUtf8(Utf8Error),
}

impl std::fmt::Display for TokenConversionError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.write_str("could not convert token: ")?;
        match self {
            TokenConversionError::UnresolvedMacro(s) => write!(f, "unresolved macro: {s}"),
            TokenConversionError::InvalidUtf8(err) => err.fmt(f),
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum ReadError {
    Eof,
    ExpectedIdentifier,
    Unbalanced,
    InvalidUtf8(Utf8Error),
}

impl std::fmt::Display for ReadError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.write_str("error while reading file: ")?;
        match self {
            ReadError::Eof => f.write_str("unexpected end of file"),
            ReadError::ExpectedIdentifier => f.write_str("expected identifier"),
            ReadError::Unbalanced => f.write_str("unbalanced curly brackets {}"),
            ReadError::InvalidUtf8(err) => err.fmt(f),
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum Error {
    Message(String),
    UnresolvedMacro(String),
    ReadError(ReadError),
    InvalidStartOfEntry,
    ExpectedFieldSep,
    UnexpectedEof,
    TokenConversion(TokenConversionError),
    InvalidUtf8(Utf8Error),
    ExpectedNextTokenOrEndOfField,
    UnclosedBracket,
    UnclosedQuote,
    TooManyChars,
    ExpectedEndOfEntry,
    NoChars,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Error::Message(msg) => f.write_str(msg),
            Error::InvalidUtf8(err) => err.fmt(f),
            Error::ReadError(_err) => f.write_str("Read error"),
            Error::InvalidStartOfEntry => f.write_str("TODO"),
            Error::UnexpectedEof => f.write_str("TODO"),
            Error::UnresolvedMacro(_) => f.write_str("TODO"),
            Error::ExpectedNextTokenOrEndOfField => f.write_str("TODO"),
            Error::TokenConversion(err) => err.fmt(f),
            Error::TooManyChars => f.write_str("too many chars"),
            Error::NoChars => f.write_str("expected char, got nothing"),
            Error::ExpectedFieldSep => f.write_str("expected field separator '='"),
            Error::UnclosedBracket => f.write_str("unclosed '{' in token"),
            Error::UnclosedQuote => f.write_str("unclosed '\"' in token"),
            Error::ExpectedEndOfEntry => f.write_str("expected end of entry"),
        }
    }
}

impl std::error::Error for Error {}

impl serde::de::Error for Error {
    fn custom<T: std::fmt::Display>(msg: T) -> Self {
        Self::Message(msg.to_string())
    }
}

impl From<ReadError> for Error {
    fn from(err: ReadError) -> Self {
        Self::ReadError(err)
    }
}

impl From<Utf8Error> for Error {
    fn from(err: Utf8Error) -> Self {
        Self::InvalidUtf8(err)
    }
}

impl From<TokenConversionError> for Error {
    fn from(err: TokenConversionError) -> Self {
        Self::TokenConversion(err)
    }
}
