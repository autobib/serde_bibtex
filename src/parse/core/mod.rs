//! Core entry parsing methods.
//!
//! Ignored characters are parsed by [`ignored`]. In general, all parsing methods assume
//! that the input can have preceding ignored characters, and do not attempt to consume ignored
//! characters following the successful parse.

// mod balanced;
mod balanced_nom;
mod ignored;

use nom::{
    branch::alt,
    bytes::complete::{is_not, tag_no_case, take_until},
    character::complete::{char, digit1, one_of},
    combinator::{eof, map, not, opt, value as nom_value, verify},
    error::ParseError,
    sequence::{delimited, preceded, tuple},
    IResult, Parser,
};

use crate::value::{Identifier, Token};
use balanced_nom::{is_balanced, take_until_protected, take_until_unbalanced};
use ignored::{ignore_comment, ignore_junk};

pub fn preceded_comment<'r, O, E: ParseError<&'r str>, F>(
    mut parser: F,
) -> impl FnMut(&'r str) -> IResult<&'r str, O, E>
where
    F: Parser<&'r str, O, E>,
{
    move |input: &str| {
        let input = ignore_comment(input);
        parser.parse(input)
    }
}

/// Consume a comma.
pub fn comma(input: &str) -> IResult<&str, ()> {
    nom_value((), preceded_comment(char(',')))(input)
}

/// Consume a comma optionally.
pub fn opt_comma(input: &str) -> IResult<&str, ()> {
    nom_value((), opt(comma))(input)
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum EntryType<'r> {
    Preamble,
    Comment,
    Macro,
    Regular(Identifier<'r>),
}

fn special_entry<'r>(label: &'static str) -> impl FnMut(&'r str) -> IResult<&'r str, ()> {
    nom_value((), tuple((char('@'), preceded_comment(tag_no_case(label)))))
}

/// Parse the entry type including preceding characters. Returns None of we hit EOF.
///
/// # Example
/// In the below entry with the cursor at `<>`
/// ```bib
/// <>@article{key,
///   title = {text} # macro
/// }
/// ```
/// consumes `@article` and returns `article` wrapped in [`EntryType::Entry`].
///
/// Most entries are matched as [`EntryType::Entry`], but there are three
/// special chunk types: `@preamble`, `@comment`, and `@string`.
/// These are matched case-insensitively into [`EntryType::Preamble`], [`EntryType::Comment`], and
/// [`EntryType::Macro`] respectively.
pub fn entry_type(input: &str) -> IResult<&str, Option<EntryType>> {
    let input = ignore_junk(input);
    let (input, not_entry) = opt(alt((
        nom_value(Some(EntryType::Preamble), special_entry("preamble")),
        nom_value(Some(EntryType::Comment), special_entry("comment")),
        nom_value(Some(EntryType::Macro), special_entry("string")),
        nom_value(None, eof),
    )))(input)?;

    match not_entry {
        Some(entry) => Ok((input, entry)),
        None => map(preceded(char('@'), preceded_comment(identifier)), |ident| {
            Some(EntryType::Regular(ident))
        })(input),
    }
}

/// Parse the opening bracket and return the corresponding closing bracket.
pub fn initial(input: &str) -> IResult<&str, char> {
    preceded_comment(map(one_of("{("), |c| if c == '{' { '}' } else { ')' }))(input)
}

/// Consume the characters at the end of the entry.
///
/// # Example
/// In the below entry with the cursor at `>`
/// ```bib
/// @article{key,
///   title = {text} # macro,>
/// }
/// ```
/// consumes `\n}`.
pub fn terminal<'r>(closing_bracket: char) -> impl FnMut(&'r str) -> IResult<&'r str, ()>
where
{
    nom_value((), preceded_comment(char(closing_bracket)))
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
///   title = {text} # macro
/// }
/// ```
/// consumes `{key` and returns `key`.
pub fn citation_key(input: &str) -> IResult<&str, &str> {
    preceded_comment(key_chars)(input)
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
///   title = {text} # macro
/// }
/// ```
/// the identifiers are `article` and `title`.
pub fn identifier(input: &str) -> IResult<&str, Identifier> {
    let (input, ()) = not(digit1)(input)?;
    map(key_chars, Identifier::from_str_unchecked)(input)
}

/// Parse a field key, e.g. `title` in `@article{key, title = {Title}}`.
///
/// # Example
/// In the below entry with the cursor at `<>`
/// ```bib
/// @article{key<>,
///   title = {text} # macro
/// }
/// ```
/// consumes `,\n  title` and returns `title`.
pub fn field_key(input: &str) -> IResult<&str, Option<Identifier>> {
    opt(preceded_comment(identifier))(input)
}

// Match a comma and field key together. Unlike `preceded(comma, field_key)`, we make the entire
// match optional.
pub fn comma_and_field_key(input: &str) -> IResult<&str, Option<Identifier>> {
    opt(preceded(comma, preceded_comment(identifier)))(input)
}

/// Parse a field separator.
///
/// # Example
/// In the below entry with the cursor at `<>`
/// ```bib
/// @article{key,
///   title<> = {text} # macro
/// }
/// ```
/// consumes ` =`.
pub fn field_sep(input: &str) -> IResult<&str, ()> {
    nom_value((), preceded_comment(char('=')))(input)
}

/// Parse a token separator.
///
/// # Example
/// In the below entry with the cursor at `<>`
/// ```bib
/// @article{key,
///   title = {text}<> # macro
/// }
/// ```
/// consumes ` #`.
pub fn token_sep(input: &str) -> IResult<&str, ()> {
    nom_value((), preceded_comment(char('#')))(input)
}

/// Parse a field value delimited by curly braces.
///
/// # Example
/// In the below entry with the cursor at `<>`
/// ```bib
/// @article{key,
///   title = <>{text} # macro
/// }
/// ```
/// consumes `{text}` and returns `text`. The brackets must be balanced.
/// - Permitted: `{nested {brackets}}`
/// - Not permitted: `{{unmatched }`
pub fn curly(input: &str) -> IResult<&str, &str> {
    delimited(char('{'), take_until_unbalanced(b'{', b'}'), char('}'))(input)
}

pub fn quoted(input: &str) -> IResult<&str, &str> {
    delimited(
        char('"'),
        take_until_protected(b'{', b'}', b'\"'),
        char('"'),
    )(input)
}

// TODO: docs
/// Parse a field value token.
pub fn token(input: &str) -> IResult<&str, Token> {
    preceded_comment(alt((
        map(curly, Token::text_from),
        map(quoted, Token::text_from),
        map(digit1, Token::text_from),
        map(identifier, Token::Macro),
    )))(input)
}

pub fn subsequent_token(input: &str) -> IResult<&str, Option<Token>> {
    let (input, opt) = opt(tuple((token_sep, token)))(input)?;
    match opt {
        Some((_, token)) => Ok((input, Some(token))),
        None => Ok((input, None)),
    }
}

pub fn bracketed_text(input: &str) -> IResult<&str, &str> {
    let curly = delimited(char('{'), take_until_unbalanced(b'{', b'}'), char('}'));
    let round = delimited(
        char('('),
        verify(take_until(")"), is_balanced(b'{', b'}')),
        char(')'),
    );

    let (input, comment) = preceded_comment(alt((curly, round)))(input)?;
    Ok((input, comment))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_entry_type() {
        assert_eq!(
            entry_type("   @ \n article {"),
            Ok((
                " {",
                Some(EntryType::Regular(Identifier::from_str_unchecked(
                    "article"
                )))
            ))
        );

        assert_eq!(
            entry_type("@art🍄cle"),
            Ok((
                "",
                Some(EntryType::Regular(Identifier::from_str_unchecked(
                    "art🍄cle"
                )))
            ))
        );

        assert_eq!(
            entry_type("@ preamble  ("),
            Ok(("  (", Some(EntryType::Preamble)))
        );

        assert_eq!(
            entry_type("@ preAMble  ("),
            Ok(("  (", Some(EntryType::Preamble)))
        );

        assert_eq!(entry_type("@ COMMent"), Ok(("", Some(EntryType::Comment))));

        assert_eq!(entry_type("@%  \nstring"), Ok(("", Some(EntryType::Macro))));

        assert_eq!(entry_type("  "), Ok(("", None)));
        assert_eq!(entry_type("ignored junk %@}"), Ok(("", None)));

        assert!(entry_type("@{").is_err());
    }

    #[test]
    fn test_identifier() {
        assert_eq!(
            identifier("a0 "),
            Ok((" ", Identifier::from_str_unchecked("a0")))
        );
        assert_eq!(
            identifier("@🍄 "),
            Ok((" ", Identifier::from_str_unchecked("@🍄")))
        );
        assert!(identifier("3key").is_err());
        assert!(identifier("(key").is_err());
        assert!(identifier(" key").is_err());
    }

    #[test]
    fn test_quoted() {
        // normal quotes
        assert_eq!(
            token("\"quoted\"} "),
            Ok(("} ", Token::text_from("quoted")))
        );

        // balanced brackets inside
        assert_eq!(
            token("\"out{mid}\""),
            Ok(("", Token::text_from("out{mid}")))
        );
        assert!(token("\"{open\"").is_err());
        assert!(token("\"{closed}}\"").is_err());

        // internal quotes are allowed
        assert_eq!(
            token("\"a{b \"}c\""),
            Ok(("", Token::text_from("a{b \"}c")))
        );
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

        // ascii number tokens
        assert_eq!(token("0123 #"), Ok((" #", Token::text_from("0123"))));
        assert_eq!(token("0c"), Ok(("c", Token::text_from("0"))));

        // macro tokens
        assert_eq!(
            token("key0 #"),
            Ok((" #", Token::Macro(Identifier::from_str_unchecked("key0"))))
        );
        assert_eq!(
            token("{out{mid{inside}mid}}, "),
            Ok((", ", Token::text_from("out{mid{inside}mid}")))
        );
    }
}
