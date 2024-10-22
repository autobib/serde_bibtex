//! Warning: the `super::str_impl` module depends heavily on the implementation in this crate for
//! safety! All of the cuts must be performed either immediately before or after an ascii codepoint,
//! so the resulting slices are valid str if they began as valid str.
use super::Read;
use super::{Identifier, Text};
use memchr::{memchr2_iter, memchr3_iter};
use std::str::{from_utf8, from_utf8_unchecked};

use crate::{
    error::{Error, ErrorCode},
    parse::BibtexParse,
    token::IDENTIFIER_ALLOWED,
};

/// Ignore junk characters between entries.
///
/// Returns (updated_pos, true) if an entry was found; otherwise (input.len(), false) if hit EOF.
pub fn next_entry_or_eof(input: &[u8], mut pos: usize) -> (usize, bool) {
    while pos < input.len() {
        pos += 1;
        match input[pos - 1] {
            b'@' => return (pos, true),
            b'%' => {
                while pos < input.len() && input[pos] != b'\n' {
                    pos += 1;
                }
                if pos == input.len() {
                    return (pos, false);
                } else {
                    // found \n, skip it
                    pos += 1
                }
            }
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
                    pos += 1
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
        end += 1
    }

    if end == start {
        return Err(Error::syntax(ErrorCode::Empty));
    }

    let s = from_utf8(&input[start..end])?;
    Ok((end, Identifier(s)))
}

/// Consume a non-empty sequence of digits [0-9]+.
///
/// Since ascii digits are valid UTF-8, we can skip the validation step.
pub fn number(input: &[u8], start: usize) -> Result<(usize, &str), Error> {
    let mut end = start;

    while end < input.len() && input[end].is_ascii_digit() {
        end += 1
    }

    if end == start {
        return Err(Error::syntax(ErrorCode::Empty));
    }

    // SAFETY: we only parsed ascii digits so this is guaranteed to be
    // valid utf8.
    Ok((end, unsafe { from_utf8_unchecked(&input[start..end]) }))
}

/// Consume a string with balanced brackets, until the string becomes unbalanced.
pub fn balanced(input: &[u8], start: usize) -> Result<(usize, &[u8]), Error> {
    let mut bracket_depth = 0;

    for offset in memchr2_iter(b'{', b'}', &input[start..]) {
        let end = start + offset;
        if input[end] == b'{' {
            bracket_depth += 1
        } else {
            // found the closing bracket
            if bracket_depth == 0 {
                return Ok((end, &input[start..end]));
            }
            bracket_depth -= 1;
        }
    }

    // we did not find find the closing bracket
    Err(Error::syntax(ErrorCode::UnterminatedTextToken))
}

/// Consume a string with balanced brackets, terminating when we hit a top-level byte 'until'.
///
///SAFETY: for the string version, `until` must be valid ASCII.
pub fn protected(until: u8) -> impl FnMut(&[u8], usize) -> Result<(usize, &[u8]), Error> {
    move |input: &[u8], start: usize| {
        let mut bracket_depth = 0;

        for offset in memchr3_iter(until, b'{', b'}', &input[start..]) {
            let end = start + offset;
            match input[end] {
                b if b == until => {
                    if bracket_depth == 0 {
                        return Ok((end, &input[start..end]));
                    }
                }
                b'{' => bracket_depth += 1,
                _ => {
                    if bracket_depth == 0 {
                        return Err(Error::syntax(ErrorCode::UnexpectedClosingBracket));
                    }
                    bracket_depth -= 1;
                }
            }
        }

        // we did not find an unprotected `"`
        Err(Error::syntax(ErrorCode::UnterminatedTextToken))
    }
}

super::create_input_impl::read_impl!([u8], SliceReader, Bytes, std::convert::identity);

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
        assert!(matches!(protected(b'"')(b"cap\"rest", 0), Ok((3, b"cap"))));
        assert!(matches!(protected(b'"')(b"cap\"rest", 1), Ok((3, b"ap"))));
        assert!(matches!(
            protected(b'"')(b"a{\"}\"rest", 0),
            Ok((4, b"a{\"}"))
        ));
        assert!(matches!(
            protected(b'"')(b"a{{\"} \"}\"rest", 0),
            Ok((8, b"a{{\"} \"}"))
        ));
        // did not find unprotected
        assert!(matches!(
            protected(b'"')(b"{\"", 0),
            Err(Error {
                code: ErrorCode::UnterminatedTextToken
            })
        ));
        // unexpected closing
        assert!(matches!(
            protected(b'"')(b"}\"", 0),
            Err(Error {
                code: ErrorCode::UnexpectedClosingBracket
            })
        ));
    }

    #[test]
    fn test_balanced() {
        assert!(matches!(balanced(b"url}abc", 0), Ok((3, b"url"))));
        assert!(matches!(
            balanced("u{}rl}🍄c".as_bytes(), 0),
            Ok((5, b"u{}rl"))
        ));
        assert!(matches!(balanced(b"u{{}}rl}abc", 1), Ok((7, b"{{}}rl"))));

        assert!(matches!(
            balanced(b"none", 0),
            Err(Error {
                code: ErrorCode::UnterminatedTextToken
            })
        ));
        assert!(matches!(
            balanced(b"{no}e", 0),
            Err(Error {
                code: ErrorCode::UnterminatedTextToken
            })
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
