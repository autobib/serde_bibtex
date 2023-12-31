use std::borrow::Cow;
use std::fmt;
// ::{fmt::Display, Error, fmt::Formatter};
use std::iter::FromIterator;

use itertools::Itertools;

use crate::error::Error;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Identifier<'r>(Cow<'r, str>);

impl<'r> From<&'r str> for Identifier<'r> {
    fn from(s: &'r str) -> Self {
        Identifier(Cow::Borrowed(s))
    }
}

impl fmt::Display for Identifier<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        self.0.fmt(f)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Token<'r> {
    Abbrev(Identifier<'r>),
    Text(Cow<'r, str>),
}

impl<'r> TryFrom<Token<'r>> for Cow<'r, str> {
    type Error = Error;
    fn try_from(token: Token<'r>) -> Result<Self, Self::Error> {
        match token {
            Token::Abbrev(Identifier(cow)) => Err(Error::UnresolvedAbbreviation(cow.to_string())),
            Token::Text(cow) => Ok(cow),
        }
    }
}

impl<'r> Token<'r> {
    pub fn abbrev_from(s: &'r str) -> Self {
        Token::Abbrev(Identifier::from(s))
    }

    pub fn text_from(s: &'r str) -> Self {
        Token::Text(Cow::Borrowed(s))
    }
}

impl<'r> From<Identifier<'r>> for Token<'r> {
    fn from(identifier: Identifier<'r>) -> Self {
        Self::Abbrev(identifier)
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum Value<'r> {
    Seq(Vec<Token<'r>>),
    Unit(Token<'r>),
}

impl<'r> FromIterator<Token<'r>> for Value<'r> {
    fn from_iter<T: IntoIterator<Item = Token<'r>>>(i: T) -> Value<'r> {
        let mut iter = i.into_iter();

        match (iter.next(), iter.next()) {
            (None, _) => Value::Unit(Token::Text(Cow::Owned(String::new()))),
            (Some(token), None) => Value::Unit(token),
            (Some(token1), Some(token2)) => {
                let mut vec: Vec<Token<'r>> = vec![token1, token2];
                vec.extend(iter);
                Value::Seq(vec)
            }
        }
    }
}

impl fmt::Display for Value<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        match self {
            Self::Unit(Token::Abbrev(ident)) => write!(f, "{}", ident),
            Self::Unit(Token::Text(cow)) => write!(f, "{{{}}}", cow),
            Self::Seq(vec) => {
                let mut preceded_by_text = false;
                let mut is_first = true;
                for token in vec {
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
                        (Token::Abbrev(s), true) => {
                            write!(f, "}} # {}", s)?;
                            preceded_by_text = false;
                        }
                        (Token::Abbrev(s), false) => {
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
    }
}

impl<'r> Value<'r> {
    pub fn resolve(&mut self) -> Result<(), Error> {
        match self {
            Value::Unit(Token::Text(_)) => {}
            Value::Unit(Token::Abbrev(Identifier(cow))) => {
                return Err(Error::UnresolvedAbbreviation(cow.to_string()));
            }
            Value::Seq(ref mut tokens) => {
                if tokens.len() == 1 {
                    let only = tokens.remove(0);
                    match only {
                        Token::Text(_) => *self = Self::Unit(only),
                        Token::Abbrev(Identifier(cow)) => {
                            return Err(Error::UnresolvedAbbreviation(cow.to_string()));
                        }
                    }
                } else {
                    let mut ret = String::new();
                    for token in tokens.iter() {
                        match token {
                            Token::Text(cow) => {
                                ret.push_str(&cow);
                            }
                            Token::Abbrev(Identifier(cow)) => {
                                return Err(Error::UnresolvedAbbreviation(cow.to_string()));
                            }
                        }
                    }
                    *self = Self::Unit(Token::Text(ret.into()));
                }
            }
        }
        Ok(())
    }

    pub fn as_unit<'a>(&'a mut self) -> Result<&'a str, Error> {
        self.resolve()?;

        match self {
            Value::Unit(Token::Text(ref cow)) => Ok(cow),
            // SAFETY: If resolve() succeeds, self must be a Value::Unit(Token::Text(_))
            _ => unreachable!(),
        }
    }

    pub fn into_unit(mut self) -> Result<Cow<'r, str>, Error> {
        self.resolve()?;

        match self {
            Value::Unit(Token::Text(cow)) => Ok(cow),
            // SAFETY: If resolve() succeeds, self must be a Value::Unit(Token::Text(_))
            _ => unreachable!(),
        }
    }

    pub fn is_unit(&self) -> bool {
        matches!(self, Self::Unit(Token::Text(_)))
    }
}

#[derive(Debug, PartialEq)]
pub struct Field<'r> {
    pub identifier: Identifier<'r>,
    pub value: Value<'r>,
}

impl fmt::Display for Field<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        write!(f, "{} = {}", self.identifier, self.value)
    }
}

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub struct EntryKey<'r>(Cow<'r, str>);

impl<'r> From<&'r str> for EntryKey<'r> {
    fn from(s: &'r str) -> Self {
        EntryKey(Cow::Borrowed(s))
    }
}

impl fmt::Display for EntryKey<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, PartialEq)]
pub struct Entry<'r> {
    pub entry_type: Identifier<'r>,
    pub key: EntryKey<'r>,
    pub fields: Vec<Field<'r>>,
}

impl fmt::Display for Entry<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        let align = ",\n  ";

        write!(f, "@{}", self.entry_type)?;
        write!(f, "{{{}{}", self.key, align)?;
        write!(f, "{}", self.fields.iter().format(align))?;
        write!(f, ",\n}}")
    }
}

#[derive(Debug, PartialEq)]
pub struct Abbreviation<'r>(pub Field<'r>);

impl fmt::Display for Abbreviation<'_> {
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
    String(Abbreviation<'r>),
    Comment(Comment<'r>),
    Preamble(Preamble<'r>),
    Eof,
}

impl<'r> From<Entry<'r>> for Event<'r> {
    fn from(entry: Entry<'r>) -> Self {
        Event::Entry(entry)
    }
}

impl<'r> From<Abbreviation<'r>> for Event<'r> {
    fn from(abbrev: Abbreviation<'r>) -> Self {
        Event::String(abbrev)
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
    fn test_value_resolve() {
        // The match asserts are necessary since Cow is transparent

        // a vector of Token::Text are merged, which requires Owning
        let mut value = Value::Seq(vec![
            Token::text_from("a"),
            Token::text_from("b"),
            Token::text_from("c"),
        ]);
        value.resolve().unwrap();
        assert!(matches!(value, Value::Unit(Token::Text(Cow::Owned(_)))));
        assert_eq!(value, Value::Unit(Token::text_from("abc")));

        // A single Token::Text in a vector can be borrowed
        let mut value = Value::Seq(vec![Token::text_from("a")]);
        value.resolve().unwrap();
        assert!(matches!(value, Value::Unit(Token::Text(Cow::Borrowed(_)))));
        assert_eq!(value, Value::Unit(Token::text_from("a")));

        // A single Token::Text in a unit can be borrowed
        let mut value = Value::Unit(Token::text_from("a"));
        value.resolve().unwrap();
        assert!(matches!(value, Value::Unit(Token::Text(Cow::Borrowed(_)))));
        assert_eq!(value, Value::Unit(Token::text_from("a")));

        // A sequence value containing an empty vector resolves to ""
        let mut value = Value::Seq(Vec::new());
        value.resolve().unwrap();
        assert!(matches!(value, Value::Unit(Token::Text(Cow::Owned(_)))));
        assert_eq!(value, Value::Unit(Token::text_from("")));

        // Abbreviations cannot be resolved
        assert!(Value::from_iter([Token::abbrev_from("A")])
            .resolve()
            .is_err());

        assert!(
            Value::from_iter([Token::text_from("a"), Token::abbrev_from("B")])
                .resolve()
                .is_err()
        );

        // Failed resolution does not mutate the Value
        let mut value = Value::from_iter([
            Token::text_from("a"),
            Token::text_from("b"),
            Token::abbrev_from("C"),
            Token::text_from("d"),
        ]);
        let value_copy = value.clone();
        assert!(value.resolve().is_err());
        assert_eq!(value, value_copy);
    }

    #[test]
    fn test_value_from_iter() {
        // the empty Value defaults to an empty owned string
        let value = Value::from_iter([]);
        assert!(matches!(value, Value::Unit(Token::Text(Cow::Owned(_)))));
        assert_eq!(value, Value::Unit(Token::text_from("")));

        assert_eq!(
            Value::from_iter([Token::abbrev_from("a")]),
            Value::Unit(Token::abbrev_from("a"))
        );
        assert_eq!(
            Value::from_iter([Token::text_from("b")]),
            Value::Unit(Token::text_from("b"))
        );
        assert_eq!(
            Value::from_iter([Token::abbrev_from("a"), Token::text_from("b")]),
            Value::Seq(vec![Token::abbrev_from("a"), Token::text_from("b")])
        );
    }

    #[test]
    fn test_value_display() {
        assert_eq!(
            Value::Seq(vec![Token::abbrev_from("auth")]).to_string(),
            "auth"
        );
        assert_eq!(Value::Unit(Token::abbrev_from("auth")).to_string(), "auth");

        assert_eq!(
            Value::Seq(vec![
                Token::abbrev_from("A"),
                Token::text_from("b"),
                Token::text_from("c"),
            ])
            .to_string(),
            "A # {bc}"
        );

        assert_eq!(
            Value::Seq(vec![
                Token::text_from("a"),
                Token::abbrev_from("B"),
                Token::text_from("c"),
            ])
            .to_string(),
            "{a} # B # {c}"
        );

        assert_eq!(Value::Seq(vec![Token::text_from("a"),]).to_string(), "{a}");

        assert_eq!(
            Value::Seq(vec![Token::text_from("a"), Token::abbrev_from("B"),]).to_string(),
            "{a} # B"
        );

        assert_eq!(
            Value::Seq(vec![Token::abbrev_from("A"), Token::abbrev_from("B"),]).to_string(),
            "A # B"
        );

        assert_eq!(
            Value::Seq(vec![
                Token::text_from("a"),
                Token::text_from("b"),
                Token::text_from("c"),
            ])
            .to_string(),
            "{abc}"
        );
    }

    #[test]
    fn test_field_display() {
        assert_eq!(
            Field {
                identifier: Identifier::from("title"),
                value: Value::Seq(vec![Token::abbrev_from("a"), Token::text_from("b"),])
            }
            .to_string(),
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
                    value: Value::Seq(vec![
                        Token::text_from("One, "),
                        Token::Abbrev(Identifier::from("A")),
                    ]),
                },
                Field {
                    identifier: Identifier::from("title"),
                    value: Value::Seq(vec![Token::text_from("A title")]),
                },
                Field {
                    identifier: Identifier::from("year"),
                    value: Value::Seq(vec![Token::text_from("2014")]),
                },
            ],
        };
        assert_eq!(
            entry.to_string(),
            "@article{key:0,\n  author = {One, } # A,\n  title = {A title},\n  year = {2014},\n}"
        )
    }
}
