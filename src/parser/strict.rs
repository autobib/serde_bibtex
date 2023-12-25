use nom::{
    branch::alt,
    bytes::complete::{is_a, is_not, tag_no_case},
    character::complete::{char, digit1, multispace0},
    combinator::{map, not, opt, verify},
    multi::{separated_list0, separated_list1},
    sequence::{delimited, separated_pair, tuple},
    IResult,
};

use super::balanced::{is_balanced, take_until_char, take_until_unbalanced};
use crate::bibliography::{
    Abbreviation, Chunk, Comment, Entry, Field, Identifier, Preamble, Token, Value,
};

pub const IDENTIFIER_CHARS: &str =
    "!$&*+-./0123456789:;<>?@ABCDEFGHIJKLMNOPQRSTUVWXYZ[]^_`abcdefghijklmnopqrstuvwxyz|~";

/// Parse an abbreviation, which is any sequence of characters in `IDENTIFIER_CHARS` that has
/// length at least 1 and does not start with a digit.
/// ```
/// use serde_bibtex::parser::strict::identifier;
///
/// assert_eq!(
///     identifier("key0"),
///     Ok(("", "key0".into()))
/// );
///
/// assert!(identifier("0key").is_err());
///
/// assert!(identifier("(i)dent").is_err());
/// ```
pub fn identifier(input: &str) -> IResult<&str, Identifier> {
    let (input, ()) = not(digit1)(input)?;
    map(is_a(IDENTIFIER_CHARS), Identifier::from)(input)
}

/// Parse a field token, which is either `{curly}`, `"quoted"`, an abbreviation, or a sequence of
/// digits.
/// ```
/// use serde_bibtex::parser::strict::token;
/// use serde_bibtex::bibliography::Token;
///
/// assert_eq!(
///     token("1234"),
///     Ok(("", Token::Text("1234")))
/// );
/// ```
/// The `{` and `}` brackets need to be balanced:
/// ```
/// # use serde_bibtex::parser::strict::token;
/// # use serde_bibtex::bibliography::Token;
/// assert_eq!(
///     token("\"{outside{inside}}\""),
///     Ok(("", Token::Text("{outside{inside}}")))
/// );
///
// /// assert!(token("\"{unbalanced\"").is_err())
/// ```
/// For a `{curly}` token, the parser eats characters until the brackets are balanced.
/// ```
/// # use serde_bibtex::parser::strict::token;
/// # use serde_bibtex::bibliography::Token;
/// assert_eq!(token("{stopped}}"), Ok(("}", Token::Text("stopped"))));
/// ```
pub fn token(input: &str) -> IResult<&str, Token> {
    let curly = map(
        delimited(char('{'), take_until_unbalanced('{', '}'), char('}')),
        Token::Text,
    );

    let quoted = map(
        delimited(
            char('"'),
            verify(take_until_char('"'), is_balanced('{', '}')),
            char('"'),
        ),
        Token::Text,
    );

    alt((
        curly,
        quoted,
        map(digit1, Token::Text),
        map(identifier, Token::from),
    ))(input)
}

/// Parse a field value by splitting at '#', and removing excess whitespace.
/// ```
/// use serde_bibtex::parser::strict::value;
/// use serde_bibtex::bibliography::Token;
/// assert_eq!(
///     value("123 # abbrev # {bracketed} # \"quoted\","),
///     Ok((",", vec![Token::Text("123"), Token::Abbrev("abbrev".into()), Token::Text("bracketed"), Token::Text("quoted")].into()))
/// );
/// ```
/// Note that the value cannot have leading whitespace and must contain at least one valid identifier.
/// ```
/// # use serde_bibtex::parser::strict::value;
/// assert!(value(" 123").is_err());
/// assert!(value("}").is_err());
/// ```
pub fn value(input: &str) -> IResult<&str, Value> {
    let (input, tokens) =
        separated_list1(tuple((multispace0, char('#'), multispace0)), token)(input)?;
    Ok((input, tokens.into()))
}

/// Parse a field `abbrev = {value}`.
/// ```
/// use serde_bibtex::parser::strict::field;
/// use serde_bibtex::bibliography::Token;
///
/// assert_eq!(
///     field("title = {A} # \"Title\""),
///     Ok(("", ("title".into(), vec![Token::Text("A"), Token::Text("Title")]).into()))
/// );
/// ```
pub fn field(input: &str) -> IResult<&str, Field> {
    map(
        separated_pair(
            identifier,
            tuple((multispace0, char('='), multispace0)),
            value,
        ),
        Field::from,
    )(input)
}

/// Consume a field separator.
fn field_sep(input: &str) -> IResult<&str, ()> {
    let (input, _) = tuple((multispace0, char(','), multispace0))(input)?;
    Ok((input, ()))
}

/// Parse a list of fields.
fn fields(input: &str) -> IResult<&str, Vec<Field>> {
    separated_list0(field_sep, field)(input)
}

/// Parse an entry body.
fn entry_body(input: &str) -> IResult<&str, (&str, Vec<Field>)> {
    let (input, (_, entry_key, _, fields, _)) = tuple((
        multispace0,
        is_not("{}(), \t\n"),
        field_sep,
        fields,
        opt(field_sep),
    ))(input)?;
    Ok((input, (entry_key, fields)))
}

/// Parse an `@entry` chunk.
/// ```
/// use serde_bibtex::parser::strict::entry;
/// assert!(
///     entry(
///         "@article{key:0,
///            author = \"Anonymous\",
///            title = {A title},
///            date = 2014,
///          }"
///     ).is_ok()
/// );
/// ```
pub fn entry(input: &str) -> IResult<&str, Entry> {
    let (input, (_, _, entry_type, _)) =
        tuple((char('@'), multispace0, identifier, multispace0))(input)?;
    let (input, (key, fields)) = alt((
        delimited(char('{'), entry_body, char('}')),
        delimited(char('('), entry_body, char(')')),
    ))(input)?;
    Ok((
        input,
        Entry {
            entry_type,
            key,
            fields,
        },
    ))
}

fn padded_field(input: &str) -> IResult<&str, Field> {
    let (input, (_, captured, _)) = tuple((multispace0, field, multispace0))(input)?;
    Ok((input, captured))
}

/// Parse an `@string` chunk.
pub fn abbreviation(input: &str) -> IResult<&str, Abbreviation> {
    let (input, _) = tuple((char('@'), multispace0, tag_no_case("string"), multispace0))(input)?;
    let (input, captured) = alt((
        delimited(char('{'), padded_field, char('}')),
        delimited(char('('), padded_field, char(')')),
    ))(input)?;
    Ok((input, Abbreviation(captured)))
}

/// Parse an `@comment` chunk.
/// ```
/// use serde_bibtex::parser::strict::comment;
/// use serde_bibtex::bibliography::{Comment, Token};
///
/// assert_eq!(
///     comment("@comment{name@gmail.com {Author One}}"),
///     Ok(("", Comment("name@gmail.com {Author One}")))
/// );
pub fn comment(input: &str) -> IResult<&str, Comment> {
    let curly = map(
        delimited(char('{'), take_until_unbalanced('{', '}'), char('}')),
        Comment,
    );

    let round = map(
        delimited(
            char('('),
            verify(take_until_char(')'), is_balanced('{', '}')),
            char(')'),
        ),
        Comment,
    );
    let (input, (_, _, _, comment)) =
        tuple((char('@'), multispace0, tag_no_case("comment"), alt((curly, round))))(input)?;
    Ok((input, comment))
}

/// Parse a value surrounded by ignored whitespace
fn padded_value(input: &str) -> IResult<&str, Value> {
    let (input, (_, captured, _)) = tuple((multispace0, value, multispace0))(input)?;
    Ok((input, captured))
}

/// Parse an `@preamble` chunk.
pub fn preamble(input: &str) -> IResult<&str, Preamble> {
    let (input, _) = tuple((char('@'), multispace0, tag_no_case("preamble"), multispace0))(input)?;
    let (input, captured) = alt((
        delimited(char('{'), padded_value, char('}')),
        delimited(char('('), padded_value, char(')')),
    ))(input)?;
    Ok((input, Preamble(captured)))
}

/// Parse a single chunk.
pub fn chunk(input: &str) -> IResult<&str, Chunk> {
    // Consume unused text.
    let (input, _) = take_until_char('@')(input)?;
    alt((
        map(entry, Chunk::from),
        map(abbreviation, Chunk::from),
        map(preamble, Chunk::from),
        map(comment, Chunk::from),
    ))(input)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_identifier() {
        assert_eq!(identifier("a0 "), Ok((" ", "a0".into())));
        assert!(identifier("3key").is_err());
        assert!(identifier(" key").is_err());
    }

    #[test]
    fn test_token() {
        assert_eq!(token("{bracketed}, "), Ok((", ", Token::Text("bracketed"))));
        assert_eq!(token("\"quoted\"} "), Ok(("} ", Token::Text("quoted"))));
        assert_eq!(token("0123 #"), Ok((" #", Token::Text("0123"))));
        assert_eq!(token("key0 #"), Ok((" #", Token::Abbrev("key0".into()))));
        assert_eq!(token("0c"), Ok(("c", Token::Text("0"))));
        assert_eq!(token(r#""out{mid}""#), Ok(("", Token::Text("out{mid}"))));
        assert_eq!(
            token("{out{mid{inside}mid}}, "),
            Ok((", ", Token::Text("out{mid{inside}mid}")))
        );
        assert!(token("{bracketed{error}").is_err());
        assert!(token(r#""{open""#).is_err());
        assert!(token(r#""{closed}}""#).is_err());
        assert!(token("{{bad}").is_err());
    }

    #[test]
    fn test_field_sep() {
        assert_eq!(field_sep("     \n,  \t"), Ok(("", ())));
        assert_eq!(field_sep(", next ="), Ok(("next =", ())));
    }

    #[test]
    fn test_value() {
        assert_eq!(
            value("{first} # {second # }\n}"),
            Ok((
                "\n}",
                vec![Token::Text("first"), Token::Text("second # ")].into()
            ))
        );
        assert!(value(" {first}").is_err());
    }
}
