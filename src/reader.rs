use crate::error::ParseError;
use crate::parse::read_event;

use crate::bib::{Abbreviation, Event, Field, Identifier, Token, Value};
use std::collections::HashMap;

#[derive(Default)]
pub struct ReaderConfig {
    pub resolve_abbreviations: bool,
}

pub struct Reader<'de> {
    pub config: ReaderConfig,
    input: &'de str,
    abbrevs: BibliographyAbbreviations<'de>,
}

impl<'de> Reader<'de> {
    #[allow(clippy::should_implement_trait)]
    pub fn from_str(input: &'de str) -> Self {
        Reader {
            input,
            config: ReaderConfig::default(),
            abbrevs: BibliographyAbbreviations::default(),
        }
    }

    pub fn read_event(&mut self) -> Result<Event<'de>, ParseError<'de>> {
        let (input, mut event) = read_event(self.input)?;
        self.input = input;

        if self.config.resolve_abbreviations {
            match event {
                Event::Entry(ref mut entry) => {
                    for field in entry.fields.iter_mut() {
                        self.abbrevs.resolve(&mut field.value);
                    }
                }
                Event::Preamble(ref mut preamble) => {
                    self.abbrevs.resolve(&mut preamble.0);
                }
                Event::String(Abbreviation(Field {
                    ref identifier,
                    ref mut value,
                })) => {
                    self.abbrevs.insert(identifier.clone(), value.clone());
                }
                _ => {}
            }
        }

        Ok(event)
    }
}

#[derive(Default)]
pub struct BibliographyAbbreviations<'de> {
    abbrevs: HashMap<Identifier<'de>, Value<'de>>,
    buffer: Vec<Token<'de>>,
}

impl<'de> BibliographyAbbreviations<'de> {
    pub fn insert(&mut self, identifier: Identifier<'de>, mut value: Value<'de>) {
        self.resolve(&mut value);
        self.abbrevs.insert(identifier, value);
    }

    /// Resolve the abbreviations in `value`. Since it may not be possible to do this in-place
    /// efficiently, we also use a pre-allocated buffer. Note that the abbreviations must be
    /// present in the abbreviation `HashMap` in order to be resolved.
    pub fn resolve(&mut self, value: &mut Value<'de>) {
        let mut breakpoint = None;

        // Try to mutate Value in-place by substituting abbreviations
        // if they have length exactly 1.
        for (idx, token) in value.0.iter_mut().enumerate() {
            if let Token::Abbrev(identifier) = token {
                if let Some(abbrev_value) = self.abbrevs.get(identifier) {
                    if abbrev_value.0.len() == 1 {
                        *token = abbrev_value.0[0].clone();
                    } else {
                        breakpoint = Some(idx);
                        break;
                    }
                }
            }
        }

        // If this fails, we have an abbreviation with length not equal to 1.
        // Since we can no longer mutate Value in place, first push to a Buffer
        // and then extend Value from the buffer.
        if let Some(pos) = breakpoint {
            self.buffer.clear();

            for token in value.0[pos..].iter() {
                match token {
                    Token::Abbrev(identifier) => match self.abbrevs.get(identifier) {
                        Some(abbrev_value) => {
                            self.buffer.extend_from_slice(&abbrev_value.0[..]);
                        }
                        _ => {
                            self.buffer.push((*token).clone());
                        }
                    },
                    _ => {
                        self.buffer.push((*token).clone());
                    }
                }
            }

            value.0.truncate(pos);
            value.0.append(&mut self.buffer);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bib::{
        Abbreviation, Entry, EntryKey, Event, Field, Identifier, Preamble, Token, Value,
    };

    #[test]
    fn test_reader() {
        let mut reader = Reader::from_str(
            r#"
            @string{A = "One, "}

            @preamble{"\macro{" # A # "}"}

            @article{key:0,
              author = A # " Author",
              title = {A title},
              year = 2014,
            }
            "#,
        );

        let string: Result<Event<'_>, ParseError> = Ok(Event::String(Abbreviation(Field {
            identifier: Identifier::from("A"),
            value: Value(vec![Token::text_from("One, ")]),
        })));

        let preamble: Result<Event<'_>, ParseError> = Ok(Event::Preamble(Preamble(Value(vec![
            Token::text_from("\\macro{"),
            Token::abbrev_from("A"),
            Token::text_from("}"),
        ]))));

        let article: Result<Event<'_>, ParseError> = Ok(Event::Entry(Entry {
            entry_type: Identifier::from("article"),
            key: EntryKey::from("key:0"),
            fields: vec![
                Field {
                    identifier: Identifier::from("author"),
                    value: Value(vec![
                        Token::Abbrev(Identifier::from("A")),
                        Token::text_from(" Author"),
                    ]),
                },
                Field {
                    identifier: Identifier::from("title"),
                    value: Value(vec![Token::text_from("A title")]),
                },
                Field {
                    identifier: Identifier::from("year"),
                    value: Value(vec![Token::text_from("2014")]),
                },
            ],
        }));

        assert_eq!(reader.read_event(), string);
        assert_eq!(reader.read_event(), preamble);
        assert_eq!(reader.read_event(), article);
        assert_eq!(reader.read_event(), Ok(Event::Eof));
        assert_eq!(reader.read_event(), Ok(Event::Eof));
    }
}
