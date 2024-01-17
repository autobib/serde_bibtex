use crate::error::TokenConversionError;
use std::borrow::Cow;
use unicase::{Ascii, UniCase};

use super::{AsciiIdentifier, Text, UnicodeIdentifier};

/// ASCII Printable characters not permitted in any identifiers.
const DISALLOWED_PRINTABLE_CHARS: &str = "{}(),=\\#%'\"";

/// Entry type, such as `article` in `@article{...`.
///
/// Rules:
/// 1. Case-insensitive.
/// 2. Any valid non-ASCII UTF-8, or ASCII chars in the printable range `b'\x21'..=b'\x7e'` which
///    are not one of [`DISALLOWED_PRINTABLE_CHARS`].
#[derive(Debug, Clone, PartialEq)]
pub enum EntryType<'r> {
    Preamble,
    Comment,
    Macro,
    Regular(UniCase<Cow<'r, str>>),
}

impl<'r> From<UnicodeIdentifier<'r>> for EntryType<'r> {
    fn from(id: UnicodeIdentifier<'r>) -> Self {
        let UnicodeIdentifier(s) = id;
        let uni = UniCase::new(s);
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
}

/// Macro variable, such as `var` in `@string{var = ...}`.
///
/// Rules:
/// 1. Case-insensitive.
/// 2. Any valid non-ASCII UTF-8, or ASCII chars in the printable range `b'\x21'..=b'\x7e'` which
///    are not one of [`DISALLOWED_PRINTABLE_CHARS`].
#[derive(Debug, Clone, Eq, Hash, PartialEq)]
pub struct Variable<'r>(pub UniCase<Cow<'r, str>>);

impl<'r> Variable<'r> {
    pub fn from_str_unchecked(s: &'r str) -> Self {
        Self(UniCase::new(Cow::Borrowed(s)))
    }
}

impl<'r> From<UnicodeIdentifier<'r>> for Variable<'r> {
    fn from(id: UnicodeIdentifier<'r>) -> Self {
        let UnicodeIdentifier(s) = id;
        Variable(UniCase::new(s))
    }
}

/// Entry key, such as `key` in `@article{key,....`.
///
/// Rules:
/// 1. Case-sensitive.
/// 2. Any valid non-ASCII UTF-8, or ASCII chars in the printable range `b'\x21'..=b'\x7e'` which
///    are not one of [`DISALLOWED_PRINTABLE_CHARS`].
#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub struct EntryKey<'r>(pub Cow<'r, str>);

impl<'r> From<UnicodeIdentifier<'r>> for EntryKey<'r> {
    fn from(id: UnicodeIdentifier<'r>) -> Self {
        let UnicodeIdentifier(s) = id;
        EntryKey(s)
    }
}

/// Field key, such as `key` in `... key = {value}, ...`.
///
/// Rules:
/// 1. Case-insensitive.
/// 2. Any valid printable ASCII except characters in [`DISALLOWED_CHARS`].
/// 2. ASCII chars in the printable range `b'\x21'..=b'\x7e'` which are not one of
/// [`DISALLOWED_PRINTABLE_CHARS`].
#[derive(Debug, Clone, PartialEq)]
pub struct FieldKey<'r>(pub Ascii<Cow<'r, str>>);

impl<'r> From<AsciiIdentifier<'r>> for FieldKey<'r> {
    fn from(id: AsciiIdentifier<'r>) -> Self {
        let AsciiIdentifier(s) = id;
        FieldKey(Ascii::new(s))
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Token<'r> {
    Macro(Variable<'r>),
    Text(Text<'r>),
}

impl<'r> TryFrom<Token<'r>> for Cow<'r, str> {
    type Error = TokenConversionError;
    fn try_from(token: Token<'r>) -> Result<Self, Self::Error> {
        match token {
            Token::Macro(Variable(s)) => Err(Self::Error::UnresolvedMacro(s.to_string())),
            Token::Text(text) => Ok(text.into_cow_str().map_err(Self::Error::InvalidUtf8)?),
        }
    }
}

impl<'r> TryFrom<Token<'r>> for Cow<'r, [u8]> {
    type Error = TokenConversionError;
    fn try_from(token: Token<'r>) -> Result<Self, Self::Error> {
        match token {
            Token::Macro(Variable(s)) => Err(Self::Error::UnresolvedMacro(s.to_string())),
            Token::Text(text) => Ok(text.into_bytes()),
        }
    }
}

impl<'r> TryFrom<Token<'r>> for Text<'r> {
    type Error = TokenConversionError;
    fn try_from(token: Token<'r>) -> Result<Self, Self::Error> {
        match token {
            Token::Macro(Variable(s)) => Err(Self::Error::UnresolvedMacro(s.to_string())),
            Token::Text(text) => Ok(text),
        }
    }
}

impl<'r> Token<'r> {
    pub fn macro_from(s: &'r str) -> Self {
        Token::Macro(Variable::from_str_unchecked(s))
    }

    pub fn text_from(s: &'r str) -> Self {
        Token::Text(Text::Str(Cow::Borrowed(s)))
    }
}
