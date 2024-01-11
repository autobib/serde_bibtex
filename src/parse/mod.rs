pub mod core;
pub mod token;

use std::borrow::Cow;

use crate::abbrev::Abbreviations;
use crate::error::Error;
use crate::parse::core::ChunkType;
use crate::value::{Identifier, Token};

/// The core stateful reader trait.
pub trait BibtexReader<'r> {
    /// Read the chunk type, returning None of EOF was reached.
    fn take_chunk_type(&mut self) -> Result<Option<ChunkType<'r>>, Error>;

    /// Consume an opening bracket `(` or `{`, and return the corresponding closing bracket.
    fn take_initial(&mut self) -> Result<char, Error>;

    /// Consume a comma separator optionally.
    fn opt_comma(&mut self) -> Result<(), Error>;

    /// Parse a citation key
    fn take_citation_key(&mut self) -> Result<&'r str, Error>;

    /// Parse a citation key
    fn take_field_key(&mut self) -> Result<Option<Identifier<'r>>, Error>;

    /// Parse a comma and field key together.
    fn take_comma_and_field_key(&mut self) -> Result<Option<Identifier<'r>>, Error>;

    /// Parse bracketed text inside `@string` and `@preamble`.
    fn take_bracketed_text(&mut self) -> Result<&'r str, Error>;

    /// Ignore a field separator ` = `.
    fn ignore_field_sep(&mut self) -> Result<(), Error>;

    /// Consume a closing bracket `closing`.
    fn take_terminal(&mut self, closing: char) -> Result<(), Error>;

    /// Take a token without resolving abbreviations.
    fn take_token(&mut self, is_first_token: &mut bool) -> Result<Option<Token<'r>>, Error>;

    // Default implementations

    fn ignore_bibliography(&mut self) -> Result<(), Error> {
        while let Some(chunk) = self.take_chunk_type()? {
            self.ignore_chunk(chunk)?;
        }
        Ok(())
    }

    fn ignore_chunk(&mut self, chunk: ChunkType<'r>) -> Result<(), Error> {
        return match chunk {
            ChunkType::Preamble => self.ignore_bracketed(),
            ChunkType::Comment => self.ignore_bracketed(),
            ChunkType::Abbreviation => self.ignore_abbreviation(),
            ChunkType::Entry(_) => self.ignore_entry(),
        };
    }

    fn ignore_bracketed(&mut self) -> Result<(), Error> {
        let _ = self.take_bracketed_text()?;
        Ok(())
    }

    fn ignore_abbreviation(&mut self) -> Result<(), Error> {
        let closing_bracket = self.take_initial()?;
        if let Some(_) = self.take_field_key()? {
            self.ignore_field_sep()?;
            self.ignore_value()?;
            self.opt_comma()?;
        }
        self.take_terminal(closing_bracket)
    }

    fn ignore_entry(&mut self) -> Result<(), Error> {
        let closing_bracket = self.take_initial()?;
        let _ = self.take_citation_key()?;
        self.ignore_fields()?;
        self.opt_comma()?;
        self.take_terminal(closing_bracket)?;
        Ok(())
    }

    fn ignore_fields(&mut self) -> Result<(), Error> {
        while let Some(_) = self.take_comma_and_field_key()? {
            self.ignore_field_sep()?;
            self.ignore_value()?;
        }
        Ok(())
    }

    fn ignore_value(&mut self) -> Result<(), Error> {
        let mut is_first_token = true;
        while let Some(_) = self.take_token(&mut is_first_token)? {}
        Ok(())
    }

    fn take_token_resolved<'s>(
        &mut self,
        abbrevs: &'s Abbreviations<'r>,
        is_first_token: &mut bool,
        scratch: &mut Vec<Token<'r>>,
    ) -> Result<Option<Token<'r>>, Error> {
        loop {
            // try to get a token from the end of the buffer
            if let Some(elem) = scratch.pop() {
                return Ok(Some(elem));
            }

            // buffer is empty, parse a new token
            let token = match self.take_token(is_first_token)? {
                None => return Ok(None),
                Some(token) => token,
            };

            match token {
                // if it is an identifier, expand it
                Token::Abbrev(ref identifier) => {
                    if let Some(token_slice) = abbrevs.get(identifier) {
                        let mut it = token_slice.iter().cloned();
                        match it.next() {
                            // we found a token: extend with the remainder and return the token.
                            // reverse since we pop() from the buffer
                            Some(token) => {
                                scratch.extend(it.rev());
                                return Ok(Some(token));
                            }
                            None => {}
                        }
                    } else {
                        return Ok(Some(token));
                    }
                }
                // otherwise
                _ => return Ok(Some(token)),
            }
        }
    }

    fn take_value_as_cow<'s>(
        &mut self,
        abbrevs: &'s Abbreviations<'r>,
        is_first_token: &mut bool,
        scratch: &mut Vec<Token<'r>>,
    ) -> Result<Cow<'r, str>, Error> {
        // get the first non-empty Token
        let mut init = loop {
            match self.take_token_resolved(abbrevs, is_first_token, scratch)? {
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
        while let Some(token) = self.take_token_resolved(abbrevs, is_first_token, scratch)? {
            let cow: Cow<'r, str> = token.try_into()?;
            if cow.len() > 0 {
                let mut owned = init.into_owned();
                owned.push_str(&cow);
                init = Cow::Owned(owned);
            }
        }
        Ok(init)
    }

    fn take_value_as_char<'s>(
        &mut self,
        abbrevs: &'s Abbreviations<'r>,
        is_first_token: &mut bool,
        scratch: &mut Vec<Token<'r>>,
    ) -> Result<char, Error> {
        // TODO: this could be optimized with a customied parse_char that
        // short-circuits when it sees more than one char.
        let parsed = self.take_value_as_cow(abbrevs, is_first_token, scratch)?;
        let mut char_iter = parsed.chars();
        match (char_iter.next(), char_iter.next()) {
            (Some(c), None) => Ok(c),
            _ => Err(Error::Message("Expected char".to_string())),
        }
    }
}
