use crate::error::{Error, ErrorCode, Result};
use memchr::memchr2_iter;

// Lookup table for bytes which could appear in an entry key. This includes the
// ascii printable characters with "{}(),= \t\n\\#%\"" removed, as well as bytes
// that could appear in non-ascii UTF-8.
//
// Note that this table is insufficient for UTF-8 validation outside the ASCII range:
// it is only used for short-circuited termination of parsing!
pub(super) static ENTRY_ALLOWED: [bool; 256] = {
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
        .find(|&b| !ENTRY_ALLOWED[*b as usize])
        .map(|b| unsafe { char::from_u32_unchecked(*b as u32) })
}

pub fn variable(s: &str) -> Result<()> {
    if s.len() == 0 {
        Err(Error::syntax(ErrorCode::Empty))
    } else if matches!(s.as_bytes()[0], b'0'..=b'9') {
        Err(Error::syntax(ErrorCode::IdentifierStartsWithDigit))
    } else {
        find_invalid_identifier_char(s).map_or_else(
            || Ok(()),
            |ch| Err(Error::syntax(ErrorCode::DisallowedChar(ch))),
        )
    }
}

#[inline]
pub fn entry_type(s: &str) -> Result<()> {
    variable(s)
}

#[inline]
pub fn entry_key(s: &str) -> Result<()> {
    variable(s)
}

fn find_invalid_ascii_char(input: &str) -> Option<char> {
    input
        .chars()
        // check ascii first so `ch as usize` will not be out of bounds
        .find(|&ch| !ch.is_ascii() || !ENTRY_ALLOWED[ch as usize])
}

pub fn field_key(s: &str) -> Result<()> {
    if s.len() == 0 {
        Err(Error::syntax(ErrorCode::Empty))
    } else if matches!(s.as_bytes()[0], b'0'..=b'9') {
        Err(Error::syntax(ErrorCode::IdentifierStartsWithDigit))
    } else {
        find_invalid_ascii_char(s).map_or_else(
            || Ok(()),
            |ch| Err(Error::syntax(ErrorCode::DisallowedChar(ch))),
        )
    }
}

pub fn balanced(input: &[u8]) -> Result<()> {
    let mut bracket_depth = 0;

    for pos in memchr2_iter(b'{', b'}', input) {
        if input[pos] == b'{' {
            bracket_depth += 1
        } else {
            // too many closing brackets
            if bracket_depth == 0 {
                return Err(Error::syntax(ErrorCode::UnexpectedClosingBracket));
            }
            bracket_depth -= 1;
        }
    }

    if bracket_depth == 0 {
        Ok(())
    } else {
        Err(Error::syntax(ErrorCode::UnterminatedTextToken))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_variable() {
        assert_eq!(variable("a123"), Ok(()));
        assert_eq!(
            variable("1234"),
            Err(Error::syntax(ErrorCode::IdentifierStartsWithDigit))
        );
        assert_eq!(
            variable("a{"),
            Err(Error::syntax(ErrorCode::DisallowedChar('{')))
        );
        assert_eq!(
            variable(" "),
            Err(Error::syntax(ErrorCode::DisallowedChar(' ')))
        );
        assert_eq!(variable(""), Err(Error::syntax(ErrorCode::Empty)));
    }

    #[test]
    fn test_field_key() {
        assert_eq!(variable("a123"), Ok(()));
        assert_eq!(
            variable("1234"),
            Err(Error::syntax(ErrorCode::IdentifierStartsWithDigit))
        );
        assert_eq!(
            field_key("a)"),
            Err(Error::syntax(ErrorCode::DisallowedChar(')')))
        );
        assert_eq!(
            field_key("🍄"),
            Err(Error::syntax(ErrorCode::DisallowedChar('🍄')))
        );
        assert_eq!(field_key(""), Err(Error::syntax(ErrorCode::Empty)));
    }

    #[test]
    fn test_balanced() {
        assert_eq!(balanced(b"1234"), Ok(()));
        assert_eq!(balanced(b""), Ok(()));
        assert_eq!(balanced(b"{}"), Ok(()));
        assert_eq!(balanced(b"{}{{}}"), Ok(()));
        assert_eq!(
            balanced(b"{"),
            Err(Error::syntax(ErrorCode::UnterminatedTextToken))
        );
        assert_eq!(
            balanced(b"{}}"),
            Err(Error::syntax(ErrorCode::UnexpectedClosingBracket))
        );
    }
}
