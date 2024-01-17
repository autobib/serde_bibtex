//! Warning: the `super::str_impl` crate depends heavily on the implementation in this crate for
//! safety! All of the cuts must be performed either immediately before or after an ascii codepoint,
//! so the resulting slices are valid str if they began as valid str.
use super::{AsciiIdentifier, Text, UnicodeIdentifier};
use super::{Read, ReadError};
use memchr::{memchr, memchr2, memchr2_iter, memchr3_iter};
use std::borrow::Cow;
use std::str::from_utf8;

use crate::parse::BibtexParse;

/// Ignore junk characters between entries.
pub fn next_entry_or_eof(input: &[u8]) -> (&[u8], bool) {
    let mut tail = input;

    loop {
        // Search for `@` or `%`.
        match memchr2(b'@', b'%', tail) {
            Some(idx) => {
                // realign to the matched index
                // SAFETY: adjacent to one of @ or %
                tail = &tail[idx..];

                if tail[0] == b'@' {
                    // we found an @, so we consume it and return
                    // SAFETY: adjacent to @
                    return (&tail[1..], true);
                } else {
                    // we found a %, so we skip to the the end of the line
                    match memchr(b'\n', tail) {
                        Some(idx) => {
                            // SAFETY: adjacent to \n
                            tail = &tail[idx + 1..];
                        }
                        None => return (&[], false),
                    }
                }
            }
            None => return (&[], false),
        }
    }
}

/// Ignore whitespace and comments within entries.
///
/// Note that this uses the built-in `.is_ascii_whitespace` and in particular
/// unlike biber does not consider U+000B VERTICAL TAB to be whitespace.
pub fn comment(input: &[u8]) -> &[u8] {
    let mut pos = 0;
    loop {
        if pos == input.len() {
            // SAFETY: empty slice
            return &input[pos..];
        }

        if input[pos].is_ascii_whitespace() {
            pos += 1;
        } else if input[pos] == b'%' {
            // SAFETY: adjacent to %
            match memchr(b'\n', &input[pos + 1..]) {
                Some(offset) => {
                    // alignment math: skip the '%' and the '\n'
                    pos += offset + 2;
                }
                // SAFETY: adjacent to \n
                None => return &input[pos..],
            }
        } else {
            // SAFETY: we did not return on a previous loop, so
            // either pos = 0 or one of the previous conditions is satisfied.
            return &input[pos..];
        }
    }
}

// Lookup table for bytes which could appear in an entry key. This includes the
// ascii printable characters with "{}(),= \t\n\\#%'\"" removed, as well as bytes
// that could appear in non-ascii utf8.
//
// Note that this table is insufficient for utf8 validation: it is only used for
// short-circuited termination of parsing!
static ENTRY_ALLOWED: [bool; 256] = {
    const PR: bool = false; // disallowed printable bytes
    const CT: bool = false; // non-printable ascii
    const __: bool = true; // permitted bytes
    [
        //   1   2   3   4   5   6   7   8   9   A   B   C   D   E   F
        CT, CT, CT, CT, CT, CT, CT, CT, CT, CT, CT, CT, CT, CT, CT, CT, // 0
        CT, CT, CT, CT, CT, CT, CT, CT, CT, CT, CT, CT, CT, CT, CT, CT, // 1
        CT, __, PR, PR, __, PR, __, __, PR, PR, __, __, PR, __, __, __, // 2
        __, __, __, __, __, __, __, __, __, __, __, __, __, PR, __, __, // 3
        __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // 4
        __, __, __, __, __, __, __, __, __, __, __, __, PR, __, __, __, // 5
        __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // 6
        __, __, __, __, __, __, __, __, __, __, __, PR, __, PR, __, CT, // 7
        __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // 8
        __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // 9
        __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // A
        __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // B
        __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // C
        __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // D
        __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // E
        __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // F
    ]
};

/// An ascii identifier is an ascii printable character permitted by [`ENTRY_ALLOWED`].
#[inline]
pub fn is_ascii_identifier_byte(b: u8) -> bool {
    ENTRY_ALLOWED[b as usize] && matches!(b, b'\x21'..=b'\x7e')
}

/// Consume until we hit a disallowed character, and then perform utf8 validation.
pub fn identifier_unicode(b: &[u8]) -> Result<(&[u8], UnicodeIdentifier), ReadError> {
    let mut pos = 0;

    if b.is_empty() {
        return Err(ReadError::Eof);
    }

    while pos < b.len() && ENTRY_ALLOWED[b[pos] as usize] {
        pos += 1
    }

    if pos == 0 {
        return Err(ReadError::ExpectedIdentifier);
    }

    match from_utf8(&b[..pos]) {
        Ok(s) => Ok((&b[pos..], UnicodeIdentifier(Cow::Borrowed(s)))),
        Err(why) => Err(ReadError::InvalidUtf8(why)),
    }
}

pub fn identifier_ascii(b: &[u8]) -> Result<(&[u8], AsciiIdentifier), ReadError> {
    let mut pos = 0;

    while pos < b.len() && is_ascii_identifier_byte(b[pos]) {
        pos += 1
    }

    if pos == 0 {
        return Err(ReadError::ExpectedIdentifier);
    }

    // SAFETY: if `valid_ascii(b[pos])` returns true, then b[pos]
    // must be an ascii printable character and therefore valid utf8.
    Ok((
        &b[pos..],
        AsciiIdentifier(Cow::Borrowed(from_utf8(&b[..pos]).unwrap())),
    ))
}

pub fn number(input: &[u8]) -> Result<(&[u8], Text), ReadError> {
    let mut pos = 0;

    if input.is_empty() {
        return Err(ReadError::Eof);
    }

    while pos < input.len() && input[pos].is_ascii_digit() {
        pos += 1
    }

    if pos == 0 {
        return Err(ReadError::ExpectedIdentifier);
    }

    // SAFETY: we only parsed ascii digits so this is guaranteed to be
    // valid utf8.
    Ok((
        &input[pos..],
        Text::Str(Cow::Borrowed(from_utf8(&input[..pos]).unwrap())),
    ))
}

pub fn balanced(input: &[u8]) -> Result<(&[u8], &[u8]), ReadError> {
    let mut bracket_depth = 0;

    for idx in memchr2_iter(b'{', b'}', input) {
        if input[idx] == b'{' {
            bracket_depth += 1
        } else {
            // found the closing bracket
            if bracket_depth == 0 {
                return Ok((&input[idx..], &input[0..idx]));
            }
            bracket_depth -= 1;
        }
    }

    // we did not find find the closing bracket
    Err(ReadError::Eof)
}

/// SAFETY: for the string version, `until` must be valid ASCII.
pub fn protected(until: u8) -> impl FnMut(&[u8]) -> Result<(&[u8], &[u8]), ReadError> {
    move |input: &[u8]| {
        let mut bracket_depth = 0;

        for idx in memchr3_iter(until, b'{', b'}', input) {
            match input[idx] {
                b if b == until => {
                    if bracket_depth == 0 {
                        return Ok((&input[idx..], &input[..idx]));
                    }
                }
                b'{' => bracket_depth += 1,
                _ => {
                    if bracket_depth == 0 {
                        return Err(ReadError::Unbalanced);
                    }
                    bracket_depth -= 1;
                }
            }
        }

        // we did not find an unprotected `"`
        Err(ReadError::Eof)
    }
}

super::create_input_impl::input_read_impl!([u8], SliceReader, Raw, std::convert::identity);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_next_entry_or_eof() {
        assert_eq!(next_entry_or_eof(b"junk"), (&b""[..], false));
        assert_eq!(next_entry_or_eof(b""), (&b""[..], false));
        assert_eq!(next_entry_or_eof(b"@art"), (&b"art"[..], true));
        assert_eq!(next_entry_or_eof(b"%@@\n@a"), (&b"a"[..], true));
        assert_eq!(next_entry_or_eof(b"\nignored @a"), (&b"a"[..], true));
        assert_eq!(next_entry_or_eof(b"%@a"), (&b""[..], false));
    }

    #[test]
    fn test_comment() {
        assert_eq!(comment(b"%   a\n ab"), &b"ab"[..]);
        assert_eq!(comment(b"  %\na"), &b"a"[..]);
        // all valid whitespace chars
        assert_eq!(comment(b"\x09\x0a\x0c\x0d\x20b"), &b"b"[..]);
        // comments ignore everything, including invalid utf-8
        assert_eq!(comment(b"%\xa8!\xfd!\x7f!\nc"), &b"c"[..]);
        // we follow whatwg convention and do not consider U+000B VERTICAL TAB
        // to be ascii whitespace, unlike biber
        assert_eq!(comment(b"\x0b"), &b"\x0b"[..]);
        assert_eq!(comment(b""), &b""[..]);
    }

    #[test]
    fn test_is_ascii_identifier_byte() {
        assert!(is_ascii_identifier_byte(b'5'));
        assert!(is_ascii_identifier_byte(b'b'));
        assert!(is_ascii_identifier_byte(b'@'));
        assert!(!is_ascii_identifier_byte(b'{'));
        assert!(!is_ascii_identifier_byte(b' '));
        assert!(!is_ascii_identifier_byte(b'\t'));
        assert!(!is_ascii_identifier_byte(b'}'));
        assert!(!is_ascii_identifier_byte(b')'));
        assert!(!is_ascii_identifier_byte(b'('));
    }

    #[test]
    fn test_protected() {
        assert_eq!(
            protected(b'"')(b"captured\"rest"),
            Ok((&b"\"rest"[..], &b"captured"[..]))
        );
        assert_eq!(
            protected(b'"')(b"a{\"}\"rest"),
            Ok((&b"\"rest"[..], &b"a{\"}"[..]))
        );
        assert_eq!(
            protected(b'"')(b"a{{\"} \"}\"rest"),
            Ok((&b"\"rest"[..], &b"a{{\"} \"}"[..]))
        );
        // did not find unprotected
        assert_eq!(protected(b'"')(b"{\""), Err(ReadError::Eof));
        // unexpected closing
        assert_eq!(protected(b'"')(b"}\""), Err(ReadError::Unbalanced));
    }

    #[test]
    fn test_balanced() {
        assert_eq!(balanced(b"url}abc"), Ok((&b"}abc"[..], &b"url"[..])));
        assert_eq!(
            balanced("u{}rl}🍄c".as_bytes()),
            Ok(("}🍄c".as_bytes(), &b"u{}rl"[..]))
        );
        assert_eq!(
            balanced(b"u{{}}rl}abc"),
            Ok((&b"}abc"[..], &b"u{{}}rl"[..]))
        );
        assert_eq!(
            balanced(b"u{{}}r{}l}abc"),
            Ok((&b"}abc"[..], &b"u{{}}r{}l"[..]))
        );

        assert_eq!(balanced(b"none"), Err(ReadError::Eof));
        assert_eq!(balanced(b"{no}e"), Err(ReadError::Eof));
    }

    use proptest::prelude::*;
    proptest! {
        #[test]
        fn no_panic(s in "\\PC*") {
            let _ = identifier_ascii(s.as_bytes());
            let _ = number(s.as_bytes());
        }
    }
}
