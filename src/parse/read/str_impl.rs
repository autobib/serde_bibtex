//! Implementation of `StrReader`, esentially by using the `SliceReader` implementation with some
//! UTF-8 check skips.
//!
//! This module uses unsafe for string conversions. The unsafe are valid since all of the string slicing
//! performed in `super::slice_impl` is adjacent to ascii codepoints, so the resulting slices are valid
//! str if they began as valid str.
use super::Read;
use super::slice_impl;
use super::{Identifier, Text};
use crate::error::{Error, ErrorCode};
use crate::token::FieldKey;
use crate::token::IDENTIFIER_ALLOWED;
use crate::token::Token;
use std::str::from_utf8_unchecked;

use crate::parse::BibtexParse;

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
        end += 1
    }

    if end == start {
        return Err(Error::syntax(ErrorCode::Empty));
    }

    Ok((end, Identifier(unsafe { input.get_unchecked(start..end) })))
}

#[inline]
pub fn number(input: &str, pos: usize) -> Result<(usize, &str), Error> {
    slice_impl::number(input.as_bytes(), pos)
}

#[inline]
pub fn balanced(input: &str, pos: usize) -> Result<(usize, &str), Error> {
    let (new, res) = slice_impl::balanced(input.as_bytes(), pos)?;
    unsafe { Ok((new, from_utf8_unchecked(res))) }
}

#[inline]
pub fn protected(until: u8) -> impl FnMut(&str, usize) -> Result<(usize, &str), Error> {
    debug_assert!(until.is_ascii());
    move |input: &str, pos: usize| {
        let (new, res) = slice_impl::protected(until)(input.as_bytes(), pos)?;
        unsafe { Ok((new, from_utf8_unchecked(res))) }
    }
}

super::create_input_impl::read_impl!(
    /// A reader that can parse BibTeX from a string slice.
    ///
    /// This the same as a [`SliceReader`](crate::SliceReader), but is able to skip some
    /// UTF-8 checks.
    ///
    /// This struct also exposes a few internal parsing methods.
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
        match self.single_token()? {
            Token::Text(Text::Bytes(_)) | Token::Variable(_) => Err(crate::error::Error::syntax(
                crate::error::ErrorCode::ExpectedTextToken,
            )),
            Token::Text(Text::Str(text)) => Ok(text),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parse::ErrorCode;

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
        assert!(matches!(protected(b'"')("🍄\"🍄rest", 0), Ok((4, "🍄"))));
        assert!(matches!(
            protected(b'"')("🍄{\"}\"🍄est", 0),
            Ok((7, "🍄{\"}"))
        ));
    }

    #[test]
    fn test_balanced() {
        assert!(matches!(balanced("url}🍄bc", 0), Ok((3, "url"))));
        assert!(matches!(balanced("u{}r🍄}🍄c", 0), Ok((8, "u{}r🍄"))));

        assert!(matches!(
            balanced("none", 2),
            Err(Error {
                code: ErrorCode::UnterminatedTextToken
            })
        ));
        assert!(matches!(
            balanced("{n🍄}e", 0),
            Err(Error {
                code: ErrorCode::UnterminatedTextToken
            })
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
            let _ = balanced(&s, 0);
            let _ = protected(b'"')(&s, 0);
            let _ = protected(b')')(&s, 0);
        }
    }
}
