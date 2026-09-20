//! Private reader primitives and shared BibTeX grammar.
use super::{EntryDelimiter, TextDelimiter};
use crate::{
    error::{Error, ErrorCode, Result},
    parse::MacroDictionary,
    token::{EntryKey, EntryType, FieldKey, IDENTIFIER_ALLOWED, Identifier, Text, Token, Variable},
};
use core::ops::Range;

pub(crate) trait BibtexReadInner<'r> {
    /// The current zero-based byte offset in the source.
    fn byte_offset(&self) -> Option<usize> {
        None
    }

    /// The span associated with an error at the current position.
    ///
    /// The default is an empty range at [`byte_offset`](Self::byte_offset), if any.
    ///
    /// [`SliceReader`] returns a span containing exactly one byte, and [`StrReader`]
    /// returns a span containing the current character, or an empty range at EOF.
    fn error_span(&self) -> Option<Range<usize>> {
        self.byte_offset().map(|pos| pos..pos)
    }

    /// Peek the next byte without moving the cursor.
    ///
    /// Returns `None` only at EOF. For string input this returns the first byte of
    /// the current character, rather than decoding it.
    fn peek(&self) -> Option<u8>;

    /// Consume exactly one byte if it matches `expected`.
    ///
    /// Returns `false` without advancing at EOF or on a mismatch. Does not skip
    /// comments or whitespace.
    ///
    /// # Safety
    ///
    /// `expected` must be ASCII.
    unsafe fn consume_ascii(&mut self, expected: u8) -> bool;

    /// Skip ASCII whitespace and TeX comments.
    fn comment(&mut self);

    /// Skip text until the next `@`, consuming the `@` and returning `true`, or
    /// reaching EOF and returning `false`.
    fn next_entry_or_eof(&mut self) -> bool;

    /// Read a nonempty identifier at the current position.
    fn identifier(&mut self) -> Result<Identifier<&'r str>>;

    /// Read a nonempty sequence of ASCII digits at the current position.
    fn number(&mut self) -> Result<&'r str>;

    /// Read text until the delimiter occurs outside balanced curly braces.
    fn text_until(&mut self, delimiter: TextDelimiter) -> Result<Text<&'r str, &'r [u8]>>;

    #[inline]
    fn expect(&mut self, expected: u8, err: impl FnOnce(Option<u8>) -> Error) -> Result<()> {
        assert!(expected.is_ascii());
        let found = self.peek();
        // SAFETY: the assertion above checks the ASCII precondition.
        if unsafe { self.consume_ascii(expected) } {
            Ok(())
        } else {
            Err(self.with_error_span(err(found)))
        }
    }

    /// Inspect the current byte without skipping comments or consuming the closer.
    #[inline]
    fn at_entry_end(&self) -> bool {
        matches!(self.peek(), Some(b'}' | b')'))
    }

    /// Read braced contents, leaving the closing brace unconsumed.
    #[inline]
    fn balanced(&mut self) -> Result<Text<&'r str, &'r [u8]>> {
        self.text_until(TextDelimiter::Brace)
    }

    /// Read text until the delimiter occurs outside balanced curly braces.
    #[inline]
    fn protected(&mut self, until: TextDelimiter) -> Result<Text<&'r str, &'r [u8]>> {
        self.text_until(until)
    }

    /// Skip comments and whitespace, then consume an entry opener.
    #[inline]
    fn start_entry(&mut self) -> Result<EntryDelimiter> {
        self.comment();
        match self.peek() {
            Some(b'{') => {
                // SAFETY: the expected byte is ASCII.
                unsafe {
                    self.consume_ascii(b'{');
                }
                Ok(EntryDelimiter::Brace)
            }
            Some(b'(') => {
                // SAFETY: the expected byte is ASCII.
                unsafe {
                    self.consume_ascii(b'(');
                }
                Ok(EntryDelimiter::Parenthesis)
            }
            found => Err(self.with_error_span(Error::expected("start of entry '{' or '('", found))),
        }
    }

    /// Skip comments and whitespace, then consume a comma if present.
    #[inline]
    fn comma_opt(&mut self) {
        self.comment();
        // SAFETY: the expected byte is ASCII.
        unsafe {
            self.consume_ascii(b',');
        }
    }

    /// Read a macro variable, rejecting an initial ASCII digit.
    ///
    /// Skips comments and whitespace and leaves either entry closer unconsumed.
    #[inline]
    fn macro_variable_opt(&mut self) -> Result<Option<Variable<&'r str>>> {
        self.comment();
        match self.peek() {
            Some(b'}' | b')') => Ok(None),
            Some(b'0'..=b'9') => {
                Err(self.with_error_span(Error::new(ErrorCode::VariableStartsWithDigit)))
            }
            _ => self.identifier().map(|id| Some(id.into())),
        }
    }

    /// Skip comments and whitespace, then require and consume '='.
    #[inline]
    fn field_sep(&mut self) -> Result<()> {
        self.comment();
        self.expect(b'=', |found| Error::expected("field separator '='", found))
    }

    /// Skip comments and whitespace, then consume '#' if present.
    ///
    /// The following token is not validated. A comma, either entry closer, or EOF
    /// returns false and remains unconsumed.
    #[inline]
    fn next_token_or_end(&mut self) -> Result<bool> {
        self.comment();
        match self.peek() {
            Some(b'#') => {
                // SAFETY: the expected byte is ASCII.
                unsafe {
                    self.consume_ascii(b'#');
                }
                Ok(true)
            }
            Some(b'}' | b')' | b',') | None => Ok(false),
            Some(_) => Err(self.with_error_span(Error::new(ErrorCode::Expected(
                "token separator '#' or end of value",
            )))),
        }
    }

    /// Skip comments and whitespace, then read one unresolved text or variable token.
    ///
    /// Quoted and braced text tokens consume their own closing delimiter.
    fn single_token(&mut self) -> Result<Token<&'r str, &'r [u8]>> {
        self.comment();
        match self.peek() {
            Some(b'{') => {
                // SAFETY: the expected byte is ASCII.
                unsafe {
                    self.consume_ascii(b'{');
                }
                let result = self.balanced()?;
                self.expect(b'}', |_| Error::new(ErrorCode::UnclosedDelimiter(b'{')))?;
                Ok(Token::Text(result))
            }
            Some(b'"') => {
                // SAFETY: the expected byte is ASCII.
                unsafe {
                    self.consume_ascii(b'"');
                }
                let result = self.protected(TextDelimiter::Quote)?;
                self.expect(b'"', |_| Error::new(ErrorCode::UnclosedDelimiter(b'"')))?;
                Ok(Token::Text(result))
            }
            Some(b'0'..=b'9') => Ok(Token::Text(Text::Str(self.number()?))),
            Some(b) if IDENTIFIER_ALLOWED[b as usize] => {
                Ok(Token::Variable(self.identifier()?.into()))
            }
            found => Err(self.with_error_span(Error::expected("value", found))),
        }
    }

    /// Skip comments and whitespace, then read a comma and the next field key, if present.
    ///
    /// A trailing comma is consumed, but a closing delimiter is left for finalization.
    /// Other input without a comma also returns None; finalization validates the closer.
    #[inline]
    fn field_or_terminal(&mut self) -> Result<Option<FieldKey<&'r str>>> {
        self.comment();
        // SAFETY: the expected byte is ASCII.
        if unsafe { self.consume_ascii(b',') } {
            self.comment();
            if self.at_entry_end() {
                Ok(None)
            } else {
                self.identifier().map(|id| Some(id.into()))
            }
        } else {
            Ok(None)
        }
    }

    /// Skip comments and whitespace, then validate and consume the closing entry delimiter.
    ///
    /// EOF here reports an unclosed delimiter at the optional opener offset.
    /// A mismatch consumes only leading comments and whitespace.
    #[inline]
    fn end_entry(&mut self, closing: EntryDelimiter, opening: Option<usize>) -> Result<()> {
        self.comment();
        self.expect(closing as u8, |found| match found {
            Some(found) => Error::new(ErrorCode::ExpectedEndOfEntry {
                expected: closing as u8,
                found,
            }),
            None => Error::new(ErrorCode::UnclosedDelimiter(closing.opening()))
                .with_span(opening.map(|start| start..start + 1)),
        })
    }

    /// Attach the current error span if the error does not already have one.
    fn with_error_span(&self, error: Error) -> Error {
        error.with_span(self.error_span())
    }

    /// Locate an ASCII delimiter immediately after consuming it.
    ///
    /// Call immediately after [`start_entry`](Self::start_entry) or a successful
    /// [`next_entry_or_eof`](Self::next_entry_or_eof), before advancing the reader again.
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

    /// Read an entry key.
    fn entry_key(&mut self) -> Result<EntryKey<&'r str>> {
        self.comment();
        Ok(self.identifier()?.into())
    }

    /// Skip comments and whitespace, then read an identifier as a variable.
    #[inline]
    fn variable(&mut self) -> Result<Variable<&'r str>> {
        self.comment();
        let id = self.identifier()?;
        Ok(id.into())
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

    /// Parse protected text inside `@comment`.
    fn comment_contents(&mut self) -> Result<Text<&'r str, &'r [u8]>> {
        self.comment();
        let closing = self.start_entry()?;
        let opening = self.opening_offset();
        let result = match closing {
            EntryDelimiter::Parenthesis => self.protected(TextDelimiter::Parenthesis)?,
            EntryDelimiter::Brace => self.balanced()?,
        };
        self.end_entry(closing, opening)?;
        Ok(result)
    }

    /// Clear `scratch` and read a nonempty value into it, leaving its terminator unconsumed.
    fn value_into(&mut self, scratch: &mut Vec<Token<&'r str, &'r [u8]>>) -> Result<()> {
        self.value_into_spanned(scratch).map(|_| ())
    }

    /// Read a value into `scratch`, returning its span before trailing comments or whitespace.
    ///
    /// Macro expansion is left to the caller. Entry delimiters remain unconsumed.
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

    /// Ignore the contents of a preamble.
    fn ignore_preamble(&mut self) -> Result<()> {
        entry(self, Self::ignore_value)
    }

    /// Ignore the contents of a macro definition.
    fn ignore_macro(&mut self) -> Result<()> {
        entry(self, |parser| {
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
        entry(self, |parser| {
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
        entry(self, |parser| {
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

impl<'r, R: BibtexReadInner<'r> + ?Sized> BibtexReadInner<'r> for &mut R {
    fn byte_offset(&self) -> Option<usize> {
        (**self).byte_offset()
    }

    fn error_span(&self) -> Option<Range<usize>> {
        (**self).error_span()
    }

    fn peek(&self) -> Option<u8> {
        (**self).peek()
    }

    unsafe fn consume_ascii(&mut self, expected: u8) -> bool {
        // SAFETY: the caller guarantees expected is ASCII.
        unsafe { (**self).consume_ascii(expected) }
    }

    fn comment(&mut self) {
        (**self).comment();
    }

    fn next_entry_or_eof(&mut self) -> bool {
        (**self).next_entry_or_eof()
    }

    fn identifier(&mut self) -> Result<Identifier<&'r str>> {
        (**self).identifier()
    }

    fn number(&mut self) -> Result<&'r str> {
        (**self).number()
    }

    fn text_until(&mut self, delimiter: TextDelimiter) -> Result<Text<&'r str, &'r [u8]>> {
        (**self).text_until(delimiter)
    }
}

fn entry<'r, R: BibtexReadInner<'r> + ?Sized, T>(
    reader: &mut R,
    body: impl FnOnce(&mut R) -> Result<T>,
) -> Result<T> {
    let closing = reader.start_entry()?;
    let opening = reader.opening_offset();
    let value = body(reader)?;
    reader.end_entry(closing, opening)?;
    Ok(value)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{BibtexRead, SliceReader, StrReader, error::Category};

    #[test]
    fn expect_constructs_errors_only_on_failure() {
        let mut reader = StrReader::new("=");
        reader
            .expect(b'=', |_| panic!("constructed an error on success"))
            .unwrap();
        let error = reader
            .expect(b'=', |found| Error::expected("field separator '='", found))
            .unwrap_err();
        assert_eq!(error.classify(), Category::Eof);
        assert_eq!(error.span(), Some(1..1));
    }

    #[test]
    fn slice_reader_spans_remain_byte_oriented() {
        for input in ["🍄".as_bytes(), b"\x80\x81\x82"] {
            let mut reader = SliceReader::new(input);
            // Set an internal byte position; public callers cannot split a character this way.
            reader.pos = 1;
            assert_eq!(reader.number().unwrap_err().span(), Some(1..2));
        }
    }

    #[test]
    fn field_iteration_leaves_the_closer_for_finalization() {
        fn check<'r>(mut reader: impl BibtexRead<'r>, input: &str) {
            let closing = reader.start_entry().unwrap();
            let opening = reader.opening_offset();
            assert_eq!(reader.entry_key().unwrap().into_inner(), "κ");
            assert_eq!(
                reader.field_or_terminal().unwrap().unwrap().into_inner(),
                "f"
            );
            reader.field_sep().unwrap();
            let mut tokens = Vec::new();
            reader.value_into(&mut tokens).unwrap();
            assert_eq!(tokens.len(), 2);
            assert!(reader.field_or_terminal().unwrap().is_none());
            assert!(reader.at_entry_end());
            assert_eq!(reader.byte_offset(), Some(input.len() - 1));
            reader.end_entry(closing, opening).unwrap();
            assert_eq!(reader.byte_offset(), Some(input.len()));
            assert!(!reader.at_entry_end());
        }

        for input in ["{κ,f={é} # \"中\",}", "(κ,f={é} # \"中\",)"] {
            check(StrReader::new(input), input);
            check(SliceReader::new(input.as_bytes()), input);
        }
    }

    #[test]
    fn finalization_rejects_a_mismatched_closer_without_consuming_it() {
        for input in ["{k,)", "(k,}"] {
            let mut reader = StrReader::new(input);
            let closing = reader.start_entry().unwrap();
            let opening = reader.opening_offset();
            reader.entry_key().unwrap();
            assert!(reader.field_or_terminal().unwrap().is_none());
            let error = reader.end_entry(closing, opening).unwrap_err();
            assert_eq!(error.classify(), Category::Syntax);
            assert_eq!(error.span(), Some(3..4));
            assert_eq!(reader.byte_offset(), Some(3));
        }
    }

    proptest::proptest! {
        #[test]
        fn grammar_operations_preserve_character_boundaries(
            input in ".*",
            operations in proptest::collection::vec(0u8..16, 0..40),
        ) {
            let mut reader = StrReader::new(&input);
            for operation in operations {
                let result = match operation {
                    0 => reader.start_entry().map(drop),
                    1 => reader.end_entry(EntryDelimiter::Brace, None),
                    2 => reader.end_entry(EntryDelimiter::Parenthesis, None),
                    3 => reader.identifier().map(drop),
                    4 => reader.number().map(drop),
                    5 => reader.balanced().map(drop),
                    6 => reader.protected(TextDelimiter::Quote).map(drop),
                    7 => reader.protected(TextDelimiter::Parenthesis).map(drop),
                    8 => reader.field_or_terminal().map(drop),
                    9 => reader.macro_variable_opt().map(drop),
                    10 => reader.single_token().map(drop),
                    11 => reader.next_token_or_end().map(drop),
                    12 => reader.field_sep(),
                    13 => { reader.comment(); Ok(()) }
                    14 => { reader.next_entry_or_eof(); Ok(()) }
                    _ => { reader.comma_opt(); Ok(()) }
                };
                let offset = reader.byte_offset().unwrap();
                proptest::prop_assert!(input.is_char_boundary(offset));
                if let Err(error) = result {
                    proptest::prop_assert!(input.get(error.span().unwrap()).is_some());
                }
            }
        }
    }
}
