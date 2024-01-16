mod create_input_impl;
mod slice_impl;
mod str_impl;
pub use slice_impl::SliceReader;
pub use str_impl::StrReader;

use crate::error::ReadError;
use std::borrow::Cow;
use std::str::Utf8Error;

pub struct UnicodeIdentifier<'r>(pub Cow<'r, str>);
pub struct AsciiIdentifier<'r>(pub Cow<'r, str>);

/// The core Text handling object.
#[derive(Debug, Clone, PartialEq)]
pub enum Text<'r> {
    Str(Cow<'r, str>),
    Raw(Cow<'r, [u8]>),
}

impl<'r> Text<'r> {
    pub fn into_cow_str(self) -> Result<Cow<'r, str>, Utf8Error> {
        match self {
            Self::Str(cow) => Ok(cow),
            Self::Raw(Cow::Borrowed(bytes)) => Ok(Cow::Borrowed(std::str::from_utf8(bytes)?)),
            Self::Raw(Cow::Owned(bytes)) => Ok(Cow::Owned(
                String::from_utf8(bytes).map_err(|e| e.utf8_error())?,
            )),
        }
    }

    pub fn into_bytes(self) -> Cow<'r, [u8]> {
        match self {
            Self::Str(Cow::Borrowed(str)) => Cow::Borrowed(str.as_bytes()),
            Self::Str(Cow::Owned(s)) => Cow::Owned(s.into()),
            Self::Raw(bytes) => bytes,
        }
    }

    pub fn len(&self) -> usize {
        match self {
            Self::Str(cow) => cow.len(),
            Self::Raw(cow) => cow.len(),
        }
    }
}

pub trait InputRead<'r> {
    /// Peek a single byte.
    fn peek(&self) -> Option<u8>;

    /// Consume a single byte after peeking.
    fn discard(&mut self);

    /// Consume a comment.
    fn comment(&mut self);

    /// Consume junk characters between entries
    fn next_entry_or_eof(&mut self) -> bool;

    /// Take a unicode identifier.
    fn identifier_unicode(&mut self) -> Result<UnicodeIdentifier<'r>, ReadError>;

    /// Take an ascii identifier.
    fn identifier_ascii(&mut self) -> Result<AsciiIdentifier<'r>, ReadError>;

    fn balanced(&mut self) -> Result<Text<'r>, ReadError>;

    fn protected(&mut self, until: u8) -> Result<Text<'r>, ReadError>;

    /// Take a text number token.
    fn number(&mut self) -> Result<Text<'r>, ReadError>;
}
