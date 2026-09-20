mod create_input_impl;
mod inner;
mod slice_impl;
mod str_impl;

#[cfg(test)]
mod tests;

pub use self::{slice_impl::SliceReader, str_impl::StrReader};
pub(crate) use inner::BibtexReadInner;

/// Valid delimiters for an entry body.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum EntryDelimiter {
    /// An entry delimited by `{` and `}`.
    Brace = b'}',
    /// An entry delimited by `(` and `)`.
    Parenthesis = b')',
}

impl EntryDelimiter {
    pub(crate) fn opening(self) -> u8 {
        match self {
            Self::Brace => b'{',
            Self::Parenthesis => b'(',
        }
    }
}

/// A delimiter terminating text outside balanced curly braces.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum TextDelimiter {
    /// A closing brace terminating braced text.
    Brace = b'}',
    /// A double quote terminating a quoted text token.
    Quote = b'"',
    /// A closing parenthesis terminating a comment entry.
    Parenthesis = b')',
}

/// Types which can be driven by a [`Deserializer`](crate::de::Deserializer) to read BibTeX input.
///
/// This trait is implemented by [`SliceReader`], [`StrReader`], and mutable references to
/// these readers. Use them with [`Deserializer::new`](crate::de::Deserializer::new).
///
/// This trait is sealed and cannot be implemented for types outside of `serde_bibtex`.
/// The reader methods are private; [`StrReader`] provides helpers for parsing individual
/// field keys, separators, and text tokens.
///
/// ```compile_fail
/// use serde_bibtex::BibtexRead;
///
/// struct CustomReader;
/// impl<'r> BibtexRead<'r> for CustomReader {}
/// ```
///
/// Importing this trait does not expose the internal reader primitives:
///
/// ```compile_fail
/// use serde_bibtex::{BibtexRead, StrReader};
///
/// let mut reader = StrReader::new("text");
/// reader.identifier();
/// ```
// The private supertrait seals this trait and keeps its methods out of the public API.
#[allow(private_bounds)]
pub trait BibtexRead<'r>: BibtexReadInner<'r> {}

impl<'r, R: BibtexRead<'r> + ?Sized> BibtexRead<'r> for &mut R {}
