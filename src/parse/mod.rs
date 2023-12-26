mod balanced;

use nom::{
    branch::alt,
    bytes::complete::{is_not, tag_no_case, take_until},
    character::complete::{char, digit1, multispace0},
    combinator::value as nom_value,
    combinator::{map, not, opt, verify},
    multi::{separated_list0, separated_list1},
    sequence::{delimited, pair, separated_pair, tuple},
    IResult,
};

use crate::bib::{
    Abbreviation, Comment, Entry, EntryKey, Event, Field, Identifier, Preamble, Token, Value,
};
use balanced::{is_balanced, take_until_unbalanced};

pub fn bibtex_comment(i: &str) -> IResult<&str, ()> {
    nom_value(
        (), // Output is thrown away.
        pair(char('%'), is_not("\n\r")),
    )(i)
}

/// Parse an abbreviation, which is any sequence of characters not in ` \t\\#%'\",=(){}` with
/// has length at least 1 and does not start with a digit.
/// ```
/// use serde_bibtex::parse::identifier;
/// use serde_bibtex::bib::Identifier;
///
/// assert_eq!(
///     identifier("key0"),
///     Ok(("", Identifier::from("key0")))
/// );
///
/// assert!(identifier("0key").is_err());
///
/// assert!(identifier("(i)dent").is_err());
/// ```
pub fn identifier(input: &str) -> IResult<&str, Identifier> {
    let (input, ()) = not(digit1)(input)?;
    map(is_not(" \t\\#%'\",=(){}"), Identifier::from)(input)
}

fn curly(input: &str) -> IResult<&str, &str> {
    delimited(char('{'), take_until_unbalanced('{', '}'), char('}'))(input)
}

/// Parse a field token, which is either `{curly}`, `"quoted"`, an abbreviation, or a sequence of
/// digits.
/// ```
/// use serde_bibtex::parse::token;
/// use serde_bibtex::bib::Token;
///
/// assert_eq!(
///     token("1234"),
///     Ok(("", Token::text_from("1234")))
/// );
/// ```
/// The `{` and `}` brackets need to be balanced:
/// ```
/// # use serde_bibtex::parse::token;
/// # use serde_bibtex::bib::Token;
/// assert_eq!(
///     token("\"{outside{inside}}\""),
///     Ok(("", Token::text_from("{outside{inside}}")))
/// );
///
/// assert!(token("\"{unbalanced\"").is_err())
/// ```
/// For a `{curly}` token, the parser eats characters until the brackets are balanced.
/// ```
/// # use serde_bibtex::parse::token;
/// # use serde_bibtex::bib::Token;
/// assert_eq!(token("{a{b} }}"), Ok(("}", Token::text_from("a{b} "))));
/// ```
pub fn token(input: &str) -> IResult<&str, Token> {
    let quoted = delimited(
        char('"'),
        verify(take_until("\""), is_balanced('{', '}')),
        char('"'),
    );

    alt((
        map(curly, Token::text_from),
        map(quoted, Token::text_from),
        map(digit1, Token::text_from),
        map(identifier, Token::Abbrev),
    ))(input)
}

/// Parse a field value by splitting at '#', and removing excess whitespace.
/// ```
/// use serde_bibtex::parse::value;
/// use serde_bibtex::bib::{Token, Value};
/// assert_eq!(
///     value("123 # abbrev # {bracketed} # \"quoted\","),
///     Ok((",", Value(vec![Token::text_from("123"), Token::Abbrev("abbrev".into()), Token::text_from("bracketed"), Token::text_from("quoted")])))
/// );
/// ```
/// The value cannot have leading whitespace and must contain at least one valid identifier.
/// ```
/// # use serde_bibtex::parse::value;
/// assert!(value(" 123").is_err());
/// assert!(value("}").is_err());
/// ```
pub fn value(input: &str) -> IResult<&str, Value> {
    let (input, tokens) =
        separated_list1(tuple((multispace0, char('#'), multispace0)), token)(input)?;
    Ok((input, Value(tokens)))
}

/// Parse a field `abbrev = {value}`.
/// ```
/// use serde_bibtex::parse::field;
/// use serde_bibtex::bib::{Token, Value, Field, Identifier};
///
/// assert_eq!(
///     field("title = {A} # \"Title\""),
///     Ok((
///         "",
///         Field {
///             identifier: Identifier::from("title"),
///             value: Value(vec![Token::text_from("A"), Token::text_from("Title")])
///         }
///     ))
/// );
/// ```
pub fn field(input: &str) -> IResult<&str, Field> {
    map(
        separated_pair(
            identifier,
            tuple((multispace0, char('='), multispace0)),
            value,
        ),
        |(identifier, value)| Field { identifier, value },
    )(input)
}

/// Consume a field separator.
fn field_sep(input: &str) -> IResult<&str, ()> {
    nom_value((), tuple((multispace0, char(','), multispace0)))(input)
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

/// Parse an `@entry` event.
/// ```
/// use serde_bibtex::parse::entry;
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
            key: EntryKey::from(key),
            fields,
        },
    ))
}

fn padded_field(input: &str) -> IResult<&str, Field> {
    let (input, (_, captured, _)) = tuple((multispace0, field, multispace0))(input)?;
    Ok((input, captured))
}

/// Parse an `@string` event.
pub fn abbreviation(input: &str) -> IResult<&str, Abbreviation> {
    let (input, _) = tuple((char('@'), multispace0, tag_no_case("string"), multispace0))(input)?;
    let (input, captured) = alt((
        delimited(char('{'), padded_field, char('}')),
        delimited(char('('), padded_field, char(')')),
    ))(input)?;
    Ok((input, Abbreviation(captured)))
}

/// Parse an `@comment` event.
/// ```
/// use serde_bibtex::parse::comment;
/// use serde_bibtex::bib::{Comment, Token};
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
            verify(take_until(")"), is_balanced('{', '}')),
            char(')'),
        ),
        Comment,
    );
    let (input, (_, _, _, comment)) = tuple((
        char('@'),
        multispace0,
        tag_no_case("comment"),
        alt((curly, round)),
    ))(input)?;
    Ok((input, comment))
}

/// Parse a token, but with unmatched brackets permitted inside quoted fields.
fn token_lenient(input: &str) -> IResult<&str, Token> {
    let quoted = delimited(char('"'), take_until("\""), char('"'));
    alt((
        map(curly, Token::text_from),
        map(quoted, Token::text_from),
        map(digit1, Token::text_from),
        map(identifier, Token::Abbrev),
    ))(input)
}

/// Parse a value surrounded by ignored whitespace
fn padded_value_lenient(input: &str) -> IResult<&str, Value> {
    let value_lenient =
        separated_list1(tuple((multispace0, char('#'), multispace0)), token_lenient);

    let (input, (_, tokens, _)) = tuple((multispace0, value_lenient, multispace0))(input)?;
    Ok((input, Value(tokens)))
}

/// Parse an `@preamble` event. Note that unmatched brackets inside quoted `Token`s are allowed.
/// Compare the behaviour with `@comment` or `@entry`.
/// ```
/// use serde_bibtex::bib::{Token, Preamble, Value, Identifier};
/// use serde_bibtex::parse::preamble;
/// let bibfile = r#"@preamble{"\mymacro{" # A # "}"}"#;
///
/// assert_eq!(
///     preamble(bibfile),
///     Ok((
///         "",
///         Preamble(Value(vec![
///             Token::text_from(r#"\mymacro{"#),
///             Token::abbrev_from("A"),
///             Token::text_from("}")
///         ]))
///     ))
/// );
/// ```
pub fn preamble(input: &str) -> IResult<&str, Preamble> {
    let (input, _) = tuple((char('@'), multispace0, tag_no_case("preamble"), multispace0))(input)?;
    let (input, captured) = alt((
        delimited(char('{'), padded_value_lenient, char('}')),
        delimited(char('('), padded_value_lenient, char(')')),
    ))(input)?;
    Ok((input, Preamble(captured)))
}

/// Parse a single bibliography event.
/// ```
/// use serde_bibtex::bib::{Event, Token, Abbreviation, Value, Field, Entry};
/// use serde_bibtex::parse::read_event;
///
/// let bibfile = r#"
///   @article{key:0,
///     author = A1 # " and " # A2,
///     title = {A title},
///     year = 2014,
///   }"#;
///
/// let entry = Entry {
///     entry_type: "article".into(),
///     key: "key:0".into(),
///     fields: vec![
///         Field {
///             identifier: "author".into(),
///             value: Value(vec![
///                 Token::abbrev_from("A1"),
///                 Token::text_from(" and "),
///                 Token::abbrev_from("A2"),
///             ]),
///         },
///         Field {
///             identifier: "title".into(),
///             value: Value(vec![Token::text_from("A title")]),
///         },
///         Field {
///             identifier: "year".into(),
///             value: Value(vec![Token::text_from("2014")]),
///         },
///     ],
/// };
///
/// assert_eq!(read_event(&bibfile), Ok(("", Event::from(entry))));
/// ```
pub fn read_event(input: &str) -> IResult<&str, Event> {
    let (input, captured) = opt(take_until("@"))(input)?;
    match captured {
        Some(_) => alt((
            map(entry, Event::from),
            map(abbreviation, Event::from),
            map(preamble, Event::from),
            map(comment, Event::from),
        ))(input),
        None => Ok(("", Event::Eof)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
                Value(vec![
                    Token::text_from("first"),
                    Token::text_from("second # ")
                ])
            ))
        );
        assert!(value(" {first}").is_err());
    }

    #[test]
    fn test_event() {
        use crate::bib::{Event, Token};

        let bibfile = r#"
          @string{A = "Author"}

          @article{key:0,
            author = "One, " # A,
            title = {A title},
            year = 2014,
          }"#;

        let abbrev = Abbreviation(Field {
            identifier: Identifier::from("A"),
            value: Value(vec![Token::text_from("Author")]),
        });

        let entry = Entry {
            entry_type: Identifier::from("article"),
            key: EntryKey::from("key:0"),
            fields: vec![
                Field {
                    identifier: Identifier::from("author"),
                    value: Value(vec![
                        Token::text_from("One, "),
                        Token::Abbrev(Identifier::from("A")),
                    ]),
                },
                Field {
                    identifier: Identifier::from("title"),
                    value: Value(vec![Token::text_from("A title")]),
                },
                Field {
                    identifier: Identifier::from("year"),
                    value: Value(vec![Token::text_from("2014")]),
                },
            ],
        };
        let (bibfile, parsed) = read_event(&bibfile).unwrap();
        assert_eq!(parsed, Event::from(abbrev));

        let (bibfile, parsed) = read_event(&bibfile).unwrap();
        assert_eq!(parsed, Event::from(entry));
        assert_eq!(bibfile, "");
    }
}
