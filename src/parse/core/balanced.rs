use memchr::{memchr2_iter, memchr3_iter};
use std::str::from_utf8;

use nom::{
    error::{Error, ErrorKind, ParseError},
    Err, IResult,
};

/// Consume bytes from the input until there are more closing brackets than opening brackets.
/// This is an error if we do not find the unbalanced closing bracket.
pub fn take_until_unbalanced_bytes(
    opening_bracket: u8,
    closing_bracket: u8,
) -> impl Fn(&[u8]) -> IResult<&[u8], &[u8]> {
    debug_assert!(opening_bracket != closing_bracket);

    move |i: &[u8]| {
        let mut bracket_depth = 0;

        for idx in memchr2_iter(opening_bracket, closing_bracket, i) {
            if i[idx] == opening_bracket {
                bracket_depth += 1
            } else {
                // found the closing bracket
                if bracket_depth == 0 {
                    return Ok((&i[idx..], &i[0..idx]));
                }
                bracket_depth -= 1;
            }
        }

        // we did not find find the closing bracket
        Err(Err::Error(Error::from_error_kind(i, ErrorKind::TakeUntil)))
    }
}

pub fn is_balanced_bytes(opening_bracket: u8, closing_bracket: u8) -> impl Fn(&[u8]) -> bool {
    debug_assert!(opening_bracket != closing_bracket);

    move |i: &[u8]| {
        let mut bracket_depth = 0;

        for idx in memchr2_iter(opening_bracket, closing_bracket, i) {
            if i[idx] == opening_bracket {
                bracket_depth += 1
            } else {
                // too many closing brackets
                if bracket_depth == 0 {
                    return false;
                }
                bracket_depth -= 1;
            }
        }

        bracket_depth == 0
    }
}

pub fn is_balanced(opening_bracket: u8, closing_bracket: u8) -> impl Fn(&str) -> bool {
    move |s: &str| is_balanced_bytes(opening_bracket, closing_bracket)(s.as_bytes())
}

/// Similar to [`nom::bytes::complete::take_until`], except it only terminates if the reached
/// byte is not protected by brackets.
pub fn take_until_protected_bytes(
    opening_bracket: u8,
    closing_bracket: u8,
    until: u8,
) -> impl Fn(&[u8]) -> IResult<&[u8], &[u8]> {
    debug_assert!(opening_bracket != closing_bracket);

    move |i: &[u8]| {
        let mut bracket_depth = 0;

        for idx in memchr3_iter(until, opening_bracket, closing_bracket, i) {
            if i[idx] == until {
                if bracket_depth == 0 {
                    return Ok((&i[idx..], &i[0..idx]));
                }
            } else if i[idx] == opening_bracket {
                bracket_depth += 1
            } else {
                // found closing bracket
                // too many closing brackets
                if bracket_depth < 0 {
                    return Err(Err::Error(Error::from_error_kind(i, ErrorKind::TakeUntil)));
                }
                bracket_depth -= 1;
            }
        }

        // we did not find find the closing bracket
        Err(Err::Error(Error::from_error_kind(i, ErrorKind::TakeUntil)))
    }
}

/// SAFETY: `opening_bracket` and `closing_bracket` must be a valid single unicode codepoint or the
/// str conversions can fail!
pub fn take_until_unbalanced(
    opening_bracket: u8,
    closing_bracket: u8,
) -> impl Fn(&str) -> IResult<&str, &str> {
    debug_assert!(opening_bracket.is_ascii());
    debug_assert!(closing_bracket.is_ascii());

    move |s: &str| match take_until_unbalanced_bytes(opening_bracket, closing_bracket)(s.as_bytes())
    {
        Ok((s, captured)) => Ok((from_utf8(s).unwrap(), from_utf8(captured).unwrap())),
        Result::Err(err) => Err(err.map_input(|b| from_utf8(b).unwrap())),
    }
}

/// SAFETY: `opening_bracket` and `closing_bracket` must be a valid single unicode codepoint or the
/// str conversions can fail!
pub fn take_until_protected(
    opening_bracket: u8,
    closing_bracket: u8,
    until: u8,
) -> impl Fn(&str) -> IResult<&str, &str> {
    debug_assert!(opening_bracket.is_ascii());
    debug_assert!(closing_bracket.is_ascii());
    debug_assert!(until.is_ascii());

    move |s: &str| match take_until_protected_bytes(opening_bracket, closing_bracket, until)(
        s.as_bytes(),
    ) {
        Ok((s, captured)) => Ok((from_utf8(s).unwrap(), from_utf8(captured).unwrap())),
        Result::Err(err) => Err(err.map_input(|b| from_utf8(b).unwrap())),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_take_until_unbalanced_bytes() {
        assert_eq!(
            take_until_unbalanced_bytes(b'(', b')')(b"url)abc"),
            Ok((&b")abc"[..], &b"url"[..]))
        );
        assert_eq!(
            take_until_unbalanced_bytes(b'(', b')')("u()rl)a🍄c".as_bytes()),
            Ok((")a🍄c".as_bytes(), &b"u()rl"[..]))
        );
        assert_eq!(
            take_until_unbalanced_bytes(b'(', b')')(b"u(())rl)abc"),
            Ok((&b")abc"[..], &b"u(())rl"[..]))
        );
        assert_eq!(
            take_until_unbalanced_bytes(b'(', b')')(b"u(())r()l)abc"),
            Ok((&b")abc"[..], &b"u(())r()l"[..]))
        );
        assert_eq!(
            take_until_unbalanced_bytes(b'(', b')')(b"u(())r()l)abc"),
            Ok((&b")abc"[..], &b"u(())r()l"[..]))
        );

        assert!(take_until_unbalanced_bytes(b'{', b'}')(b"none").is_err());
        assert!(take_until_unbalanced_bytes(b'{', b'}')(b"{no}e").is_err());
    }

    #[test]
    fn test_take_until_protected_bytes() {
        assert_eq!(
            take_until_protected_bytes(b'(', b')', b'"')(b"captured\"rest"),
            Ok((&b"\"rest"[..], &b"captured"[..]))
        );
        assert_eq!(
            take_until_protected_bytes(b'(', b')', b'$')(b"a($)$rest"),
            Ok((&b"$rest"[..], &b"a($)"[..]))
        );
        assert_eq!(
            take_until_protected_bytes(b'(', b')', b'$')(b"a(($) $)$rest"),
            Ok((&b"$rest"[..], &b"a(($) $)"[..]))
        );
        // did not find unprotected
        assert!(take_until_protected_bytes(b'(', b')', b'$')(b"($").is_err());
        // unexpected closing
        assert!(take_until_protected_bytes(b'(', b')', b'$')(b")$").is_err());
    }

    #[test]
    fn test_take_until_unbalanced() {
        assert_eq!(
            take_until_unbalanced(b'(', b')')("ur🍄)abc"),
            Ok((")abc", "ur🍄"))
        );
        assert_eq!(
            take_until_unbalanced(b'(', b')')("u())🍄bc"),
            Ok((")🍄bc", "u()"))
        );
        assert_eq!(
            take_until_unbalanced(b'(', b')')("u(())rl)abc"),
            Ok((")abc", "u(())rl"))
        );
        assert_eq!(
            take_until_unbalanced(b'(', b')')("u(())r()l)abc"),
            Ok((")abc", "u(())r()l"))
        );
        assert_eq!(
            take_until_unbalanced(b'(', b')')("u(())r(labc"),
            Err(nom::Err::Error(nom::error::Error::new(
                "u(())r(labc",
                ErrorKind::TakeUntil
            )))
        );
    }

    #[test]
    fn test_take_until_protected() {
        assert_eq!(
            take_until_protected(b'(', b')', b'"')("🍄\"🍄"),
            Ok(("\"🍄", "🍄"))
        );
        assert_eq!(
            take_until_protected(b'(', b')', b'$')("a($)🍄$rest"),
            Ok(("$rest", "a($)🍄"))
        );
    }

    #[test]
    fn test_is_balanced() {
        assert!(is_balanced(b'(', b')')("(())()"));
        assert!(is_balanced(b'{', b'}')("  {{{}🍄}}{   }{ {}} "));
        assert!(is_balanced(b'(', b')')("some t🍄xt"));
        assert!(is_balanced(b'(', b')')("(co🍄tents(nested))  "));
        assert!(!is_balanced(b'{', b'}')("\"{unbalanced\""));
        assert!(!is_balanced(b'{', b'}')("{o🍄en"));
        assert!(!is_balanced(b'{', b'}')("{clo🍄ed}}"));
        assert!(!is_balanced(b'a', b'b')("abb"));
    }
}
