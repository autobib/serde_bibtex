mod create_input_impl;
mod slice_impl;
mod str_impl;

pub use slice_impl::SliceReader;
pub use str_impl::StrReader;

use crate::error::Error;

#[derive(Debug)]
pub struct Identifier<S: AsRef<str>>(pub S);

/// The core Text handling object.
#[derive(Debug, Clone, PartialEq)]
pub enum Text<S: AsRef<str>, B: AsRef<[u8]>> {
    Str(S),
    Bytes(B),
}

impl<S, B> Text<S, B>
where
    S: AsRef<str>,
    B: AsRef<[u8]>,
{
    pub fn own(&self) -> Text<String, Vec<u8>> {
        match self {
            Text::Str(s) => Text::Str(s.as_ref().to_string()),
            Text::Bytes(b) => Text::Bytes(b.as_ref().to_vec()),
        }
    }
}

impl<'r> Text<&'r str, &'r [u8]> {
    pub fn into_str(self) -> Result<&'r str, Error> {
        match self {
            Self::Str(s) => Ok(s),
            Self::Bytes(b) => Ok(std::str::from_utf8(b)?),
        }
    }

    pub fn into_bytes(self) -> &'r [u8] {
        match self {
            Self::Str(s) => s.as_bytes(),
            Self::Bytes(bytes) => bytes,
        }
    }
}

impl<S: AsRef<str>, B: AsRef<[u8]>> Text<S, B> {
    pub fn len(&self) -> usize {
        match self {
            Self::Str(s) => s.as_ref().len(),
            Self::Bytes(b) => b.as_ref().len(),
        }
    }
}

pub trait Read<'r> {
    /// Peek a single byte.
    fn peek(&self) -> Option<u8>;

    /// Discard a single byte. This is only valid after a previous .peek() returned a value!
    fn discard(&mut self);

    /// Discard comments and whitespace.
    fn comment(&mut self);

    /// Discard junk characters between entries, and return true if another entry is found and
    /// false otherwise.
    fn next_entry_or_eof(&mut self) -> bool;

    /// Parse a unicode identifier.
    fn identifier(&mut self) -> Result<Identifier<&'r str>, Error>;

    /// Parse a balanced text token.
    fn balanced(&mut self) -> Result<Text<&'r str, &'r [u8]>, Error>;

    /// Parse a quoted or bracketed text token.
    fn protected(&mut self, until: u8) -> Result<Text<&'r str, &'r [u8]>, Error>;

    /// Parse a text number token.
    fn number(&mut self) -> Result<&'r str, Error>;
}
