mod macros;
mod read;

use crate::error::{Error, ErrorCode, Result};

use crate::token::{EntryKey, EntryType, FieldKey, IDENTIFIER_ALLOWED, Text, Token, Variable};
pub use macros::MacroDictionary;
pub use read::{BibtexRead, SliceReader, StrReader};

impl<'r, R: BibtexRead<'r>> BibtexParse<'r> for R {}

pub trait BibtexParse<'r>: BibtexRead<'r> + Sized {
    fn locate(&self, error: Error) -> Error {
        error.with_span(self.error_span())
    }

    fn opening_offset(&self) -> Option<usize> {
        self.byte_offset().and_then(|pos| pos.checked_sub(1))
    }

    /// Locate an identifier immediately after parsing it.
    fn identifier_span(&self, id: &str) -> Option<core::ops::Range<usize>> {
        let end = self.byte_offset()?;
        Some(end.checked_sub(id.len())?..end)
    }

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
    fn expect(&mut self, expected: u8, err: impl FnOnce(Option<u8>) -> Error) -> Result<()> {
        let found = self.peek();
        if found == Some(expected) {
            self.discard();
            Ok(())
        } else {
            Err(self.locate(err(found)))
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
            found => Err(self.locate(Error::expected("start of entry '{' or '('", found))),
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
            Some(b'0'..=b'9') => {
                Err(self.locate(Error::syntax(ErrorCode::VariableStartsWithDigit)))
            }
            _ => {
                let id = self.identifier()?;
                Ok(Some(id.into()))
            }
        }
    }

    /// Ignore a field separator  `=`.
    fn field_sep(&mut self) -> Result<()> {
        self.comment();
        self.expect(b'=', |found| Error::expected("field separator '='", found))?;
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
            Some(_) => Err(self.locate(Error::syntax(ErrorCode::Expected(
                "token separator '#' or end of value",
            )))),
        }
    }

    /// Take a single token.
    fn single_token(&mut self) -> Result<Token<&'r str, &'r [u8]>> {
        self.comment();
        match self.peek() {
            Some(b'{') => {
                self.discard();
                let result = self.balanced()?;
                self.expect(b'}', |_| Error::syntax(ErrorCode::UnclosedDelimiter(b'{')))?;
                Ok(Token::Text(result))
            }
            Some(b'"') => {
                self.discard();
                let result = self.protected(b'"')?;
                self.expect(b'"', |_| Error::syntax(ErrorCode::UnclosedDelimiter(b'"')))?;
                Ok(Token::Text(result))
            }
            Some(b'0'..=b'9') => Ok(Token::Text(Text::Str(self.number()?))),
            Some(b) if IDENTIFIER_ALLOWED[b as usize] => {
                Ok(Token::Variable(self.identifier()?.into()))
            }
            found => Err(self.locate(Error::expected("value", found))),
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

        self.single_token().map(Some)
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

    /// Parse protected text inside `@comment`.
    fn comment_contents(&mut self) -> Result<Text<&'r str, &'r [u8]>> {
        self.comment();
        let closing = self.initial()?;
        let opening = self.opening_offset();
        let result = match closing {
            b')' => self.protected(closing)?,
            b'}' => self.balanced()?,
            _ => unreachable!(),
        };
        self.terminal(closing, opening)?;
        Ok(result)
    }

    /// Consume a closing bracket `closing`.
    fn terminal(&mut self, closing: u8, opening: Option<usize>) -> Result<()> {
        self.comment();
        self.expect(closing, |found| match found {
            Some(found) => Error::syntax(ErrorCode::ExpectedEndOfEntry {
                expected: closing,
                found,
            }),
            None => Error::expected("end of entry", None).in_entry(closing, opening),
        })?;
        Ok(())
    }

    /// Read tokens until there are no more remaining in the buffer.
    fn value_into(&mut self, scratch: &mut Vec<Token<&'r str, &'r [u8]>>) -> Result<()> {
        self.value_into_spanned(scratch).map(|_| ())
    }

    fn value_into_spanned(
        &mut self,
        scratch: &mut Vec<Token<&'r str, &'r [u8]>>,
    ) -> Result<Option<core::ops::Range<usize>>> {
        scratch.clear();
        self.comment();
        let start = self.byte_offset();
        loop {
            scratch.push(self.single_token()?);
            let end = self.byte_offset();
            if !self.next_token_or_end()? {
                return Ok(start.zip(end).map(|(start, end)| start..end));
            }
        }
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

    /// Parse an entry body.
    fn entry<T>(&mut self, body: impl FnOnce(&mut Self) -> Result<T>) -> Result<T> {
        let closing = self.initial()?;
        let opening = self.opening_offset();
        let value = body(self).map_err(|err| err.in_entry(closing, opening))?;
        self.terminal(closing, opening)?;
        Ok(value)
    }

    /// Ignore the contents of a preamble.
    fn ignore_preamble(&mut self) -> Result<()> {
        self.entry(Self::ignore_value)
    }

    /// Ignore the contents of a macro definition.
    fn ignore_macro(&mut self) -> Result<()> {
        self.entry(|parser| {
            if parser.macro_variable_opt()?.is_some() {
                parser.field_sep()?;
                parser.ignore_value()?;
                parser.comma_opt();
            }
            Ok(())
        })
    }

    /// Ignore the contents of a macro definition, but capture into `abbrevs`.
    fn ignore_macro_captured(
        &mut self,
        abbrevs: &mut MacroDictionary<&'r str, &'r [u8]>,
    ) -> Result<()> {
        self.entry(|parser| {
            if let Some(identifier) = parser.macro_variable_opt()? {
                let mut tokens = Vec::new();
                parser.field_sep()?;
                parser.value_into(&mut tokens)?;
                abbrevs.insert(identifier, tokens);
                parser.comma_opt();
            }
            Ok(())
        })
    }

    /// Ignore the contents of a regular entry.
    fn ignore_regular_entry(&mut self) -> Result<()> {
        self.entry(|parser| {
            let _ = parser.entry_key()?;
            parser.ignore_fields()?;
            parser.comma_opt();
            Ok(())
        })
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::Category;

    struct CustomSpanReader;

    impl<'r> BibtexRead<'r> for CustomSpanReader {
        fn error_span(&self) -> Option<core::ops::Range<usize>> {
            Some(10..20)
        }
        fn peek(&self) -> Option<u8> {
            Some(b'!')
        }
        fn comment(&mut self) {}
        fn discard(&mut self) {
            unreachable!()
        }
        fn next_entry_or_eof(&mut self) -> bool {
            unreachable!()
        }
        fn identifier(&mut self) -> Result<crate::token::Identifier<&'r str>> {
            unreachable!()
        }
        fn balanced(&mut self) -> Result<Text<&'r str, &'r [u8]>> {
            unreachable!()
        }
        fn protected(&mut self, _: u8) -> Result<Text<&'r str, &'r [u8]>> {
            unreachable!()
        }
        fn number(&mut self) -> Result<&'r str> {
            unreachable!()
        }
    }

    #[test]
    fn parser_uses_custom_reader_spans() {
        let mut reader = CustomSpanReader;
        assert_eq!(reader.byte_offset(), None);
        assert_eq!(reader.field_sep().unwrap_err().span(), Some(10..20));
        let error = Error::expected("value", None).with_span(Some(3..3));
        assert_eq!(reader.locate(error).span(), Some(3..3));
    }

    #[test]
    fn expect_constructs_errors_only_on_failure() {
        let mut reader = StrReader::new("=");
        reader
            .expect(b'=', |_| panic!("constructed an error on success"))
            .unwrap();
        let error = reader
            .expect(b'=', |found| Error::expected("field separator '='", found))
            .unwrap_err();
        assert_eq!(
            error.to_string(),
            "unexpected end of input; expected field separator '='"
        );
        assert_eq!(error.classify(), Category::Eof);
    }

    #[test]
    fn standalone_grammar_eof_has_no_entry_context() {
        for error in [
            StrReader::new("").variable().unwrap_err(),
            StrReader::new("").macro_variable_opt().unwrap_err(),
            StrReader::new(",").field_or_terminal().unwrap_err(),
        ] {
            assert_eq!(
                error.to_string(),
                "unexpected end of input; expected identifier"
            );
            assert_eq!(error.classify(), Category::Eof);
        }
        let error = StrReader::new("{x}#").ignore_value().unwrap_err();
        assert_eq!(error.to_string(), "unexpected end of input; expected value");
        assert_eq!(error.classify(), Category::Eof);
    }
}
