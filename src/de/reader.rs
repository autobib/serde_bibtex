use std::borrow::Cow;

use crate::abbrev::Abbreviations;
use crate::bib::{Identifier, Token};
use crate::error::Error;
use crate::parse::{entry_key, first_token, identifier, subsequent_token, take_flag, Flag, identifier_str};

// @article{key,
//   author = {Auth},
//   year = 20 # undefined,
// }
//
// is equivalent to the JSON
//
// {
//   "entry_type": "article",
//   "entry_key": "key",
//   "fields": {
//     "author": [{"text": "Auth"}],
//     "year": [{"text": "2023"}, {"abbrev": "undefined"}],
//   }
// }
//
// But we also have fancier processing. For instance, if we first have
//
// @string{undefined = {23}}
//
// we can parse this into the struct
//
// #[derive(Deserialize)]
// struct Entry<'a> {
//     #[serde(rename = "entry_type")]
//     type: &'a str,
//     #[serde(rename = "entry_key")]
//     key: &'a str,
//     fields: Fields<'a>
// }
//
// struct Fields<'a> {
//     author: &'a str,
//     year: i64,
// }

// TODO: parsing variants
// resolving parser (for abbreviations)
// enforcing valid (to check parsing grammar)
// comment style (bibtex vs biber)
//
// TODO
// Use DeserializeSeed to build a custom receiver so all of the Value are slices of a fixed
// pre-allocated array.

/// A struct to sequentially read from an entry stored in `self.input`. Rather than record the
/// internal of the parser, we instead determine the current position based on the subsequent
/// characters. This correctly parses valid BibTex, but also correctly parses a wide variety of
/// non-bibtex input.
///
/// The general structure of an entry as as follows:
/// ```bib
/// @entry_type{entry_key,
///   field_key = field_value,
///   ...,
/// }
/// ```
/// Here are the cases depending on the prefix of `self.input` after whitespace stripping.
/// 1. `@`: we are parsing `entry_type`.
/// 2. `{` or `(`: we are parsing `entry_key`.
/// 3. `,` followed by optional whitespace and not one of `})`: we are parsing `field_key`.
/// 4. `,` followed by optional whitespace and one of `})`: we have reached the end of the fields.
/// 4. `=`: we are parsing `field_value`.
/// We enforce that `self.input` is always whitespace-stripped.
///
/// Note that this parsing grammar is more flexible than the classic BibTex parser. For example,
/// we do not enforce matching brackets at the beginning and end of the entry.
pub struct ResolvingReader<'s, 'r> {
    pub input: &'r str,
    abbrevs: &'s Abbreviations<'r>,
    buffered_unit: Option<Cow<'r, str>>,
    buffered_flag: Option<Flag>,
    token_buffer: Vec<Token<'r>>,
    is_first_token: bool,
}

impl<'s, 'r> ResolvingReader<'s, 'r> {
    pub fn new(input: &'r str, abbrevs: &'s Abbreviations<'r>) -> Self {
        Self {
            input,
            abbrevs,
            buffered_unit: None,
            buffered_flag: None,
            token_buffer: Vec::new(),
            is_first_token: true,
        }
    }

    pub fn take_entry_type(&mut self) -> Result<Identifier<'r>, Error> {
        let (input, key) = identifier(self.input)?;
        self.input = input;
        Ok(key)
    }

    pub fn take_entry_key(&mut self) -> Result<&'r str, Error> {
        let (input, key) = entry_key(self.input)?;
        self.input = input;
        Ok(key)
    }

    pub fn take_field_key(&mut self) -> Result<Identifier<'r>, Error> {
        let (input, key) = identifier(self.input)?;
        self.input = input;
        Ok(key)
    }

    /// Consume a Flag and the subsequent value.
    pub fn skip(&mut self) -> Result<(), Error> {
        match self.take_flag()? {
            Flag::EntryType => todo!(),
            Flag::EntryKey => todo!(),
            Flag::FieldKey => todo!(),
            Flag::FieldValue => {
                while let Some(_) = take_token(&mut self.input, &mut self.is_first_token)? {}
                Ok(())
            }
            Flag::EndOfEntry => Ok(()),
        }
    }

    pub fn clear_buffered_unit(&mut self) {
        self.buffered_unit = None;
    }

    pub fn clear_buffered_flag(&mut self) {
        self.buffered_flag = None;
    }

    // pub fn clear_buffer(&mut self) {
    //     self.buffered_unit = None;
    //     self.buffered_flag = None;
    //     self.token_buffer.clear();
    //     self.is_first_token = true;
    // }

    pub fn take_null(&mut self) -> Result<(), Error> {
        match self.buffered_unit.take() {
            None => parse_null(
                &mut self.input,
                &mut self.is_first_token,
                &mut self.token_buffer,
                &self.abbrevs,
            ),
            Some(cow) => {
                if cow.len() > 0 {
                    Ok(())
                } else {
                    Err(Error::Message("Expected null".to_string()))
                }
            }
        }
    }

    /// Take a `FieldValue` as a `char`.
    pub fn take_char(&mut self) -> Result<char, Error> {
        // TODO: this could be optimized with a customied parse_char that
        // short-circuits when it sees more than one char.
        let parsed = self.take_unit()?;
        let mut char_iter = parsed.chars();
        match (char_iter.next(), char_iter.next()) {
            (Some(c), None) => Ok(c),
            _ => Err(Error::Message("Expected char".to_string())),
        }
    }

    /// Take a `FieldValue` as `Cow<'r, str>`.
    pub fn take_unit(&mut self) -> Result<Cow<'r, str>, Error> {
        match self.buffered_unit.take() {
            Some(cow) => Ok(cow),
            None => parse_unit(
                &mut self.input,
                &mut self.is_first_token,
                &mut self.token_buffer,
                &self.abbrevs,
            ),
        }
    }

    fn insert_buffered<'a>(
        buffered_unit: &'a mut Option<Cow<'r, str>>,
        parsed: Cow<'r, str>,
    ) -> &'a mut Cow<'r, str> {
        buffered_unit.insert(parsed)
    }
    /// Peek a `FieldValue` as a `&Cow<'r, str>`.
    pub fn peek_unit(&mut self) -> Result<&Cow<'r, str>, Error> {
        match self.buffered_unit {
            Some(ref cow) => Ok(cow),
            None => {
                let parsed = parse_unit(
                    &mut self.input,
                    &mut self.is_first_token,
                    &mut self.token_buffer,
                    &self.abbrevs,
                )?;
                Ok(Self::insert_buffered(&mut self.buffered_unit, parsed))
            }
        }
    }

    /// Peek a `Flag`, returning the value, but do not consume it.
    pub fn peek_flag(&mut self) -> Result<Flag, Error> {
        match self.buffered_flag {
            Some(flag) => Ok(flag),
            None => {
                let (input, received) = take_flag(self.input)?;
                self.input = input;
                self.buffered_flag = Some(received);
                Ok(received)
            }
        }
    }

    /// Take any `Flag` and return it.
    pub fn take_flag(&mut self) -> Result<Flag, Error> {
        match self.buffered_flag.take() {
            Some(flag) => Ok(flag),
            None => {
                let (input, received) = take_flag(self.input)?;
                self.input = input;
                Ok(received)
            }
        }
    }
}

fn parse_null<'r>(
    input: &mut &'r str,
    is_first_token: &mut bool,
    token_buffer: &mut Vec<Token<'r>>,
    abbrevs: &Abbreviations<'r>,
) -> Result<(), Error> {
    while let Some(token) = take_token_resolved(input, is_first_token, token_buffer, abbrevs)? {
        match token {
            Token::Text(cow) if cow.len() == 0 => {}
            _ => return Err(Error::Message("Expected null, get something".to_string())),
        }
    }
    Ok(())
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
    let token = match take_token_resolved(input, is_first_token, token_buffer, abbrevs)? {
        Some(cow) => cow,
        None => return Ok(Cow::Borrowed("")),
    };
    let mut init: Cow<'r, str> = token.try_into()?;

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
        let (updated, token) = first_token(input)?;
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
