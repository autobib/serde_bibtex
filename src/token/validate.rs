//! # Validation methods
//! This module exposes some methods to aid validation of BibTeX-type strings.

// use crate::error::{Error, ErrorCode, Result};
use memchr::memchr2_iter;

use super::TokenError;

// pub struct TokenError

/// Lookup table for bytes which could appear in an entry key. This includes the
/// ascii printable characters with "{}(),= \t\n\\#%\"" removed, as well as bytes
/// that could appear in non-ascii UTF-8.
///
/// Note that this table is insufficient for UTF-8 validation outside the ASCII range:
/// it is only used for short-circuited termination of parsing!
pub(crate) static IDENTIFIER_ALLOWED: [bool; 256] = {
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

/// Returns `Some(ch)` if the input does not contain a disallowed char `ch`, and `None` otherwise.
///
/// A disallowed char is any char in `"{}(),=\\#%\""`.
fn find_invalid_identifier_char(input: &str) -> Option<char> {
    input
        .as_bytes()
        .iter()
        .find(|&b| !IDENTIFIER_ALLOWED[*b as usize])
        .map(|b| unsafe { char::from_u32_unchecked(*b as u32) })
}

fn check_identifier(s: &str) -> Result<(), TokenError> {
    if s.is_empty() {
        Err(TokenError::Empty)
    } else {
        find_invalid_identifier_char(s)
            .map_or_else(|| Ok(()), |ch| Err(TokenError::InvalidChar(ch)))
    }
}

pub fn check_variable(s: &str) -> Result<(), TokenError> {
    check_identifier(s)?;
    // SAFETY: if is_identifer(s) does not fail, then s is non-empty
    if s.as_bytes()[0].is_ascii_digit() {
        Err(TokenError::StartsWithDigit)
    } else {
        Ok(())
    }
}

/// Check if a given string is valid as a variable.
#[inline]
pub fn is_variable(s: &str) -> bool {
    check_variable(s).is_ok()
}

#[inline]
pub fn check_field_key(s: &str) -> Result<(), TokenError> {
    check_identifier(s)
}

/// Check if a given string is valid as a field key.
#[inline]
pub fn is_field_key(s: &str) -> bool {
    check_field_key(s).is_ok()
}

#[inline]
pub fn check_entry_type(s: &str) -> Result<(), TokenError> {
    check_identifier(s)
}

/// Check if a given string is valid as an entry type.
#[inline]
pub fn is_entry_type(s: &str) -> bool {
    check_entry_type(s).is_ok()
}

/// Check if a given string is valid as a regular entry type.
#[inline]
pub fn is_regular_entry_type(s: &str) -> bool {
    if s.eq_ignore_ascii_case("string")
        || s.eq_ignore_ascii_case("comment")
        || s.eq_ignore_ascii_case("preamble")
    {
        false
    } else {
        is_entry_type(s)
    }
}

#[inline]
pub fn check_entry_key(s: &str) -> Result<(), TokenError> {
    check_identifier(s)
}

/// Check if a given string is valid as an entry key.
#[inline]
pub fn is_entry_key(s: &str) -> bool {
    check_entry_key(s).is_ok()
}

pub fn check_balanced(input: &[u8]) -> Result<(), TokenError> {
    let mut bracket_depth = 0;

    for pos in memchr2_iter(b'{', b'}', input) {
        if input[pos] == b'{' {
            bracket_depth += 1
        } else {
            // too many closing brackets
            if bracket_depth == 0 {
                return Err(TokenError::ExtraClosingBracket);
            }
            bracket_depth -= 1;
        }
    }

    if bracket_depth == 0 {
        Ok(())
    } else {
        Err(TokenError::ExtraOpeningBracket)
    }
}

/// Check if a given string has balanced `{}` brackets.
#[inline]
pub fn is_balanced(input: &[u8]) -> bool {
    check_balanced(input).is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_variable() {
        assert_eq!(check_variable("a123"), Ok(()));
        assert_eq!(check_variable("1234"), Err(TokenError::StartsWithDigit));
        assert_eq!(check_variable("a{"), Err(TokenError::InvalidChar('{')));
        assert_eq!(check_variable(" "), Err(TokenError::InvalidChar(' ')));
        assert_eq!(check_variable(""), Err(TokenError::Empty));
    }

    #[test]
    fn test_field_key() {
        assert_eq!(check_variable("a123"), Ok(()));
        assert_eq!(check_variable("1234"), Err(TokenError::StartsWithDigit));
        assert_eq!(check_field_key("a)"), Err(TokenError::InvalidChar(')')));
        assert_eq!(check_field_key("🍄"), Ok(()));
        assert_eq!(check_field_key(""), Err(TokenError::Empty));
    }

    #[test]
    fn test_balanced() {
        assert_eq!(check_balanced(b"1234"), Ok(()));
        assert_eq!(check_balanced(b""), Ok(()));
        assert_eq!(check_balanced(b"{}"), Ok(()));
        assert_eq!(check_balanced(b"{}{{}}"), Ok(()));
        assert_eq!(check_balanced(b"{"), Err(TokenError::ExtraOpeningBracket));
        assert_eq!(check_balanced(b"{}}"), Err(TokenError::ExtraClosingBracket));
    }
}
