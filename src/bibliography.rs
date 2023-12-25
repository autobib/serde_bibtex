use itertools::Itertools;
use std::fmt::{Display, Error, Formatter};

#[derive(Debug, PartialEq, Clone)]
pub struct Identifier<'a> {
    pub inner: &'a str,
}

impl Display for Identifier<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(f, "{}", self.inner)
    }
}

impl<'a> From<&'a str> for Identifier<'a> {
    fn from(inner: &'a str) -> Self {
        Identifier { inner }
    }
}

#[derive(Debug, PartialEq)]
pub enum Token<'a> {
    Abbrev(Identifier<'a>),
    Text(&'a str),
}

impl Display for Token<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        match self {
            Self::Abbrev(s) => write!(f, "{}", s),
            Self::Text(s) => write!(f, "{{{}}}", s),
        }
    }
}

impl<'a> From<Identifier<'a>> for Token<'a> {
    fn from(identifier: Identifier<'a>) -> Self {
        Self::Abbrev(identifier)
    }
}

#[derive(Debug, PartialEq)]
pub struct Value<'a> {
    pub tokens: Vec<Token<'a>>,
}

impl Display for Value<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(f, "{}", self.tokens.iter().format(" # "))
    }
}

impl<'a> From<Vec<Token<'a>>> for Value<'a> {
    fn from(tokens: Vec<Token<'a>>) -> Self {
        Value { tokens }
    }
}

#[derive(Debug, PartialEq)]
pub struct Field<'a> {
    pub identifier: Identifier<'a>,
    pub value: Value<'a>,
}

impl<'a> From<(Identifier<'a>, Value<'a>)> for Field<'a> {
    fn from((identifier, value): (Identifier<'a>, Value<'a>)) -> Self {
        Field { identifier, value }
    }
}

impl<'a> From<(Identifier<'a>, Vec<Token<'a>>)> for Field<'a> {
    fn from((identifier, tokens): (Identifier<'a>, Vec<Token<'a>>)) -> Self {
        Field {
            identifier,
            value: tokens.into(),
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct Entry<'a> {
    pub entry_type: Identifier<'a>,
    pub key: &'a str,
    pub fields: Vec<Field<'a>>,
}

#[derive(Debug, PartialEq)]
pub struct Abbreviation<'a>(pub Field<'a>);

#[derive(Debug, PartialEq)]
pub struct Comment<'a>(pub &'a str);

#[derive(Debug, PartialEq)]
pub struct Preamble<'a>(pub Value<'a>);

#[derive(Debug, PartialEq)]
pub enum Chunk<'a> {
    Entry(Entry<'a>),
    String(Abbreviation<'a>),
    Comment(Comment<'a>),
    Preamble(Preamble<'a>),
}

impl<'a> From<Entry<'a>> for Chunk<'a> {
    fn from(entry: Entry<'a>) -> Self {
        Chunk::Entry(entry)
    }
}

impl<'a> From<Abbreviation<'a>> for Chunk<'a> {
    fn from(abbrev: Abbreviation<'a>) -> Self {
        Chunk::String(abbrev)
    }
}

impl<'a> From<Comment<'a>> for Chunk<'a> {
    fn from(comment: Comment<'a>) -> Self {
        Chunk::Comment(comment)
    }
}

impl<'a> From<Preamble<'a>> for Chunk<'a> {
    fn from(preamble: Preamble<'a>) -> Self {
        Chunk::Preamble(preamble)
    }
}

pub type RawBibliography<'a> = Vec<Chunk<'a>>;
