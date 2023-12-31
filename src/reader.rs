use crate::error::ParseError;
use std::borrow::Cow;
use crate::parse::read_event;

use crate::bib::{Abbreviation, Event, Field, Identifier, Token, Value};
use std::collections::HashMap;

#[derive(Default)]
pub struct ReaderConfig {
    pub resolve_abbreviations: bool,
}

pub struct Reader<'r> {
    pub config: ReaderConfig,
    input: &'r str,
    abbrevs: StringAbbreviations<'r>,
}

impl<'r> Reader<'r> {
    #[allow(clippy::should_implement_trait)]
    pub fn from_str(input: &'r str) -> Self {
        Reader {
            input,
            config: ReaderConfig::default(),
            abbrevs: StringAbbreviations::default(),
        }
    }

    pub fn read_event(&mut self) -> Result<Event<'r>, ParseError<'r>> {
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
            value: Value::from_iter([Token::text_from("One, ")]),
        })));

        let preamble: Result<Event<'_>, ParseError> =
            Ok(Event::Preamble(Preamble(Value::from_iter([
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
                    value: Value::from_iter([
                        Token::Abbrev(Identifier::from("A")),
                        Token::text_from(" Author"),
                    ]),
                },
                Field {
                    identifier: Identifier::from("title"),
                    value: Value::from_iter([Token::text_from("A title")]),
                },
                Field {
                    identifier: Identifier::from("year"),
                    value: Value::from_iter([Token::text_from("2014")]),
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
