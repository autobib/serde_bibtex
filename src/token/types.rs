//! Fundamental components of a bibliography.
use unicase::UniCase;

use super::{
    check_balanced, check_entry_key, check_entry_type, check_field_key, check_variable,
    ConversionError, TokenParseError,
};

#[derive(Debug)]
pub struct Identifier<S: AsRef<str>>(pub S);

/// The core Text handling object.
#[derive(Debug, Clone, PartialEq)]
pub enum Text<S: AsRef<str>, B: AsRef<[u8]>> {
    Str(S),
    Bytes(B),
}

impl<S, B> Text<S, B>
where
    S: AsRef<str>,
    B: AsRef<[u8]>,
{
    /// Convert the text token into an owned variant.
    pub fn own(&self) -> Text<String, Vec<u8>> {
        match self {
            Text::Str(s) => Text::Str(s.as_ref().to_string()),
            Text::Bytes(b) => Text::Bytes(b.as_ref().to_vec()),
        }
    }
}

impl<'r> Text<&'r str, &'r [u8]> {
    /// Attempt to convert into a string slice.
    pub fn into_str(self) -> Result<&'r str, std::str::Utf8Error> {
        match self {
            Self::Str(s) => Ok(s),
            Self::Bytes(b) => Ok(std::str::from_utf8(b)?),
        }
    }

    /// Convert into raw bytes.
    pub fn into_bytes(self) -> &'r [u8] {
        match self {
            Self::Str(s) => s.as_bytes(),
            Self::Bytes(bytes) => bytes,
        }
    }
}

impl<S: AsRef<str>, B: AsRef<[u8]>> Text<S, B> {
    /// Check if the token is empty.
    pub fn is_empty(&self) -> bool {
        match self {
            Self::Str(s) => s.as_ref().is_empty(),
            Self::Bytes(b) => b.as_ref().is_empty(),
        }
    }

    /// How many bytes does the token consist of?
    pub fn len(&self) -> usize {
        match self {
            Self::Str(s) => s.as_ref().len(),
            Self::Bytes(b) => b.as_ref().len(),
        }
    }
}

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
    pub fn new(input: S) -> Result<Self, TokenParseError<S>> {
        match check_entry_type(input.as_ref()) {
            Ok(()) => Ok(Self::new_unchecked(input)),
            Err(error) => Err(TokenParseError { input, error }),
        }
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
pub struct Variable<S: AsRef<str>>(UniCase<S>);

impl<S: AsRef<str>> Variable<S> {
    #[inline]
    pub(crate) fn new_unchecked(s: S) -> Self {
        Self(UniCase::unicode(s))
    }

    /// Construct a new variable, checking that the input satisfies the requirements.
    pub fn new(input: S) -> Result<Self, TokenParseError<S>> {
        match check_variable(input.as_ref()) {
            Ok(()) => Ok(Self::new_unchecked(input)),
            Err(error) => Err(TokenParseError { input, error }),
        }
    }

    /// Return the inner type.
    pub fn into_inner(self) -> S {
        self.0.into_inner()
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
pub struct EntryKey<S: AsRef<str>>(S);

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
    pub fn new(input: S) -> Result<Self, TokenParseError<S>> {
        match check_entry_key(input.as_ref()) {
            Ok(()) => Ok(Self::new_unchecked(input)),
            Err(error) => Err(TokenParseError { input, error }),
        }
    }

    /// Return the inner type.
    pub fn into_inner(self) -> S {
        self.0
    }
}

impl<S: AsRef<str>> AsRef<str> for EntryKey<S> {
    fn as_ref(&self) -> &str {
        self.0.as_ref()
    }
}

/// Field key, such as `key` in `... key = {value}, ...`.
/// 1. Case-insensitive.
/// 2. Does not contain a char in `"{}(),=\\#%\""`.
#[derive(Debug, Clone, PartialEq)]
pub struct FieldKey<S: AsRef<str>>(UniCase<S>);

impl<S: AsRef<str>> FieldKey<S> {
    #[inline]
    pub(crate) fn new_unchecked(s: S) -> Self {
        Self(UniCase::new(s))
    }

    /// Construct a new field key, checking that the input satisfies the requirements.
    pub fn new(input: S) -> Result<Self, TokenParseError<S>> {
        match check_field_key(input.as_ref()) {
            Ok(()) => Ok(Self::new_unchecked(input)),
            Err(error) => Err(TokenParseError { input, error }),
        }
    }

    /// Return the inner type.
    pub fn into_inner(self) -> S {
        self.0.into_inner()
    }
}

impl<S: AsRef<str>> AsRef<str> for FieldKey<S> {
    fn as_ref(&self) -> &str {
        self.0.as_ref()
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
    pub fn variable(s: S) -> Result<Self, TokenParseError<S>> {
        Ok(Token::Variable(Variable::new(s)?))
    }

    /// Construct a new text string variant.
    pub fn str(input: S) -> Result<Self, TokenParseError<S>> {
        match check_balanced(input.as_ref().as_bytes()) {
            Ok(()) => Ok(Token::Text(Text::Str(input))),
            Err(error) => Err(TokenParseError { input, error }),
        }
    }

    /// Construct a new text bytes variant.
    pub fn bytes(input: B) -> Result<Self, TokenParseError<B>> {
        match check_balanced(input.as_ref()) {
            Ok(()) => Ok(Token::Text(Text::Bytes(input))),
            Err(error) => Err(TokenParseError { input, error }),
        }
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
    type Error = ConversionError;
    fn try_from(token: Token<&'r str, &'r [u8]>) -> Result<Self, Self::Error> {
        match token {
            Token::Variable(Variable(s)) => {
                Err(ConversionError::UnexpandedMacro(s.as_ref().to_owned()))
            }
            Token::Text(text) => Ok(text.into_str()?),
        }
    }
}

impl<'r> TryFrom<Token<&'r str, &'r [u8]>> for &'r [u8] {
    type Error = ConversionError;
    fn try_from(token: Token<&'r str, &'r [u8]>) -> Result<Self, Self::Error> {
        match token {
            Token::Variable(Variable(s)) => {
                Err(ConversionError::UnexpandedMacro(s.as_ref().to_string()))
            }
            Token::Text(text) => Ok(text.into_bytes()),
        }
    }
}

impl<S: AsRef<str>, B: AsRef<[u8]>> TryFrom<Token<S, B>> for Text<S, B> {
    type Error = ConversionError;
    fn try_from(token: Token<S, B>) -> Result<Self, Self::Error> {
        match token {
            Token::Variable(Variable(s)) => {
                Err(ConversionError::UnexpandedMacro(s.as_ref().to_string()))
            }
            Token::Text(text) => Ok(text),
        }
    }
}
