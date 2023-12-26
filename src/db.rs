use std::collections::HashMap;

use crate::bib::{Entry, EntryKey, Event};
use crate::error::ParseError;
use crate::reader::Reader;

#[derive(Default)]
pub struct Bibliography<'de> {
    entries: HashMap<EntryKey<'de>, Entry<'de>>,
}

impl<'de> Bibliography<'de> {
    pub fn get_entry<T>(&self, key: T) -> Option<&Entry<'de>>
    where
        T: Into<EntryKey<'de>>,
    {
        self.entries.get(&key.into())
    }
    /// Insert an [`Event`] into the bibliography.
    /// ```
    /// use serde_bibtex::Bibliography;
    /// ```
    pub fn insert(&mut self, entry: Entry<'de>) {
        self.entries.insert(entry.key.clone(), entry);
    }
}

impl<'de> Bibliography<'de> {
    #[allow(clippy::should_implement_trait)]
    pub fn from_str(input: &'de str) -> Result<Self, ParseError> {
        let mut reader = Reader::from_str(input);
        reader.config.resolve_abbreviations = true;

        let mut bibliography = Self::default();

        loop {
            match reader.read_event()? {
                Event::Eof => {
                    return Ok(bibliography);
                }
                Event::Entry(entry) => {
                    bibliography.insert(entry);
                }
                _ => {}
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_str() {
        let bibliography = Bibliography::from_str(
            r#"
            @string{A = "Auth" # {or}}

            @string{A1 = "One, " # A}

            @string{A2 = "Two, " # A}

            @article{key:0,
              author = A1 # " and " # A2,
              title = {A title},
              year = 2014,
            }
            "#,
        )
        .unwrap();

        let entry = bibliography.get_entry("key:0").unwrap();

        assert_eq!(
            entry.to_string(),
            "@article{key:0,\n  \
               author = {One, } # {Auth} # {or} # { and } # {Two, } # {Auth} # {or},\n  \
               title = {A title},\n  \
               year = {2014},\n\
               }",
        );
    }
}
