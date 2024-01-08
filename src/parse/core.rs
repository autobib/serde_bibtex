//! Core entry parsing methods.
//!
//! Ignored characters are parsed by [`ignored`]. In general, all parsing methods assume
//! that the input can have preceding ignored characters, and do not attempt to consume ignored
//! characters following the successful parse.
use nom::{
    branch::alt,
    bytes::complete::{is_not, take_until},
    character::complete::{anychar, char, digit1, multispace0, not_line_ending, one_of},
    combinator::{map, not, opt, value as nom_value, verify},
    sequence::{delimited, preceded, tuple},
    IResult,
};

use super::balanced::{is_balanced, take_until_unbalanced};
use crate::bib::{Identifier, Token};

/// Consume ignored characters.
pub fn ignored(input: &str) -> IResult<&str, ()> {
    let mut buffer = input;
    loop {
        let (stepped, _) = multispace0(buffer)?;
        match stepped.as_bytes().get(0) {
            // consume until line ending and loop
            Some(b'%') => {
                let (stepped, _) = tuple((anychar, not_line_ending))(stepped)?;
                buffer = stepped;
            }
            // ignore the next char and loop
            Some(b'\'') => {
                let (stepped, _) = tuple((anychar, anychar))(stepped)?;
                buffer = stepped
            }
            // break
            _ => break Ok((stepped, ())),
        }
    }
}

/// Parse the entry type including preceding characters.
///
/// # Example
/// In the below entry with the cursor at `<>`
/// ```bib
/// <>@article{key,
///   title = {text} # abbrev
/// }
/// ```
/// consumes `@article` and returns `article.
pub fn entry_type(input: &str) -> IResult<&str, Identifier> {
    preceded(tuple((ignored, char('@'), ignored)), identifier)(input)
}

/// Characters allowed in [`citation_key`] or [`identifier`].
pub fn key_chars(input: &str) -> IResult<&str, &str> {
    is_not("{}(),= \t\n\\#%'\"")(input)
}

/// Parse a citation key including the preceding bracket.
///
/// # Example
/// In the below entry with the cursor at `<>`
/// ```bib
/// @article<>{key,
///   title = {text} # abbrev
/// }
/// ```
/// consumes `{key` and returns `key`.
pub fn citation_key(input: &str) -> IResult<&str, (&str, char)> {
    let (input, (_, open, _, key)) = tuple((ignored, one_of("{("), ignored, key_chars))(input)?;
    Ok((input, (key, open)))
}

/// Consume the characters at the end of the entry.
///
/// # Example
/// In the below entry with the cursor at `<>`
/// ```bib
/// @article{key,
///   title = {text} # abbrev<>,
/// }
/// ```
/// consumes `,\n}`.
pub fn terminal(input: &str, matching: char) -> IResult<&str, ()> {
    nom_value(
        (),
        tuple((ignored, opt(tuple((char(','), ignored))), char(matching))),
    )(input)
}

/// Parse an identifier.
///
/// An identifier is any sequence of characters not in ` \t\\#%'\",=(){}` which
/// has length at least 1 and does not start with a digit.
///
/// # Example
/// In the below entry
/// ```bib
/// @article{key,
///   title = {text} # abbrev
/// }
/// ```
/// the identifiers are `article` and `title`.
pub fn identifier(input: &str) -> IResult<&str, Identifier> {
    let (input, ()) = not(digit1)(input)?;
    map(key_chars, Identifier::from)(input)
}

/// Parse a field key, e.g. `title` in `@article{key, title = {Title}}`.
///
/// # Example
/// In the below entry with the cursor at `<>`
/// ```bib
/// @article{key<>,
///   title = {text} # abbrev
/// }
/// ```
/// consumes `,\n  title` and returns `title`.
pub fn field_key(input: &str) -> IResult<&str, Option<Identifier>> {
    opt(preceded(tuple((ignored, char(','), ignored)), identifier))(input)
}

/// Parse a field separator.
///
/// # Example
/// In the below entry with the cursor at `<>`
/// ```bib
/// @article{key,
///   title<> = {text} # abbrev
/// }
/// ```
/// consumes ` =`.
pub fn field_sep(input: &str) -> IResult<&str, ()> {
    nom_value((), tuple((ignored, char('='))))(input)
}

/// Parse a token separator.
///
/// # Example
/// In the below entry with the cursor at `<>`
/// ```bib
/// @article{key,
///   title = {text}<> # abbrev
/// }
/// ```
/// consumes ` #`.
pub fn token_sep(input: &str) -> IResult<&str, ()> {
    nom_value((), tuple((ignored, char('#'))))(input)
}

/// Parse a field value delimited by curly braces.
///
/// # Example
/// In the below entry with the cursor at `<>`
/// ```bib
/// @article{key,
///   title = <>{text} # abbrev
/// }
/// ```
/// consumes `{text}` and returns `text`. The brackets must be balanced.
/// - Permitted: `{nested {brackets}}`
/// - Not permitted: `{{unmatched }`
pub fn curly(input: &str) -> IResult<&str, &str> {
    delimited(char('{'), take_until_unbalanced('{', '}'), char('}'))(input)
}

// TODO: docs
/// Parse a field value token.
pub fn token(input: &str) -> IResult<&str, Token> {
    let quoted = delimited(
        char('"'),
        verify(take_until("\""), is_balanced('{', '}')),
        char('"'),
    );

    preceded(
        ignored,
        alt((
            map(curly, Token::text_from),
            map(quoted, Token::text_from),
            map(digit1, Token::text_from),
            map(identifier, Token::Abbrev),
        )),
    )(input)
}

pub fn subsequent_token(input: &str) -> IResult<&str, Option<Token>> {
    let (input, opt) = opt(tuple((token_sep, token)))(input)?;
    match opt {
        Some((_, token)) => Ok((input, Some(token))),
        None => Ok((input, None)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ignored() {
        assert_eq!(ignored("  "), Ok(("", ())));
        assert_eq!(ignored("% ignored\n rest"), Ok(("rest", ())));
        assert_eq!(ignored("'i'g'n'o'r'e'd rest"), Ok(("rest", ())));
        assert_eq!(ignored("'🍄 rest"), Ok(("rest", ()))); // note: errors in biber because of
                                                           // unicode handling
        assert_eq!(ignored("'i%ig\nrest"), Ok(("rest", ())));
        assert_eq!(ignored("'%rest"), Ok(("rest", ())));
        assert_eq!(ignored("%"), Ok(("", ())));
        assert_eq!(ignored("%%'\nrest"), Ok(("rest", ())));
        assert_eq!(ignored("% i\r\n 'rrest"), Ok(("rest", ())));
    }

    #[test]
    fn test_identifier() {
        assert_eq!(identifier("a0 "), Ok((" ", Identifier::from("a0"))));
        assert!(identifier("3key").is_err());
        assert!(identifier(" key").is_err());
    }

    #[test]
    fn test_token() {
        // bracketed tokens
        assert_eq!(
            token("{bracketed}, "),
            Ok((", ", Token::text_from("bracketed")))
        );
        assert!(token("{bracketed{error}").is_err());
        assert!(token("{{bad}").is_err());

        // quoted tokens
        assert_eq!(
            token("\"quoted\"} "),
            Ok(("} ", Token::text_from("quoted")))
        );
        assert_eq!(
            token("\"out{mid}\""),
            Ok(("", Token::text_from("out{mid}")))
        );
        assert!(token("\"{open\"").is_err());
        assert!(token("\"{closed}}\"").is_err());

        // ascii number tokens
        assert_eq!(token("0123 #"), Ok((" #", Token::text_from("0123"))));
        assert_eq!(token("0c"), Ok(("c", Token::text_from("0"))));

        // abbreviation tokens
        assert_eq!(token("key0 #"), Ok((" #", Token::Abbrev("key0".into()))));
        assert_eq!(
            token("{out{mid{inside}mid}}, "),
            Ok((", ", Token::text_from("out{mid{inside}mid}")))
        );
    }
}
