//! Warning: the `super::str_impl` module depends heavily on the implementation in this crate for
//! safety! All of the cuts must be performed either immediately before or after an ascii codepoint,
//! so the resulting slices are valid str if they began as valid str.
use core::str::{from_utf8, from_utf8_unchecked};

use memchr::{memchr2_iter, memchr3_iter};

use super::{BibtexRead, BibtexReadInner, TextDelimiter};
use crate::{
    error::{Error, ErrorCode},
    token::{IDENTIFIER_ALLOWED, Identifier, Text},
};

fn error_span(input: &[u8], start: usize) -> core::ops::Range<usize> {
    start..start + usize::from(start < input.len())
}

/// Ignore junk characters between entries.
///
/// Returns (updated_pos, true) if an entry was found; otherwise (input.len(), false) if hit EOF.
pub fn next_entry_or_eof(input: &[u8], pos: usize) -> (usize, bool) {
    let mut inside_comment = false;

    for offset in memchr3_iter(b'@', b'%', b'\n', &input[pos..]) {
        let idx = pos + offset;
        match (input[idx], inside_comment) {
            (b'\n', true) => inside_comment = false,
            (b'@', false) => return (idx + 1, true),
            (b'%', false) => inside_comment = true,
            _ => {}
        }
    }
    (input.len(), false)
}

/// Ignore whitespace and comments within entries.
///
/// Note that this follows the same convention as the built-in `u8::is_ascii_whitespace`
/// and in particular unlike biber does not consider U+000B VERTICAL TAB to be whitespace.
pub fn comment(input: &[u8], mut pos: usize) -> usize {
    while pos < input.len() {
        match input[pos] {
            // ASCII whitespace
            b'\t' | b'\n' | b'\x0C' | b'\r' | b' ' => pos += 1,
            // begin comment
            b'%' => {
                pos += 1;
                while pos < input.len() && input[pos] != b'\n' {
                    pos += 1;
                }
                if pos == input.len() {
                    return pos;
                } else {
                    // found \n, skip it
                    pos += 1;
                }
            }
            _ => return pos,
        }
    }
    input.len()
}

/// Consume until we hit a disallowed character, and then perform UTF-8 validation.
pub fn identifier(input: &[u8], start: usize) -> Result<(usize, Identifier<&str>), Error> {
    let mut end = start;

    while end < input.len() && IDENTIFIER_ALLOWED[input[end] as usize] {
        end += 1;
    }

    if end == start {
        return Err(Error::expected("identifier", input.get(start).copied()));
    }

    let bytes = &input[start..end];
    let s = from_utf8(bytes).map_err(|err| Error::utf8(err).with_utf8_span(Some(start..end)))?;
    Ok((end, Identifier(s)))
}

/// Consume a non-empty sequence of digits [0-9]+.
///
/// Since ascii digits are valid UTF-8, we can skip the validation step.
pub fn number(input: &[u8], start: usize) -> Result<(usize, &str), Error> {
    let mut end = start;

    while end < input.len() && input[end].is_ascii_digit() {
        end += 1;
    }

    if end == start {
        return Err(Error::expected("number", input.get(start).copied()));
    }

    // SAFETY: we only parsed ascii digits so this is guaranteed to be
    // valid utf8.
    Ok((end, unsafe { from_utf8_unchecked(&input[start..end]) }))
}

/// Read text up to a delimiter outside balanced curly braces.
pub fn text_until(
    delimiter: TextDelimiter,
) -> impl FnMut(&[u8], usize) -> Result<(usize, &[u8]), Error> {
    move |input: &[u8], start: usize| {
        let mut bracket_depth = 0;

        for offset in memchr3_iter(delimiter as u8, b'{', b'}', &input[start..]) {
            let end = start + offset;
            let byte = input[end];
            if byte == delimiter as u8 && bracket_depth == 0 {
                return Ok((end, &input[start..end]));
            }
            match byte {
                b'{' => bracket_depth += 1,
                b'}' => {
                    if bracket_depth == 0 {
                        return Err(Error::new(ErrorCode::UnexpectedClosingBracket)
                            .with_span(Some(error_span(input, end))));
                    }
                    bracket_depth -= 1;
                }
                _ => {}
            }
        }

        // A nested brace is more specific than the outer delimiter.
        let opening = if bracket_depth > 0 {
            b'{'
        } else {
            match delimiter {
                TextDelimiter::Brace => b'{',
                TextDelimiter::Quote => b'"',
                TextDelimiter::Parenthesis => b'(',
            }
        };
        Err(unclosed(input, start, opening, bracket_depth))
    }
}

// Produce better failure spans by iterating backwards to recover the innermost unmatched brace.
fn unclosed(input: &[u8], start: usize, opening: u8, depth: usize) -> Error {
    let span = if depth > 0 {
        let mut closed = 0;
        let offset = memchr2_iter(b'{', b'}', &input[start..])
            .rev()
            .find(|&offset| {
                if input[start + offset] == b'}' {
                    closed += 1;
                    false
                } else if closed > 0 {
                    closed -= 1;
                    false
                } else {
                    true
                }
            })
            .expect("positive brace depth implies an unmatched opener");
        start + offset..start + offset + 1
    } else if start > 0 && input[start - 1] == opening {
        start - 1..start
    } else {
        start..start
    };
    Error::new(ErrorCode::UnclosedDelimiter(opening)).with_span(Some(span))
}

super::create_input_impl::read_impl!(
    /// A reader that can parse BibTeX from a slice.
    ///
    /// If you have a string, you should use a [`StrReader`](crate::StrReader) to skip some UTF-8
    /// checks.
    #[derive(Debug, Clone)]
    pub struct SliceReader<'r>(&'r [u8]);

    Bytes;

    core::convert::identity;
);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_next_entry_or_eof() {
        assert_eq!(next_entry_or_eof(b"junk", 0), (4, false));
        assert_eq!(next_entry_or_eof(b"junk", 2), (4, false));
        assert_eq!(next_entry_or_eof(b"", 0), (0, false));
        assert_eq!(next_entry_or_eof(b"  @art", 2), (3, true));
        assert_eq!(next_entry_or_eof(b"%@@\n@a", 0), (5, true));
        assert_eq!(next_entry_or_eof(b"\nignored @a", 0), (10, true));
        assert_eq!(next_entry_or_eof(b"%@a", 0), (3, false));
    }

    #[test]
    fn test_comment() {
        assert_eq!(comment(b"%   a\n ab", 0), 7);
        assert_eq!(comment(b"%   a\n ab", 1), 4);
        assert_eq!(comment(b"  %\na", 1), 4);
        // all valid whitespace chars
        assert_eq!(comment(b"\x09\x0a\x0c\x0d\x20b", 0), 5);
        assert_eq!(comment(b"\x09\x0a\x0c\x0d\x20b", 2), 5);
        // comments ignore everything, including invalid utf-8
        assert_eq!(comment(b"%\xa8!\xfd!\x7f!\nc", 0), 8);
        // we follow whatwg convention and do not consider U+000B VERTICAL TAB
        // to be ascii whitespace, unlike biber
        assert_eq!(comment(b"\x0b", 0), 0);
        assert_eq!(comment(b"", 0), 0);
    }

    #[test]
    fn test_protected() {
        assert!(matches!(
            text_until(TextDelimiter::Quote)(b"cap\"rest", 0),
            Ok((3, b"cap"))
        ));
        assert!(matches!(
            text_until(TextDelimiter::Quote)(b"cap\"rest", 1),
            Ok((3, b"ap"))
        ));
        assert!(matches!(
            text_until(TextDelimiter::Quote)(b"a{\"}\"rest", 0),
            Ok((4, b"a{\"}"))
        ));
        assert!(matches!(
            text_until(TextDelimiter::Quote)(b"a{{\"} \"}\"rest", 0),
            Ok((8, b"a{{\"} \"}"))
        ));
        // did not find unprotected
        assert!(matches!(
            text_until(TextDelimiter::Quote)(b"{\"", 0),
            Err(ref err) if matches!(err.inner_code(), ErrorCode::UnclosedDelimiter(b'{'))
        ));
        // unexpected closing
        assert!(matches!(
            text_until(TextDelimiter::Quote)(b"}\"", 0),
            Err(ref err) if matches!(err.inner_code(), ErrorCode::UnexpectedClosingBracket)
        ));
    }

    #[test]
    fn test_balanced() {
        assert!(matches!(
            text_until(TextDelimiter::Brace)(b"url}abc", 0),
            Ok((3, b"url"))
        ));
        assert!(matches!(
            text_until(TextDelimiter::Brace)("u{}rl}🍄c".as_bytes(), 0),
            Ok((5, b"u{}rl"))
        ));
        assert!(matches!(
            text_until(TextDelimiter::Brace)(b"u{{}}rl}abc", 1),
            Ok((7, b"{{}}rl"))
        ));

        assert!(matches!(
            text_until(TextDelimiter::Brace)(b"none", 0),
            Err(ref err) if matches!(err.inner_code(), ErrorCode::UnclosedDelimiter(b'{'))
        ));
        assert!(matches!(
            text_until(TextDelimiter::Brace)(b"{no}e", 0),
            Err(ref err) if matches!(err.inner_code(), ErrorCode::UnclosedDelimiter(b'{'))
        ));
    }

    use proptest::prelude::*;
    proptest! {
        #[test]
        fn no_panic(s in "\\PC*") {
            let _ = number(s.as_bytes(), 0);
        }
    }
}
