use crate::error::{Error, ErrorCode, Result};
use memchr::memchr2_iter;

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

fn is_identifier(s: &str) -> Result<()> {
    if s.is_empty() {
        Err(Error::syntax(ErrorCode::Empty))
    } else {
        find_invalid_identifier_char(s).map_or_else(
            || Ok(()),
            |ch| Err(Error::syntax(ErrorCode::DisallowedChar(ch))),
        )
    }
}

pub fn is_variable(s: &str) -> Result<()> {
    is_identifier(s)?;
    // SAFETY: if is_identifer(s) does not fail, then s is non-empty
    if s.as_bytes()[0].is_ascii_digit() {
        Err(Error::syntax(ErrorCode::VariableStartsWithDigit))
    } else {
        Ok(())
    }
}

#[inline]
pub fn is_field_key(s: &str) -> Result<()> {
    is_identifier(s)
}

#[inline]
pub fn is_entry_type(s: &str) -> Result<()> {
    is_identifier(s)
}

#[inline]
pub fn is_entry_key(s: &str) -> Result<()> {
    is_identifier(s)
}

pub fn is_balanced(input: &[u8]) -> Result<()> {
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
        assert_eq!(is_variable("a123"), Ok(()));
        assert_eq!(
            is_variable("1234"),
            Err(Error::syntax(ErrorCode::VariableStartsWithDigit))
        );
        assert_eq!(
            is_variable("a{"),
            Err(Error::syntax(ErrorCode::DisallowedChar('{')))
        );
        assert_eq!(
            is_variable(" "),
            Err(Error::syntax(ErrorCode::DisallowedChar(' ')))
        );
        assert_eq!(is_variable(""), Err(Error::syntax(ErrorCode::Empty)));
    }

    #[test]
    fn test_field_key() {
        assert_eq!(is_variable("a123"), Ok(()));
        assert_eq!(
            is_variable("1234"),
            Err(Error::syntax(ErrorCode::VariableStartsWithDigit))
        );
        assert_eq!(
            is_field_key("a)"),
            Err(Error::syntax(ErrorCode::DisallowedChar(')')))
        );
        assert_eq!(is_field_key("🍄"), Ok(()));
        assert_eq!(is_field_key(""), Err(Error::syntax(ErrorCode::Empty)));
    }

    #[test]
    fn test_balanced() {
        assert_eq!(is_balanced(b"1234"), Ok(()));
        assert_eq!(is_balanced(b""), Ok(()));
        assert_eq!(is_balanced(b"{}"), Ok(()));
        assert_eq!(is_balanced(b"{}{{}}"), Ok(()));
        assert_eq!(
            is_balanced(b"{"),
            Err(Error::syntax(ErrorCode::UnterminatedTextToken))
        );
        assert_eq!(
            is_balanced(b"{}}"),
            Err(Error::syntax(ErrorCode::UnexpectedClosingBracket))
        );
    }
}
