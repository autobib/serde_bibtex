use std::borrow::Cow;
use std::fmt;
use std::iter::FromIterator;

use itertools::Itertools;
use serde::Deserialize;

use crate::error::Error;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Deserialize)]
pub struct Identifier<'r>(#[serde(borrow)] &'r str);

impl fmt::Display for Identifier<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        self.0.fmt(f)
    }
}

impl<'r> Identifier<'r> {
    pub fn from_str_unchecked(s: &'r str) -> Self {
        Self(s)
    }

    pub fn into_raw(self) -> &'r str {
        let Self(cow) = self;
        cow
    }
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub enum Token<'r> {
    #[serde(borrow)]
    Macro(Identifier<'r>),
    #[serde(borrow)]
    Text(Cow<'r, str>),
}

impl<'r> TryFrom<Token<'r>> for Cow<'r, str> {
    type Error = Error;
    fn try_from(token: Token<'r>) -> Result<Self, Self::Error> {
        match token {
            Token::Macro(Identifier(s)) => Err(Error::UnresolvedMacro(s.to_string())),
            Token::Text(cow) => Ok(cow),
        }
    }
}

impl<'r> Token<'r> {
    pub fn macro_from(s: &'r str) -> Self {
        Token::Macro(Identifier::from_str_unchecked(s))
    }

    pub fn text_from(s: &'r str) -> Self {
        Token::Text(Cow::Borrowed(s))
    }
}

impl<'r> From<Identifier<'r>> for Token<'r> {
    fn from(identifier: Identifier<'r>) -> Self {
        Self::Macro(identifier)
    }
}

#[derive(Debug, Default, PartialEq, Clone, Deserialize)]
pub struct Value<'r>(#[serde(borrow)] pub Vec<Token<'r>>);

impl<'r> FromIterator<Token<'r>> for Value<'r> {
    fn from_iter<T: IntoIterator<Item = Token<'r>>>(i: T) -> Value<'r> {
        Self(i.into_iter().collect())
    }
}

impl fmt::Display for Value<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        let mut preceded_by_text = false;
        let mut is_first = true;
        for token in self.0.as_slice() {
            match (token, preceded_by_text) {
                (Token::Text(s), false) => {
                    if !is_first {
                        write!(f, " # ")?;
                    }
                    write!(f, "{{{}", s)?;
                    preceded_by_text = true;
                }
                (Token::Text(s), true) => {
                    write!(f, "{}", s)?;
                    preceded_by_text = true;
                }
                (Token::Macro(s), true) => {
                    write!(f, "}} # {}", s)?;
                    preceded_by_text = false;
                }
                (Token::Macro(s), false) => {
                    if !is_first {
                        write!(f, " # ")?;
                    }
                    write!(f, "{}", s)?;
                    preceded_by_text = false;
                }
            }
            is_first = false;
        }

        if preceded_by_text {
            write!(f, "}}")?;
        }

        Ok(())
    }
}

#[derive(Debug, PartialEq, Deserialize)]
pub struct Field<'r>(
    #[serde(borrow)] pub Identifier<'r>,
    #[serde(borrow)] pub Value<'r>,
);

impl fmt::Display for Field<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        write!(f, "{} = {}", self.0, self.1)
    }
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Deserialize)]
pub struct CitationKey<'r>(#[serde(borrow)] &'r str);

impl<'r> From<&'r str> for CitationKey<'r> {
    fn from(s: &'r str) -> Self {
        CitationKey(s)
    }
}

impl fmt::Display for CitationKey<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, PartialEq, Deserialize)]
pub struct Entry<'r> {
    #[serde(borrow)]
    pub entry_type: Identifier<'r>,
    #[serde(borrow)]
    pub citation_key: CitationKey<'r>,
    #[serde(borrow)]
    pub fields: Vec<Field<'r>>,
}

impl fmt::Display for Entry<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        const FIELD_SEP: &'static str = ",\n  ";

        write!(f, "@{}", self.entry_type)?;
        write!(f, "{{{}{}", self.citation_key, FIELD_SEP)?;
        write!(f, "{}", self.fields.iter().format(FIELD_SEP))?;
        write!(f, ",\n}}")
    }
}

#[derive(Debug, PartialEq)]
pub struct Macro<'r>(pub Field<'r>);

impl fmt::Display for Macro<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        write!(f, "@string{{{}}}", self.0)
    }
}

#[derive(Debug, PartialEq)]
pub struct Comment<'r>(pub &'r str);

impl fmt::Display for Comment<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        write!(f, "@comment{{{}}}", self.0)
    }
}

#[derive(Debug, PartialEq)]
pub struct Preamble<'r>(pub Value<'r>);

impl fmt::Display for Preamble<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        write!(f, "@preamble{{{}}}", self.0)
    }
}

#[derive(Debug, PartialEq)]
pub enum Event<'r> {
    Entry(Entry<'r>),
    String(Macro<'r>),
    Comment(Comment<'r>),
    Preamble(Preamble<'r>),
    Eof,
}

impl<'r> From<Entry<'r>> for Event<'r> {
    fn from(entry: Entry<'r>) -> Self {
        Event::Entry(entry)
    }
}

impl<'r> From<Macro<'r>> for Event<'r> {
    fn from(m: Macro<'r>) -> Self {
        Event::String(m)
    }
}

impl<'r> From<Comment<'r>> for Event<'r> {
    fn from(comment: Comment<'r>) -> Self {
        Event::Comment(comment)
    }
}

impl<'r> From<Preamble<'r>> for Event<'r> {
    fn from(preamble: Preamble<'r>) -> Self {
        Event::Preamble(preamble)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_value_display() {
        assert_eq!(
            Value::from_iter([Token::macro_from("auth")]).to_string(),
            "auth"
        );
        assert_eq!(
            Value::from_iter([Token::macro_from("auth")]).to_string(),
            "auth"
        );

        assert_eq!(
            Value::from_iter([
                Token::macro_from("A"),
                Token::text_from("b"),
                Token::text_from("c"),
            ])
            .to_string(),
            "A # {bc}"
        );

        assert_eq!(
            Value::from_iter([
                Token::text_from("a"),
                Token::macro_from("B"),
                Token::text_from("c"),
            ])
            .to_string(),
            "{a} # B # {c}"
        );

        assert_eq!(Value::from_iter([Token::text_from("a")]).to_string(), "{a}");

        assert_eq!(
            Value::from_iter([Token::text_from("a"), Token::macro_from("B"),]).to_string(),
            "{a} # B"
        );

        assert_eq!(
            Value::from_iter([Token::macro_from("A"), Token::macro_from("B"),]).to_string(),
            "A # B"
        );

        assert_eq!(
            Value::from_iter([
                Token::text_from("a"),
                Token::text_from("b"),
                Token::text_from("c"),
            ])
            .to_string(),
            "{abc}"
        );
    }
}
