//! This crate uses a large amount of unsafe for string conversions. The unsafe are valid since all
//! of the cuts performed in `super::slice_impl` are at ascii codepoints, so the resulting slices
//! are valid str if they began as valid str.
use super::slice_impl;
use super::{AsciiIdentifier, Text, UnicodeIdentifier};
use super::{Read, ReadError};
use std::borrow::Cow;
use std::str::from_utf8_unchecked;

use crate::parse::BibtexParse;

#[inline]
pub fn next_entry_or_eof(input: &str) -> (&str, bool) {
    let (bytes, res) = slice_impl::next_entry_or_eof(input.as_bytes());
    unsafe { (from_utf8_unchecked(bytes), res) }
}

#[inline]
pub fn comment(input: &str) -> &str {
    let bytes = slice_impl::comment(input.as_bytes());
    unsafe { from_utf8_unchecked(bytes) }
}

#[inline]
pub fn identifier_unicode(input: &str) -> Result<(&str, UnicodeIdentifier), ReadError> {
    let (bytes, res) = slice_impl::identifier_unicode(input.as_bytes())?;
    unsafe { Ok((from_utf8_unchecked(bytes), res)) }
}

#[inline]
pub fn identifier_ascii(input: &str) -> Result<(&str, AsciiIdentifier), ReadError> {
    let (bytes, res) = slice_impl::identifier_ascii(input.as_bytes())?;
    unsafe { Ok((from_utf8_unchecked(bytes), res)) }
}

#[inline]
pub fn number(input: &str) -> Result<(&str, Text), ReadError> {
    let (bytes, res) = slice_impl::number(input.as_bytes())?;
    unsafe { Ok((from_utf8_unchecked(bytes), res)) }
}

#[inline]
pub fn balanced(input: &str) -> Result<(&str, &str), ReadError> {
    let (bytes, res) = slice_impl::balanced(input.as_bytes())?;
    unsafe { Ok((from_utf8_unchecked(bytes), from_utf8_unchecked(res))) }
}

#[inline]
pub fn protected(until: u8) -> impl FnMut(&str) -> Result<(&str, &str), ReadError> {
    debug_assert!(until.is_ascii());
    move |input: &str| {
        let (bytes, res) = slice_impl::protected(until)(input.as_bytes())?;
        unsafe { Ok((from_utf8_unchecked(bytes), from_utf8_unchecked(res))) }
    }
}

super::create_input_impl::input_read_impl!(str, StrReader, Str, str::as_bytes);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_next_entry_or_eof() {
        assert_eq!(next_entry_or_eof("junk"), ("", false));
        assert_eq!(next_entry_or_eof(""), ("", false));
        assert_eq!(next_entry_or_eof("@art"), ("art", true));
        assert_eq!(next_entry_or_eof("%@@\n@a"), ("a", true));
        assert_eq!(next_entry_or_eof("\nignored @a"), ("a", true));
        assert_eq!(next_entry_or_eof("%@a"), ("", false));
    }

    #[test]
    fn test_comment() {
        assert_eq!(comment("%   a\n ab"), "ab");
        assert_eq!(comment("  %\na"), "a");
        // all valid whitespace chars
        assert_eq!(comment("\x09\x0a\x0c\x0d\x20b"), "b");
        // we follow whatwg convention and do not consider U+000B VERTICAL TAB
        // to be ascii whitespace, unlike biber
        assert_eq!(comment("\x0b"), "\x0b");
        assert_eq!(comment(""), "");
    }

    #[test]
    fn test_protected() {
        assert_eq!(
            protected(b'"')("captured🍄\"🍄rest"),
            Ok(("\"🍄rest", "captured🍄"))
        );
        assert_eq!(protected(b'"')("🍄{\"}\"🍄est"), Ok(("\"🍄est", "🍄{\"}")));
        assert_eq!(
            protected(b'"')("a{{\"} \"}\"🍄est"),
            Ok(("\"🍄est", "a{{\"} \"}"))
        );
        // did not find unprotected
        assert_eq!(protected(b'"')("{\""), Err(ReadError::Eof));
        // unexpected closing
        assert_eq!(protected(b'"')("}\""), Err(ReadError::Unbalanced));
    }

    #[test]
    fn test_balanced() {
        assert_eq!(balanced("url}🍄bc"), Ok(("}🍄bc", "url")));
        assert_eq!(balanced("u{}r🍄}🍄c"), Ok(("}🍄c", "u{}r🍄")));
        assert_eq!(balanced("u{{}}rl}abc"), Ok(("}abc", "u{{}}rl")));
        assert_eq!(balanced("u{{}}r{}l}🍄bc"), Ok(("}🍄bc", "u{{}}r{}l")));

        assert_eq!(balanced("none"), Err(ReadError::Eof));
        assert_eq!(balanced("{n🍄}e"), Err(ReadError::Eof));
    }

    use proptest::prelude::*;
    proptest! {
        #[test]
        fn no_panic(s in "\\PC*") {
            let _ = next_entry_or_eof(&s);
            let _ = comment(&s);
            let _ = identifier_unicode(&s);
            let _ = identifier_ascii(&s);
            let _ = number(&s);
            let _ = balanced(&s);
            let _ = protected(b'"')(&s);
            let _ = protected(b')')(&s);
        }
    }
}
