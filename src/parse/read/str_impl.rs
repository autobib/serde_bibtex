//! Implementation of `StrReader`, esentially by using the `SliceReader` implementation with some
//! UTF-8 check skips.
//!
//! This module uses unsafe for string conversions. The unsafe are valid since all of the string slicing
//! performed in `super::slice_impl` is adjacent to ascii codepoints, so the resulting slices are valid
//! str if they began as valid str.
use core::str::from_utf8_unchecked;

use super::{BibtexRead, BibtexReadInner, TextDelimiter, slice_impl};
use crate::{
    error::Error,
    token::{FieldKey, IDENTIFIER_ALLOWED, Identifier, Text, Token},
};

fn error_span(input: &str, start: usize) -> core::ops::Range<usize> {
    debug_assert!(input.is_char_boundary(start));
    let mut end = start + usize::from(start < input.len());
    while !input.is_char_boundary(end) {
        end += 1;
    }
    start..end
}

#[inline]
pub fn next_entry_or_eof(input: &str, pos: usize) -> (usize, bool) {
    slice_impl::next_entry_or_eof(input.as_bytes(), pos)
}

#[inline]
pub fn comment(input: &str, pos: usize) -> usize {
    slice_impl::comment(input.as_bytes(), pos)
}

#[inline]
pub fn identifier(input: &str, start: usize) -> Result<(usize, Identifier<&str>), Error> {
    let mut end = start;

    while end < input.len() && IDENTIFIER_ALLOWED[input.as_bytes()[end] as usize] {
        end += 1;
    }

    if end == start {
        return Err(Error::expected(
            "identifier",
            input.as_bytes().get(start).copied(),
        ));
    }

    Ok((end, Identifier(unsafe { input.get_unchecked(start..end) })))
}

#[inline]
pub fn number(input: &str, pos: usize) -> Result<(usize, &str), Error> {
    slice_impl::number(input.as_bytes(), pos)
}

#[inline]
pub fn text_until(
    delimiter: TextDelimiter,
) -> impl FnMut(&str, usize) -> Result<(usize, &str), Error> {
    move |input: &str, pos: usize| {
        let (new, res) = slice_impl::text_until(delimiter)(input.as_bytes(), pos)?;
        // SAFETY: the scanner starts at a character boundary and ends at an ASCII delimiter.
        unsafe { Ok((new, from_utf8_unchecked(res))) }
    }
}

super::create_input_impl::read_impl!(
    /// A reader that can parse BibTeX from a string slice.
    ///
    /// This is the same as a [`SliceReader`](crate::SliceReader), but is able to skip some
    /// UTF-8 checks.
    ///
    /// Use [`Self::read_field_key`], [`Self::skip_field_sep`], and [`Self::read_text_token`]
    /// to parse individual field keys, separators, and text tokens.
    ///
    /// ```
    /// use serde_bibtex::StrReader;
    ///
    /// let mut reader = StrReader::new(" % comment\n title = {hé{llo}}");
    /// assert_eq!(reader.read_field_key()?.into_inner(), "title");
    /// reader.skip_field_sep()?;
    /// assert_eq!(reader.read_text_token()?, "hé{llo}");
    /// # Ok::<(), serde_bibtex::Error>(())
    /// ```
    #[derive(Debug, Clone)]
    pub struct StrReader<'r>(&'r str);

    Str;

    str::as_bytes;
);

impl<'r> StrReader<'r> {
    /// Read a field key.
    pub fn read_field_key(&mut self) -> crate::error::Result<FieldKey<&'r str>> {
        self.comment();
        self.identifier().map(Into::into)
    }

    /// Skip a field separator `=`.
    pub fn skip_field_sep(&mut self) -> crate::error::Result<()> {
        self.field_sep()
    }

    /// Read a single text token, which is one of `{text}`, `"text"`, or `01234`.
    pub fn read_text_token(&mut self) -> crate::error::Result<&'r str> {
        self.comment();
        let start = self.pos;
        match self.single_token()? {
            Token::Text(Text::Bytes(_)) | Token::Variable(_) => Err(crate::error::Error::new(
                crate::error::ErrorCode::ExpectedTextToken,
            )
            .with_span(Some(start..self.pos))),
            Token::Text(Text::Str(text)) => Ok(text),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::ErrorCode;

    #[test]
    fn test_next_entry_or_eof() {
        assert_eq!(next_entry_or_eof("junk", 0), (4, false));
        assert_eq!(next_entry_or_eof("", 0), (0, false));
        assert_eq!(next_entry_or_eof("@art", 0), (1, true));
        assert_eq!(next_entry_or_eof("%@@\n@a", 0), (5, true));
        assert_eq!(next_entry_or_eof("\nignored @a", 0), (10, true));
        assert_eq!(next_entry_or_eof("%@a", 0), (3, false));
    }

    #[test]
    fn test_comment() {
        assert_eq!(comment("%   a\n ab", 0), 7);
    }

    #[test]
    fn test_protected() {
        assert!(matches!(
            text_until(TextDelimiter::Quote)("🍄\"🍄rest", 0),
            Ok((4, "🍄"))
        ));
        assert!(matches!(
            text_until(TextDelimiter::Quote)("🍄{\"}\"🍄est", 0),
            Ok((7, "🍄{\"}"))
        ));
    }

    #[test]
    fn test_balanced() {
        assert!(matches!(
            text_until(TextDelimiter::Brace)("url}🍄bc", 0),
            Ok((3, "url"))
        ));
        assert!(matches!(
            text_until(TextDelimiter::Brace)("u{}r🍄}🍄c", 0),
            Ok((8, "u{}r🍄"))
        ));

        assert!(matches!(
            text_until(TextDelimiter::Brace)("none", 2),
            Err(ref err) if matches!(err.inner_code(), ErrorCode::UnclosedDelimiter(b'{'))
        ));
        assert!(matches!(
            text_until(TextDelimiter::Brace)("{n🍄}e", 0),
            Err(ref err) if matches!(err.inner_code(), ErrorCode::UnclosedDelimiter(b'{'))
        ));
    }

    use proptest::prelude::*;
    proptest! {
        #[test]
        fn no_panic(s in "\\PC*") {
            let _ = next_entry_or_eof(&s, 0);
            let _ = comment(&s, 0);
            let _ = identifier(&s, 0);
            let _ = number(&s, 0);
            let _ = text_until(TextDelimiter::Brace)(&s, 0);
            let _ = text_until(TextDelimiter::Quote)(&s, 0);
            let _ = text_until(TextDelimiter::Parenthesis)(&s, 0);
        }
    }
}
