use nom::{
    branch::alt,
    character::complete::{char, digit1, multispace0},
    combinator::map,
    multi::separated_list1,
    sequence::{delimited, tuple},
    IResult,
};

use super::balanced::{take_until_char, take_until_unbalanced};
use crate::bibliography::{Identifier, Token, Value};

pub fn identifier(input: &str) -> IResult<&str, Identifier> {
    todo!()
}

pub fn token(input: &str) -> IResult<&str, Token> {
    let curly = map(
        delimited(char('{'), take_until_unbalanced('{', '}'), char('}')),
        Token::Text,
    );

    let quoted = map(
        delimited(char('"'), take_until_char('"'), char('"')),
        Token::Text,
    );

    alt((
        curly,
        quoted,
        map(digit1, Token::Text),
        map(identifier, Token::from),
    ))(input)
}

pub fn value_lenient(input: &str) -> IResult<&str, Value> {
    let (input, tokens) =
        separated_list1(tuple((multispace0, char('#'), multispace0)), token)(input)?;
    Ok((input, tokens.into()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_token() {
        assert_eq!(
            token(r#""{mismatched brackets" #"#),
            Ok((" #", Token::Text("{mismatched brackets")))
        );
        assert_eq!(token(r#""{open""#), Ok(("", Token::Text("{open"))));
        assert_eq!(token(r#""{closed}}""#), Ok(("", Token::Text("{closed}}"))));
    }
}
