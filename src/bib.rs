use std::borrow::Cow;
use std::fmt::{Display, Error, Formatter};

use itertools::Itertools;

use crate::error::ConversionError;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Identifier<'de>(Cow<'de, str>);

impl<'de> From<&'de str> for Identifier<'de> {
    fn from(s: &'de str) -> Self {
        Identifier(Cow::Borrowed(s))
    }
}

impl Display for Identifier<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Token<'de> {
    Abbrev(Identifier<'de>),
    Text(Cow<'de, str>),
}

impl<'de> TryFrom<Token<'de>> for Cow<'de, str> {
    type Error = ConversionError;
    fn try_from(token: Token<'de>) -> Result<Self, Self::Error> {
        match token {
            Token::Abbrev(_) => Err(ConversionError::NotText),
            Token::Text(cow) => Ok(cow),
        }
    }
}

impl<'de> Token<'de> {
    pub fn abbrev_from(s: &'de str) -> Self {
        Token::Abbrev(Identifier::from(s))
    }

    pub fn text_from(s: &'de str) -> Self {
        Token::Text(Cow::Borrowed(s))
    }
}

impl Display for Token<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        match self {
            Self::Abbrev(s) => write!(f, "{}", s),
            Self::Text(s) => write!(f, "{{{}}}", s),
        }
    }
}

impl<'de> From<Identifier<'de>> for Token<'de> {
    fn from(identifier: Identifier<'de>) -> Self {
        Self::Abbrev(identifier)
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct Value<'de>(pub Vec<Token<'de>>);

impl Display for Value<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(f, "{}", self.0.iter().format(" # "))
    }
}

impl<'de> IntoIterator for Value<'de> {
    type Item = Token<'de>;
    type IntoIter = std::vec::IntoIter<Token<'de>>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<'de> TryFrom<Value<'de>> for Cow<'de, str> {
    type Error = ConversionError;
    fn try_from(mut value: Value<'de>) -> Result<Self, Self::Error> {
        if value.0.len() == 1 {
            Ok(Cow::try_from(value.0.remove(0))?)
        } else {
            let mut ret = String::new();
            for token in value.0.drain(..) {
                if let Token::Text(cow) = token {
                    ret.push_str(&cow);
                } else {
                    return Err(ConversionError::NotText);
                }
            }
            Ok(ret.into())
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct Field<'de> {
    pub identifier: Identifier<'de>,
    pub value: Value<'de>,
}

impl Display for Field<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(f, "{} = {}", self.identifier, self.value)
    }
}

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub struct EntryKey<'de>(Cow<'de, str>);

impl<'de> From<&'de str> for EntryKey<'de> {
    fn from(s: &'de str) -> Self {
        EntryKey(Cow::Borrowed(s))
    }
}

impl Display for EntryKey<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, PartialEq)]
pub struct Entry<'de> {
    pub entry_type: Identifier<'de>,
    pub key: EntryKey<'de>,
    pub fields: Vec<Field<'de>>,
}

impl Display for Entry<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        let align = ",\n  ";

        write!(f, "@{}", self.entry_type)?;
        write!(f, "{{{}{}", self.key, align)?;
        write!(f, "{}", self.fields.iter().format(align))?;
        write!(f, ",\n}}")
    }
}

#[derive(Debug, PartialEq)]
pub struct Abbreviation<'de>(pub Field<'de>);

impl Display for Abbreviation<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(f, "@string{{{}}}", self.0)
    }
}

#[derive(Debug, PartialEq)]
pub struct Comment<'de>(pub &'de str);

impl Display for Comment<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(f, "@comment{{{}}}", self.0)
    }
}

#[derive(Debug, PartialEq)]
pub struct Preamble<'de>(pub Value<'de>);

impl Display for Preamble<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(f, "@preamble{{{}}}", self.0)
    }
}

#[derive(Debug, PartialEq)]
pub enum Event<'de> {
    Entry(Entry<'de>),
    String(Abbreviation<'de>),
    Comment(Comment<'de>),
    Preamble(Preamble<'de>),
    Eof,
}

impl<'de> From<Entry<'de>> for Event<'de> {
    fn from(entry: Entry<'de>) -> Self {
        Event::Entry(entry)
    }
}

impl<'de> From<Abbreviation<'de>> for Event<'de> {
    fn from(abbrev: Abbreviation<'de>) -> Self {
        Event::String(abbrev)
    }
}

impl<'de> From<Comment<'de>> for Event<'de> {
    fn from(comment: Comment<'de>) -> Self {
        Event::Comment(comment)
    }
}

impl<'de> From<Preamble<'de>> for Event<'de> {
    fn from(preamble: Preamble<'de>) -> Self {
        Event::Preamble(preamble)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_value_into_cow() {
        let value = Value(vec![
            Token::text_from("a"),
            Token::text_from("b"),
            Token::text_from("c"),
        ]);

        assert!(matches!(Cow::try_from(value.clone()), Ok(Cow::Owned(_))));
        assert_eq!(Cow::try_from(value), Ok(Cow::Owned("abc".to_string())),);

        let value = Value(vec![Token::text_from("a")]);
        assert!(matches!(Cow::try_from(value.clone()), Ok(Cow::Borrowed(_))));
        assert_eq!(Cow::try_from(value.clone()), Ok(Cow::Borrowed("a")));

        let value = Value(vec![Token::abbrev_from("A")]);
        assert!(Cow::try_from(value).is_err());

        let value = Value(vec![Token::text_from("a"), Token::abbrev_from("B")]);
        assert!(Cow::try_from(value).is_err());
    }

    #[test]
    fn test_identifier_display() {
        assert_eq!(format!("{}", Identifier::from("A")), "A");
    }

    #[test]
    fn test_token_display() {
        assert_eq!(format!("{}", Token::abbrev_from("auth")), "auth");

        assert_eq!(format!("{}", Token::text_from("contents")), "{contents}");

        assert_eq!(format!("{}", Token::text_from("{")), "{{}");
    }

    #[test]
    fn test_value_display() {
        assert_eq!(
            format!("{}", Value(vec![Token::abbrev_from("auth")])),
            "auth"
        );

        assert_eq!(
            format!(
                "{}",
                Value(vec![
                    Token::abbrev_from("a"),
                    Token::text_from("b"),
                    Token::text_from("c"),
                ])
            ),
            "a # {b} # {c}"
        );
    }

    #[test]
    fn test_field_display() {
        assert_eq!(
            format!(
                "{}",
                Field {
                    identifier: Identifier::from("title"),
                    value: Value(vec![Token::abbrev_from("a"), Token::text_from("b"),])
                }
            ),
            "title = a # {b}"
        );
    }

    #[test]
    fn test_entry_display() {
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
        assert_eq!(
            format!("{}", entry),
            "@article{key:0,\n  author = {One, } # A,\n  title = {A title},\n  year = {2014},\n}"
        )
    }
}
