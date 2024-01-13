pub mod core;
pub mod token;

use crate::error::Error;
use crate::parse::core::EntryType;
use crate::value::{Identifier, Token, Value};

use crate::macros::MacroDictionary;

/// The core stateful reader trait.
pub trait BibtexReader<'r> {
    /// Read the chunk type, returning None if EOF was reached.
    fn take_entry_type(&mut self) -> Result<Option<EntryType<'r>>, Error>;

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

    /// Read tokens until there are no more remaining in the buffer.
    fn take_value_into<'s>(&mut self, scratch: &mut Vec<Token<'r>>) -> Result<(), Error> {
        scratch.clear();
        let mut is_first_token = true;

        while let Some(token) = self.take_token(&mut is_first_token)? {
            scratch.push(token)
        }
        Ok(())
    }

    fn ignore_bibliography(&mut self) -> Result<(), Error> {
        while let Some(chunk) = self.take_entry_type()? {
            self.ignore_chunk(chunk)?;
        }
        Ok(())
    }

    fn ignore_chunk(&mut self, chunk: EntryType<'r>) -> Result<(), Error> {
        return match chunk {
            EntryType::Preamble => self.ignore_bracketed(),
            EntryType::Comment => self.ignore_bracketed(),
            EntryType::Macro => self.ignore_abbreviation(),
            EntryType::Regular(_) => self.ignore_entry(),
        };
    }

    fn ignore_entry_captured(
        &mut self,
        chunk: EntryType<'r>,
        abbrevs: &mut MacroDictionary<'r>,
    ) -> Result<(), Error> {
        return match chunk {
            EntryType::Preamble => self.ignore_bracketed(),
            EntryType::Comment => self.ignore_bracketed(),
            EntryType::Macro => self.ignore_abbreviation_captured(abbrevs),
            EntryType::Regular(_) => self.ignore_entry(),
        };
    }

    fn ignore_bracketed(&mut self) -> Result<(), Error> {
        let _ = self.take_bracketed_text()?;
        Ok(())
    }

    fn ignore_abbreviation_captured(
        &mut self,
        abbrevs: &mut MacroDictionary<'r>,
    ) -> Result<(), Error> {
        let closing_bracket = self.take_initial()?;
        if let Some(identifier) = self.take_field_key()? {
            let mut tokens = Vec::new();
            self.ignore_field_sep()?;
            self.take_value_into(&mut tokens)?;
            abbrevs.insert(identifier, Value(tokens));
            self.opt_comma()?;
        }
        self.take_terminal(closing_bracket)
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
}
