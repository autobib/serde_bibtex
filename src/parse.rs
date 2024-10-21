mod macros;
mod read;

use crate::error::{Error, ErrorCode, Result};

use crate::token::{EntryKey, EntryType, FieldKey, Text, Token, Variable};
pub use macros::MacroDictionary;
pub use read::{Read, SliceReader, StrReader};

pub trait BibtexParse<'r>: Read<'r> {
    /// Read the entry type, returning None if EOF was reached.
    fn entry_type(&mut self) -> Result<Option<EntryType<&'r str>>> {
        if self.next_entry_or_eof() {
            self.comment();
            let id = self.identifier()?;
            Ok(Some(id.into()))
        } else {
            Ok(None)
        }
    }

    #[inline]
    fn expect(&mut self, expected: u8, err: Error) -> Result<()> {
        if self.peek() == Some(expected) {
            self.discard();
            Ok(())
        } else {
            Err(err)
        }
    }

    /// Consume an opening bracket `(` or `{`, and return the corresponding closing bracket.
    fn initial(&mut self) -> Result<u8> {
        self.comment();
        match self.peek() {
            Some(b'{') => {
                self.discard();
                Ok(b'}')
            }
            Some(b'(') => {
                self.discard();
                Ok(b')')
            }
            _ => Err(Error::syntax(ErrorCode::InvalidStartOfEntry)),
        }
    }

    /// Read an entry key.
    fn entry_key(&mut self) -> Result<EntryKey<&'r str>> {
        self.comment();
        Ok(self.identifier()?.into())
    }

    /// Consume a comma separator optionally.
    fn comma_opt(&mut self) {
        self.comment();
        if self.peek() == Some(b',') {
            self.discard();
        }
    }

    /// Consume a variable
    #[inline]
    fn variable(&mut self) -> Result<Variable<&'r str>> {
        self.comment();
        let id = self.identifier()?;
        Ok(id.into())
    }

    /// Return macro definition, if any.
    fn macro_variable_opt(&mut self) -> Result<Option<Variable<&'r str>>> {
        self.comment();
        match self.peek() {
            Some(b'}' | b')') => Ok(None),
            Some(b'0'..=b'9') => Err(Error::syntax(ErrorCode::VariableStartsWithDigit)),
            _ => {
                let id = self.identifier()?;
                Ok(Some(id.into()))
            }
        }
    }

    /// Ignore a field separator  `=`.
    fn field_sep(&mut self) -> Result<()> {
        self.comment();
        self.expect(b'=', Error::syntax(ErrorCode::ExpectedFieldSep))?;
        Ok(())
    }

    /// Ignore a token separator `#`, returning true if it was captured and false otherwise.
    fn next_token_or_end(&mut self) -> Result<bool> {
        self.comment();
        match self.peek() {
            Some(b'#') => {
                self.discard();
                Ok(true)
            }
            Some(b'}' | b')' | b',') | None => Ok(false),
            Some(_) => Err(Error::syntax(ErrorCode::ExpectedNextTokenOrEndOfField)),
        }
    }

    /// Take a token without resolving abbreviations.
    fn token(&mut self, is_first_token: &mut bool) -> Result<Option<Token<&'r str, &'r [u8]>>> {
        // first token is mandatory
        if *is_first_token {
            *is_first_token = false;
        // otherwise, check if there is another one: if not, we are done
        } else if !self.next_token_or_end()? {
            return Ok(None);
        }

        self.comment();
        match self.peek() {
            Some(b'{') => {
                self.discard();
                let result = self.balanced()?;
                self.expect(b'}', Error::syntax(ErrorCode::UnclosedBracket))?;
                Ok(Some(Token::Text(result)))
            }
            Some(b'"') => {
                self.discard();
                let result = self.protected(b'"')?;
                self.expect(b'"', Error::syntax(ErrorCode::UnclosedQuote))?;
                Ok(Some(Token::Text(result)))
            }
            Some(b'0'..=b'9') => Ok(Some(Token::Text(Text::Str(self.number()?)))),
            Some(_) => Ok(Some(Token::Variable(self.identifier()?.into()))),
            _ => Err(Error::eof()),
        }
    }

    /// Parse a comma and field key together to determine if there is another field.
    fn field_or_terminal(&mut self) -> Result<Option<FieldKey<&'r str>>> {
        self.comment();
        match self.peek() {
            Some(b',') => {
                self.discard();
                self.comment();
                match self.peek() {
                    Some(b'}' | b')') => Ok(None),
                    _ => Ok(Some(self.identifier()?.into())),
                }
            }
            _ => Ok(None),
        }
    }

    /// Parse bracketed text inside `@string` and `@preamble`.
    fn comment_contents(&mut self) -> Result<Text<&'r str, &'r [u8]>> {
        self.comment();
        let closing = self.initial()?;
        let result = match closing {
            b')' => self.protected(closing)?,
            b'}' => self.balanced()?,
            _ => unreachable!(),
        };
        self.terminal(closing)?;
        Ok(result)
    }

    /// Consume a closing bracket `closing`.
    fn terminal(&mut self, closing: u8) -> Result<()> {
        self.comment();
        self.expect(closing, Error::syntax(ErrorCode::ExpectedEndOfEntry))?;
        Ok(())
    }

    /// Read tokens until there are no more remaining in the buffer.
    fn value_into(&mut self, scratch: &mut Vec<Token<&'r str, &'r [u8]>>) -> Result<()> {
        scratch.clear();
        let mut is_first_token = true;

        while let Some(token) = self.token(&mut is_first_token)? {
            scratch.push(token)
        }
        Ok(())
    }

    /// Ignore an entire bibliography, while still checking validity.
    fn ignore_bibliography(&mut self) -> Result<()> {
        while let Some(chunk) = self.entry_type()? {
            self.ignore_entry(chunk)?;
        }
        Ok(())
    }

    /// Ignore a single entry.
    fn ignore_entry(&mut self, chunk: EntryType<&'r str>) -> Result<()> {
        match chunk {
            EntryType::Preamble => self.ignore_preamble(),
            EntryType::Comment => self.ignore_comment(),
            EntryType::Macro => self.ignore_macro(),
            EntryType::Regular(_) => self.ignore_regular_entry(),
        }
    }

    /// Ignore a single entry, but capture any macros.
    fn ignore_entry_captured(
        &mut self,
        chunk: EntryType<&'r str>,
        abbrevs: &mut MacroDictionary<&'r str, &'r [u8]>,
    ) -> Result<()> {
        match chunk {
            EntryType::Preamble => self.ignore_preamble(),
            EntryType::Comment => self.ignore_comment(),
            EntryType::Macro => self.ignore_macro_captured(abbrevs),
            EntryType::Regular(_) => self.ignore_regular_entry(),
        }
    }

    /// Ignore the contents of a comment.
    fn ignore_comment(&mut self) -> Result<()> {
        let _ = self.comment_contents()?;
        Ok(())
    }

    /// Ignore the contents of a preamble.
    fn ignore_preamble(&mut self) -> Result<()> {
        let closing_bracket = self.initial()?;
        self.ignore_value()?;
        self.terminal(closing_bracket)
    }

    /// Ignore the contents of a macro definition.
    fn ignore_macro(&mut self) -> Result<()> {
        let closing_bracket = self.initial()?;
        if (self.macro_variable_opt()?).is_some() {
            self.field_sep()?;
            self.ignore_value()?;
            self.comma_opt();
        }
        self.terminal(closing_bracket)
    }

    /// Ignore the contents of a macro definition, but capture into `abbrevs`.
    fn ignore_macro_captured(
        &mut self,
        abbrevs: &mut MacroDictionary<&'r str, &'r [u8]>,
    ) -> Result<()> {
        let closing_bracket = self.initial()?;
        if let Some(identifier) = self.macro_variable_opt()? {
            let mut tokens = Vec::new();
            self.field_sep()?;
            self.value_into(&mut tokens)?;
            abbrevs.insert(identifier, tokens);
            self.comma_opt();
        }
        self.terminal(closing_bracket)
    }

    /// Ignore the contents of a regular entry.
    fn ignore_regular_entry(&mut self) -> Result<()> {
        let closing_bracket = self.initial()?;
        let _ = self.entry_key()?;
        self.ignore_fields()?;
        self.comma_opt();
        self.terminal(closing_bracket)?;
        Ok(())
    }

    /// Ignore the fields in a regular entry.
    fn ignore_fields(&mut self) -> Result<()> {
        while self.field_or_terminal()?.is_some() {
            self.field_sep()?;
            self.ignore_value()?;
        }
        Ok(())
    }

    /// Ignore a single value for a field.
    fn ignore_value(&mut self) -> Result<()> {
        let mut is_first_token = true;
        while (self.token(&mut is_first_token)?).is_some() {}
        Ok(())
    }
}
