mod create_input_impl;
mod slice_impl;
mod str_impl;

pub use slice_impl::SliceReader;
pub use str_impl::StrReader;

use crate::error::Error;
use crate::token::{Identifier, Text};

/// A trait to represent a type which can be parsed as BibTeX.
///
/// This trait is implemented by [`SliceReader`] and [`StrReader`].
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
