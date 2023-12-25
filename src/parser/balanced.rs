use nom::error::{Error, ErrorKind, ParseError};
use nom::Err;
use nom::IResult;

pub fn take_until_char(halt: char) -> impl Fn(&str) -> IResult<&str, &str> {
    move |i: &str| match i.find(halt) {
        None => Err(Err::Error(Error::from_error_kind(i, ErrorKind::TakeUntil))),
        Some(index) => Ok((&i[index..], &i[0..index])),
    }
}

/// A parser designed to work inside the `nom::sequence::delimited` parser, e.g.:
/// ```
/// use nom::character::complete::char;
/// use nom::sequence::delimited;
/// use serde_bibtex::parser::balanced::take_until_unbalanced;
///
/// let mut parser = delimited(char('{'), take_until_unbalanced('{', '}'), char('}'));
/// assert_eq!(parser("{{inside}inside}abc"), Ok(("abc", "{inside}inside")));
/// ```
/// It skips nested brackets until it finds an extra unbalanced closing bracket. This function is
/// very similar to `nom::bytes::complete::take_until("}")`, except it also takes nested brackets.
pub fn take_until_unbalanced(
    opening_bracket: char,
    closing_bracket: char,
) -> impl Fn(&str) -> IResult<&str, &str> {
    move |i: &str| {
        let mut index = 0;
        let mut bracket_depth = 0;
        while let Some(n) = &i[index..].find(&[opening_bracket, closing_bracket][..]) {
            index += n;
            let mut it = i[index..].chars();
            match it.next().unwrap_or_default() {
                c if c == opening_bracket => {
                    bracket_depth += 1;
                    index += opening_bracket.len_utf8();
                }
                c if c == closing_bracket => {
                    bracket_depth -= 1;
                    index += closing_bracket.len_utf8();
                }
                _ => unreachable!(),
            };
            // We found the unmatched closing bracket.
            if bracket_depth == -1 {
                // We do not consume it.
                index -= closing_bracket.len_utf8();
                return Ok((&i[index..], &i[0..index]));
            };
        }

        if bracket_depth == 0 {
            Ok(("", i))
        } else {
            Err(Err::Error(Error::from_error_kind(i, ErrorKind::TakeUntil)))
        }
    }
}

/// Verify that a string has balanced opening and closing brackets.
/// ```
/// use serde_bibtex::parser::balanced::is_balanced;
///
/// assert!(is_balanced('{', '}')("{{{} }}{   }{ {}}"));
/// assert!(!is_balanced('a', 'b')("abb"));
pub fn is_balanced(opening_bracket: char, closing_bracket: char) -> impl Fn(&str) -> bool {
    move |i: &str| {
        // Iterate over opening and closing brackets, keeping track of the current bracket
        // depth. The depth can never drop below 0, and must be exactly 0 at the end.
        let mut index = 0;
        let mut bracket_depth = 0;
        while let Some(n) = &i[index..].find(&[opening_bracket, closing_bracket][..]) {
            index += n;
            let mut it = i[index..].chars();
            match it.next().unwrap_or_default() {
                c if c == opening_bracket => {
                    bracket_depth += 1;
                    index += opening_bracket.len_utf8();
                }
                c if c == closing_bracket => {
                    bracket_depth -= 1;
                    index += closing_bracket.len_utf8();
                }
                _ => unreachable!(),
            }
            if bracket_depth == -1 {
                return false;
            }
        }
        bracket_depth == 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nom::error::ErrorKind;

    #[test]
    fn test_take_until_unbalanced() {
        assert_eq!(
            take_until_unbalanced('(', ')')("url)abc"),
            Ok((")abc", "url"))
        );
        assert_eq!(
            take_until_unbalanced('(', ')')("u()rl)abc"),
            Ok((")abc", "u()rl"))
        );
        assert_eq!(
            take_until_unbalanced('(', ')')("u(())rl)abc"),
            Ok((")abc", "u(())rl"))
        );
        assert_eq!(
            take_until_unbalanced('(', ')')("u(())r()l)abc"),
            Ok((")abc", "u(())r()l"))
        );
        assert_eq!(
            take_until_unbalanced('(', ')')("u(())r()labc"),
            Ok(("", "u(())r()labc"))
        );
        assert_eq!(
            take_until_unbalanced('(', ')')("u(())r(labc"),
            Err(nom::Err::Error(nom::error::Error::new(
                "u(())r(labc",
                ErrorKind::TakeUntil
            )))
        );
        assert_eq!(
            take_until_unbalanced('€', 'ü')("€uü€€üürlüabc"),
            Ok(("üabc", "€uü€€üürl"))
        );
    }

    #[test]
    fn test_is_balanced() {
        assert!(is_balanced('(', ')')("(())()"));
        assert!(is_balanced('(', ')')("some text"));
        assert!(is_balanced('(', ')')("(contents(nested))  "));
        assert!(!is_balanced('{', '}')("\"{unbalanced\""));
        assert!(!is_balanced('{', '}')("{open"));
    }
}
