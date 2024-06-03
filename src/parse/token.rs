//! Fundamental components of a bibliography.
use unicase::UniCase;

use super::{Identifier, Text};

use crate::error::{Error, ErrorCode, Result};
use crate::validate::{
    check_balanced, check_entry_key, check_entry_type, check_field_key, check_variable,
};

/// Entry type, such as `article` in `@article{...`.
/// 1. Case-insensitive.
/// 2. Does not contain a char in `"{}(),=\\#%\""`.
#[derive(Debug, Clone, PartialEq)]
pub enum EntryType<S: AsRef<str>> {
    /// A `preamble` entry type.
    Preamble,
    /// A `comment` entry type.
    Comment,
    /// A `string` entry type.
    Macro,
    /// Any other entry type.
    Regular(UniCase<S>),
}

impl<S: AsRef<str>> EntryType<S> {
    pub(crate) fn new_unchecked(s: S) -> Self {
        let uni = UniCase::unicode(s);
        if uni == UniCase::ascii("preamble") {
            Self::Preamble
        } else if uni == UniCase::ascii("comment") {
            Self::Comment
        } else if uni == UniCase::ascii("string") {
            Self::Macro
        } else {
            Self::Regular(uni)
        }
    }

    /// Construct a new entry type, checking that the input satisfies the requirements.
    pub fn new(s: S) -> Result<Self> {
        check_entry_type(s.as_ref())?;
        Ok(Self::new_unchecked(s))
    }
}

impl<S: AsRef<str>> From<Identifier<S>> for EntryType<S> {
    fn from(Identifier(s): Identifier<S>) -> Self {
        Self::new_unchecked(s)
    }
}

/// Macro variable, such as `var` in `@string{var = ...}`.
/// 1. Case-insensitive.
/// 2. Does not contain a char in `"{}(),=\\#%\""`.
/// 3. Does not begin with an ASCII digit.
#[derive(Debug, Clone, Eq, Hash, PartialEq)]
pub struct Variable<S: AsRef<str>>(pub UniCase<S>);

impl<S: AsRef<str>> Variable<S> {
    #[inline]
    pub(crate) fn new_unchecked(s: S) -> Self {
        Self(UniCase::unicode(s))
    }

    /// Construct a new variable, checking that the input satisfies the requirements.
    pub fn new(s: S) -> Result<Self> {
        check_variable(s.as_ref())?;
        Ok(Self(UniCase::new(s)))
    }

    /// Convert to an owned `String` variant.
    pub fn own(&self) -> Variable<String> {
        let Variable(s) = self;
        Variable::new_unchecked(s.as_ref().to_string())
    }
}

impl<S: AsRef<str>> AsRef<str> for Variable<S> {
    fn as_ref(&self) -> &str {
        self.0.as_ref()
    }
}

impl<S: AsRef<str>> From<Identifier<S>> for Variable<S> {
    fn from(id: Identifier<S>) -> Self {
        let Identifier(s) = id;
        Variable(UniCase::new(s))
    }
}

/// Entry key, such as `key` in `@article{key,....`.
/// 1. Case-sensitive.
/// 2. Does not contain a char in `"{}(),=\\#%\""`.
#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub struct EntryKey<S: AsRef<str>>(pub S);

impl<S: AsRef<str>> From<Identifier<S>> for EntryKey<S> {
    fn from(id: Identifier<S>) -> Self {
        let Identifier(s) = id;
        EntryKey(s)
    }
}

impl<S: AsRef<str>> EntryKey<S> {
    #[inline]
    pub(crate) fn new_unchecked(s: S) -> Self {
        Self(s)
    }

    /// Construct a new entry key, checking that the input satisfies the requirements.
    pub fn new(s: S) -> Result<Self> {
        check_entry_key(s.as_ref())?;
        Ok(Self::new_unchecked(s))
    }
}

/// Field key, such as `key` in `... key = {value}, ...`.
/// 1. Case-insensitive.
/// 2. Does not contain a char in `"{}(),=\\#%\""`.
#[derive(Debug, Clone, PartialEq)]
pub struct FieldKey<S: AsRef<str>>(pub UniCase<S>);

impl<S: AsRef<str>> FieldKey<S> {
    #[inline]
    pub(crate) fn new_unchecked(s: S) -> Self {
        Self(UniCase::unicode(s))
    }

    /// Construct a new field key, checking that the input satisfies the requirements.
    pub fn new(s: S) -> Result<Self> {
        check_field_key(s.as_ref())?;
        Ok(Self(UniCase::new(s)))
    }
}

impl<S: AsRef<str>> From<Identifier<S>> for FieldKey<S> {
    #[inline]
    fn from(id: Identifier<S>) -> Self {
        let Identifier(s) = id;
        Self::new_unchecked(s)
    }
}

/// A value token representing one part of a value `{Title } # 2012 # var`.
#[derive(Debug, Clone, PartialEq)]
pub enum Token<S: AsRef<str>, B: AsRef<[u8]>> {
    /// A macro variable.
    Variable(Variable<S>),
    /// A text token.
    Text(Text<S, B>),
}

impl<S, B> Token<S, B>
where
    S: AsRef<str>,
    B: AsRef<[u8]>,
{
    #[inline]
    #[allow(dead_code)]
    pub(crate) fn variable_unchecked(s: S) -> Self {
        Token::Variable(Variable::new_unchecked(s))
    }

    #[inline]
    pub(crate) fn str_unchecked(s: S) -> Self {
        Token::Text(Text::Str(s))
    }

    /// Construct a new variable variant.
    pub fn variable(s: S) -> Result<Self> {
        Ok(Token::Variable(Variable::new(s)?))
    }

    /// Construct a new text string variant.
    pub fn str(s: S) -> Result<Self> {
        check_balanced(s.as_ref().as_bytes())?;
        Ok(Token::Text(Text::Str(s)))
    }

    /// Construct a new text bytes variant.
    pub fn bytes(b: B) -> Result<Self> {
        check_balanced(b.as_ref())?;
        Ok(Token::Text(Text::Bytes(b)))
    }

    /// Convert to an owned `String` variant.
    pub fn own(value: &Token<S, B>) -> Token<String, Vec<u8>> {
        match value {
            Token::Variable(Variable(s)) => {
                Token::Variable(Variable::new_unchecked(s.as_ref().to_string()))
            }
            Token::Text(text) => Token::Text(text.own()),
        }
    }
}

impl<'r> TryFrom<Token<&'r str, &'r [u8]>> for &'r str {
    type Error = Error;
    fn try_from(token: Token<&'r str, &'r [u8]>) -> Result<Self> {
        match token {
            Token::Variable(Variable(s)) => {
                Err(Error::syntax(ErrorCode::UnresolvedMacro(s.to_string())))
            }
            Token::Text(text) => Ok(text.into_str()?),
        }
    }
}

impl<'r> TryFrom<Token<&'r str, &'r [u8]>> for &'r [u8] {
    type Error = Error;
    fn try_from(token: Token<&'r str, &'r [u8]>) -> Result<Self> {
        match token {
            Token::Variable(Variable(s)) => Err(Error::syntax(ErrorCode::UnresolvedMacro(
                s.as_ref().to_string(),
            ))),
            Token::Text(text) => Ok(text.into_bytes()),
        }
    }
}

impl<S: AsRef<str>, B: AsRef<[u8]>> TryFrom<Token<S, B>> for Text<S, B> {
    type Error = Error;
    fn try_from(token: Token<S, B>) -> Result<Self> {
        match token {
            Token::Variable(Variable(s)) => Err(Error::syntax(ErrorCode::UnresolvedMacro(
                s.as_ref().to_string(),
            ))),
            Token::Text(text) => Ok(text),
        }
    }
}
