use std::borrow::Cow;

use crate::abbrev::Abbreviations;
use crate::bib::{Identifier, Token};
use crate::error::Error;
use crate::parse::core::{
    citation_key, entry_type, field_key, field_sep, subsequent_token, terminal, token,
};

// TODO: parsing variants
// resolving parser (for abbreviations)
// enforcing valid (to check parsing grammar)
// comment style (bibtex vs biber)

/// An enum to track the current parsing position inside an entry.
#[derive(Debug, Copy, Clone)]
pub(crate) enum Position {
    EntryType,
    CitationKey,
    Fields,
    EndOfEntry,
}

impl Position {
    pub fn step(&mut self) {
        *self = match self {
            Self::EntryType => Self::CitationKey,
            Self::CitationKey => Self::Fields,
            Self::Fields => Self::EndOfEntry,
            Self::EndOfEntry => Self::EntryType,
        };
    }
}

#[derive(Debug)]
pub struct ResolvingReader<'s, 'r> {
    input: &'r str,
    abbrevs: &'s Abbreviations<'r>,
    token_buffer: Vec<Token<'r>>,
    is_first_token: bool,
    parsing_state: Position,
    matching: char,
}

impl<'s, 'r> ResolvingReader<'s, 'r> {
    pub fn new(input: &'r str, abbrevs: &'s Abbreviations<'r>) -> Self {
        Self {
            input,
            abbrevs,
            token_buffer: Vec::new(),
            is_first_token: true,
            parsing_state: Position::EndOfEntry,
            matching: '}', // default for self.take_citation_key()
        }
    }

    pub fn update_position(&mut self) -> &Position {
        self.parsing_state.step();
        &self.parsing_state
    }

    pub fn get_position(&self) -> &Position {
        &self.parsing_state
    }

    pub fn take_entry_type(&mut self) -> Result<Identifier<'r>, Error> {
        let (input, key) = entry_type(self.input)?;
        self.input = input;
        Ok(key)
    }

    pub fn take_citation_key(&mut self) -> Result<&'r str, Error> {
        let (input, (key, open)) = citation_key(self.input)?;
        self.input = input;
        // defaults to '}' in Self::new(...)
        if open == '(' {
            self.matching = ')';
        }
        Ok(key)
    }

    pub fn take_field_key(&mut self) -> Result<Option<Identifier<'r>>, Error> {
        let (input, key) = field_key(self.input)?;
        self.input = input;
        Ok(key)
    }

    pub fn take_terminal(&mut self) -> Result<(), Error> {
        let (input, ()) = terminal(self.input, self.matching)?;
        self.input = input;
        Ok(())
    }

    pub fn take_value_as_char(&mut self) -> Result<char, Error> {
        // TODO: this could be optimized with a customied parse_char that
        // short-circuits when it sees more than one char.
        let parsed = self.take_value_as_cow()?;
        let mut char_iter = parsed.chars();
        match (char_iter.next(), char_iter.next()) {
            (Some(c), None) => Ok(c),
            _ => Err(Error::Message("Expected char".to_string())),
        }
    }

    /// Take a `FieldValue` as `Cow<'r, str>`.
    pub fn take_value_as_cow(&mut self) -> Result<Cow<'r, str>, Error> {
        self.take_flag_value()?;
        parse_unit(
            &mut self.input,
            &mut self.is_first_token,
            &mut self.token_buffer,
            &self.abbrevs,
        )
    }

    pub fn ignore_entry(&mut self) -> Result<(), Error> {
        let _ = self.take_entry_type()?;
        let _ = self.take_citation_key()?;
        while let Some(_) = self.take_field_key()? {
            self.ignore_value()?
        }
        self.take_terminal()?;
        Ok(())
    }

    pub fn ignore_value(&mut self) -> Result<(), Error> {
        self.take_flag_value()?;
        while let Some(_) = take_token(&mut self.input, &mut self.is_first_token)? {}
        Ok(())
    }

    /// Take any `Flag` and return it.
    pub fn take_flag_value(&mut self) -> Result<(), Error> {
        let (input, received) = field_sep(self.input)?;
        self.input = input;
        Ok(received)
    }

    pub fn take_token(&mut self) -> Result<Option<Token<'r>>, Error> {
        take_token_resolved(
            &mut self.input,
            &mut self.is_first_token,
            &mut self.token_buffer,
            &self.abbrevs,
        )
    }
}

/// Attempt to combine all of the tokens in a FieldValue into a single string. If there is only a
/// single non-empty `Token`, this borrowes from `input`. Otherwise, we own the string and push to
/// it.
fn parse_unit<'r>(
    input: &mut &'r str,
    is_first_token: &mut bool,
    token_buffer: &mut Vec<Token<'r>>,
    abbrevs: &Abbreviations<'r>,
) -> Result<Cow<'r, str>, Error> {
    // get the first non-empty Token
    let mut init = loop {
        match take_token_resolved(input, is_first_token, token_buffer, abbrevs)? {
            Some(token) => {
                let cow: Cow<'r, str> = token.try_into()?;
                if cow.len() > 0 {
                    break cow;
                }
            }
            None => return Ok(Cow::Borrowed("")),
        }
    };

    // append subsequent Tokens to it
    while let Some(token) = take_token_resolved(input, is_first_token, token_buffer, abbrevs)? {
        let cow: Cow<'r, str> = token.try_into()?;
        if cow.len() > 0 {
            let mut owned = init.into_owned();
            owned.push_str(&cow);
            init = Cow::Owned(owned);
        }
    }
    Ok(init)
}

/// Take a single token from the input and resolve it using `abbrevs`. Since `abbrevs` returns a
/// slice `&[Token<'r>]` which might contain more than one token, we buffer the excess tokens into
/// `token_buffer`, and try to read from `token_buffer` if possible.
///
/// Note that `&[Token<'r>]` might be empty, in which case the identifier is essentially ignored.
fn take_token_resolved<'r>(
    input: &mut &'r str,
    is_first_token: &mut bool,
    token_buffer: &mut Vec<Token<'r>>,
    abbrevs: &Abbreviations<'r>,
) -> Result<Option<Token<'r>>, Error> {
    loop {
        // try to get a token from the end of the buffer
        if let Some(first) = token_buffer.pop() {
            return Ok(Some(first));
        }

        // buffer is empty, parse a new token
        let token = match take_token(input, is_first_token)? {
            None => return Ok(None),
            Some(token) => token,
        };

        match token {
            // if it is an identifier, expand it into the token_buffer and loop
            Token::Abbrev(ref identifier) => {
                if let Some(token_slice) = abbrevs.get(identifier) {
                    // reverse since we pop() from the buffer
                    token_buffer.extend(token_slice.iter().rev().cloned());
                } else {
                    return Ok(Some(token));
                }
            }
            // otherwise
            _ => return Ok(Some(token)),
        }
    }
}

/// Take a single token from the input. This also consumes the separator if `is_first_token` is
/// True.
///
/// SAFETY: if you manually parse the final token, ensure that `is_first_token` is reset to True.
fn take_token<'r>(
    input: &mut &'r str,
    is_first_token: &mut bool,
) -> Result<Option<Token<'r>>, Error> {
    if *is_first_token {
        let (updated, token) = token(input)?;
        *is_first_token = false;
        *input = updated;
        Ok(Some(token))
    } else {
        let (updated, opt_token) = subsequent_token(input)?;
        *input = updated;
        match opt_token {
            Some(_) => Ok(opt_token),
            None => {
                *is_first_token = true;
                Ok(None)
            }
        }
    }
}
