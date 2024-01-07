//! Core entry parsing methods.
//!
//! Ignored characters are parsed by [`bibtex_ignored`]. In general, all parsing methods assume
//! that the input can have preceding ignored characters, and do not attempt to delete ignored
//! characters following the successful parse.
use nom::{
    branch::alt,
    bytes::complete::{is_not, take_until},
    character::complete::{char, digit1, multispace0, one_of},
    combinator::{map, not, opt, value as nom_value, verify},
    sequence::{delimited, preceded, tuple},
    IResult,
};

use super::balanced::{is_balanced, take_until_unbalanced};
use crate::bib::{Identifier, Token};

/// Consume characters ignored by bib(la)tex
pub fn bibtex_ignored(input: &str) -> IResult<&str, ()> {
    // TODO: incorporate bibtex_comment
    //     nom_value(
    //         (), // Output is thrown away.
    //         pair(char('%'), is_not("\n\r")),
    //     )(i)

    // TODO: incorporate bibtex skips, e.g. \% is discarded, or '%
    nom_value((), multispace0)(input)
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
    preceded(
        tuple((bibtex_ignored, char('@'), bibtex_ignored)),
        identifier,
    )(input)
}

fn key_chars(input: &str) -> IResult<&str, &str> {
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
    let (input, (_, open, _, key)) =
        tuple((bibtex_ignored, one_of("{("), bibtex_ignored, key_chars))(input)?;
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
        tuple((
            bibtex_ignored,
            opt(tuple((char(','), bibtex_ignored))),
            char(matching),
        )),
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
    opt(preceded(
        tuple((bibtex_ignored, char(','), bibtex_ignored)),
        identifier,
    ))(input)
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
    nom_value((), tuple((bibtex_ignored, char('='))))(input)
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
    nom_value((), tuple((bibtex_ignored, char('#'))))(input)
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
        bibtex_ignored,
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
