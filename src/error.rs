pub type ParseError<'r> = nom::Err<nom::error::Error<&'r str>>;
use std;
use std::fmt::{self, Display};

use serde::{de, ser};

pub type Result<T> = std::result::Result<T, Error>;

// This is a bare-bones implementation. A real library would provide additional
// information in its error type, for example the line and column at which the
// error occurred, the byte offset into the input, or the current key being
// processed.
#[derive(Debug, PartialEq)]
pub enum Error {
    // One or more variants that can be created by data structures through the
    // `ser::Error` and `de::Error` traits. For example the Serialize impl for
    // Mutex<T> might return an error because the mutex is poisoned, or the
    // Deserialize impl for a struct may return an error because a required
    // field is missing.
    Message(String),

    // Zero or more variants that can be created directly by the Serializer and
    // Deserializer without going through `ser::Error` and `de::Error`. These
    // are specific to the format, in this case JSON.
    Eof,
    UnresolvedAbbreviation(String),
    ExpectedBoolean(std::str::ParseBoolError),
    ExpectedInteger(std::num::ParseIntError),
    ExpectedFloat(std::num::ParseFloatError),
    ExpectedChar,
    ExpectedNullValue,
    NullValue,
    ParseError,
    FlagError,
    // Syntax,
    // ExpectedBoolean,
    // ExpectedString,
    // ExpectedNull,
    // ExpectedArray,
    // ExpectedArrayComma,
    // ExpectedArrayEnd,
    // ExpectedMap,
    // ExpectedMapColon,
    // ExpectedMapComma,
    // ExpectedMapEnd,
    // ExpectedEnum,
    // TrailingCharacters,
}

impl ser::Error for Error {
    fn custom<T: Display>(msg: T) -> Self {
        Error::Message(msg.to_string())
    }
}

impl de::Error for Error {
    fn custom<T: Display>(msg: T) -> Self {
        Error::Message(msg.to_string())
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::Message(msg) => f.write_str(msg),
            Error::Eof => f.write_str("unexpected end of input"),
            Error::UnresolvedAbbreviation(s) => write!(f, "unresolved abbreviation: '{}'", s),
            Error::ExpectedBoolean(err) => err.fmt(f),
            Error::ExpectedInteger(err) => err.fmt(f),
            Error::ExpectedFloat(err) => err.fmt(f),
            Error::ExpectedChar => f.write_str("expected char"),
            Error::ExpectedNullValue => f.write_str("null value has contents"),
            Error::NullValue => f.write_str("value has no contents"),
            Error::ParseError => f.write_str("TODO!!!"),
            Error::FlagError => f.write_str("TODO!!!"),
        }
    }
}

impl std::error::Error for Error {}

impl From<std::str::ParseBoolError> for Error {
    fn from(err: std::str::ParseBoolError) -> Self {
        Self::ExpectedBoolean(err)
    }
}

impl From<std::num::ParseIntError> for Error {
    fn from(err: std::num::ParseIntError) -> Self {
        Self::ExpectedInteger(err)
    }
}

impl From<std::num::ParseFloatError> for Error {
    fn from(err: std::num::ParseFloatError) -> Self {
        Self::ExpectedFloat(err)
    }
}

impl From<ParseError<'_>> for Error {
    fn from(err: ParseError<'_>) -> Self {
        println!("{}", err);
        Self::ParseError
    }
}
