use unicase::{Ascii, UniCase};

use nom::{Err, IResult};

use std::str::from_utf8;

#[derive(Debug, PartialEq)]
pub enum TokenParseError<'r> {
    InvalidUtf8(&'r [u8]),
    ZeroLength,
    UnexpectedEof,
    UnmatchedClosingBracket,
    FieldKeyStartsWithDigit(char),
}

pub const DISALLOWED_PRINTABLE_CHARS: &'static str = "{}(),=\\#%'\"";

/// Entry type, such as `article` in `@article{...`.
///
/// Rules:
/// 1. Case-insensitive.
/// 2. Any valid non-ASCII UTF-8, or ASCII chars in the printable range `b'\x21'..=b'\x7e'` which
///    are not one of [`DISALLOWED_PRINTABLE_CHARS`].
pub struct EntryType<S: AsRef<str>>(pub UniCase<S>);

impl<S: AsRef<str>> EntryType<S> {
    pub fn new(s: S) -> Self {
        Self(UniCase::new(s))
    }
}

/// Macro variable, such as `var` in `@string{var = ...}`.
///
/// Rules:
/// 1. Case-insensitive.
/// 2. Any valid non-ASCII UTF-8, or ASCII chars in the printable range `b'\x21'..=b'\x7e'` which
///    are not one of [`DISALLOWED_PRINTABLE_CHARS`].
pub struct Variable<S: AsRef<str>>(pub UniCase<S>);

impl<S: AsRef<str>> Variable<S> {
    pub fn new(s: S) -> Self {
        Self(UniCase::new(s))
    }
}

/// Entry key, such as `key` in `@article{key,....`.
///
/// Rules:
/// 1. Case-sensitive.
/// 2. Any valid non-ASCII UTF-8, or ASCII chars in the printable range `b'\x21'..=b'\x7e'` which
///    are not one of [`DISALLOWED_PRINTABLE_CHARS`].
#[derive(Debug, PartialEq, Eq, Hash, Copy, Clone)]
pub struct EntryKey<S: AsRef<str>>(pub S);

impl<S: AsRef<str>> EntryKey<S> {
    pub fn new(s: S) -> Self {
        Self(s)
    }
}

/// Field key, such as `key` in `... key = {value}, ...`.
///
/// Rules:
/// 1. Case-insensitive.
/// 2. Any valid printable ASCII except characters in [`DISALLOWED_CHARS`].
/// 2. ASCII chars in the printable range `b'\x21'..=b'\x7e'` which are not one of
/// [`DISALLOWED_PRINTABLE_CHARS`].
pub struct FieldKey<S: AsRef<str>>(pub Ascii<S>);

impl<S: AsRef<str>> FieldKey<S> {
    pub fn new(s: S) -> Self {
        Self(Ascii::new(s))
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

/// To capture from bytes, we consume until we hit a disallowed character, and then perform utf8
/// validation.
pub fn take_entry_unicode_bytes(b: &[u8]) -> IResult<&[u8], &str, TokenParseError> {
    let mut pos = 0;

    if b.is_empty() {
        return Err(Err::Error(TokenParseError::UnexpectedEof));
    }

    while pos < b.len() && ENTRY_ALLOWED[b[pos] as usize] {
        pos += 1
    }

    if pos == 0 {
        return Err(Err::Error(TokenParseError::ZeroLength));
    }

    match from_utf8(&b[..pos]) {
        Ok(s) => Ok((&b[pos..], s)),
        _ => Err(Err::Failure(TokenParseError::InvalidUtf8(&b[..pos]))),
    }
}

/// To capture from str, since every disallowed byte is either ascii or a byte which cannot
/// appear anywhere in a valid str, we do not need to perform the validation check as before.
pub fn take_entry_unicode(s: &str) -> IResult<&str, &str, TokenParseError> {
    let b = s.as_bytes();

    let mut pos = 0;

    if b.is_empty() {
        return Err(Err::Error(TokenParseError::UnexpectedEof));
    }

    while pos < b.len() && ENTRY_ALLOWED[b[pos] as usize] {
        pos += 1
    }

    if pos == 0 {
        return Err(Err::Error(TokenParseError::ZeroLength));
    }

    // SAFETY: If pos != b.len(), then b[pos] is a disallowed character. Since b was originally
    // valid utf8, b[pos] must in fact be an ascii character and therefore a 1-byte utf8 codepoint.
    // Thus the slices b[pos..] and b[..pos] are both utf8.
    Ok((from_utf8(&b[pos..]).unwrap(), from_utf8(&b[..pos]).unwrap()))
}

/// A valid field key can only be an ascii printable character.
#[inline]
pub fn valid_field_key(b: u8) -> bool {
    ENTRY_ALLOWED[b as usize] && matches!(b, b'\x21'..=b'\x7e')
}

pub fn take_entry_ascii_bytes(b: &[u8]) -> IResult<&[u8], &str, TokenParseError> {
    let mut pos = 0;

    if b.is_empty() {
        return Err(Err::Error(TokenParseError::UnexpectedEof));
    } else if b[0].is_ascii_digit() {
        return Err(Err::Error(TokenParseError::FieldKeyStartsWithDigit(
            b[0] as char,
        )));
    } else {
        while pos < b.len() && valid_field_key(b[pos]) {
            pos += 1
        }

        if pos == 0 {
            return Err(Err::Error(TokenParseError::ZeroLength));
        }

        // SAFETY: if `valid_field_key(b[pos])` returns true, then b[pos]
        // must be an ascii printable character and therefore valid utf8.
        Ok((&b[pos..], from_utf8(&b[..pos]).unwrap()))
    }
}

pub fn take_entry_ascii(s: &str) -> IResult<&str, &str, TokenParseError> {
    let (rest, captured) = take_entry_ascii_bytes(s.as_bytes())?;
    // SAFETY: since captured is a valid str, so is `rest`.
    Ok((from_utf8(rest).unwrap(), captured))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_take_entry_key_bytes() {
        assert_eq!(take_entry_unicode_bytes(b"abc"), Ok((&b""[..], "abc")));
        assert_eq!(take_entry_unicode_bytes(b"0@u04"), Ok((&b""[..], "0@u04")));
        assert_eq!(
            take_entry_unicode_bytes(b"12\x11"),
            Ok((&b"\x11"[..], "12"))
        );
        assert_eq!(
            take_entry_unicode_bytes(b""),
            Err(Err::Error(TokenParseError::UnexpectedEof))
        );
        assert_eq!(
            take_entry_unicode_bytes(b"\nentry"),
            Err(Err::Error(TokenParseError::ZeroLength))
        );
        assert_eq!(
            take_entry_unicode_bytes("🍄 8🍄".as_bytes()),
            Ok((" 8🍄".as_bytes(), "🍄"))
        );
        assert_eq!(
            take_entry_unicode_bytes(b"0@\xe4  "),
            Err(Err::Failure(TokenParseError::InvalidUtf8(&b"0@\xe4"[..])))
        );
    }

    #[test]
    fn test_take_entry_key() {
        assert_eq!(take_entry_unicode("abc"), Ok(("", "abc")));
        assert_eq!(take_entry_unicode("🍄🍄"), Ok(("", "🍄🍄")));
        assert_eq!(take_entry_unicode("🍄ü a🍄e"), Ok((" a🍄e", "🍄ü")));
        assert_eq!(
            take_entry_unicode(""),
            Err(Err::Error(TokenParseError::UnexpectedEof))
        );
    }

    #[test]
    fn test_take_entry_ascii() {
        assert_eq!(take_entry_ascii("abc"), Ok(("", "abc")));
        assert_eq!(take_entry_ascii("a1 c"), Ok((" c", "a1")));
        assert_eq!(
            take_entry_ascii("🍄"),
            Err(Err::Error(TokenParseError::ZeroLength))
        );
        assert_eq!(
            take_entry_ascii("4bc"),
            Err(Err::Error(TokenParseError::FieldKeyStartsWithDigit('4')))
        );
    }

    use proptest::prelude::*;
    proptest! {
        #[test]
        fn doesnt_crash(s in "\\PC*") {
            let _ = take_entry_unicode(&s);
            let _ = take_entry_ascii(&s);
        }
    }
}
