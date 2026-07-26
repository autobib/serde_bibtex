mod create_input_impl;
mod slice_impl;
mod str_impl;

pub use slice_impl::SliceReader;
pub use str_impl::StrReader;

use crate::error::Error;
use crate::token::{Identifier, Text};

/// A pull parser which can be driven by a [`Deserializer`](crate::de::Deserializer) to parse BibTeX.
///
/// This trait is implemented by [`SliceReader`] and [`StrReader`].
pub trait BibtexRead<'r> {
    /// Peek the next byte in the input without advancing the position.
    fn peek(&self) -> Option<u8>;

    /// Advance forward a single byte, assuming that there are remaining bytes.
    ///
    /// Implementors may assume that a previous call to [`peek`](Self::peek) returned something
    /// and no other methods were call in between.
    fn discard(&mut self);

    /// Advance forward over comments and whitespace.
    fn comment(&mut self);

    /// Advance forward until the beginning of an entry is found, or the end of the file is reached,
    /// returning if an entry was found.
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
