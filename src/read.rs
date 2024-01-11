use crate::abbrev::Abbreviations;
use crate::de::EntryDeserializer;
use crate::error::Error;

struct Reader<'s, 'r, D> {
    de: EntryDeserializer<'s, 'r>,
    _marker: std::marker::PhantomData<D>,
}

impl<'s, 'r, D> Reader<'s, 'r, D> {
    pub fn from_str(input: &'r str, abbrevs: &'s mut Abbreviations<'r>) -> Self {
        Self {
            de: EntryDeserializer::new(input, abbrevs),
            _marker: std::marker::PhantomData,
        }
    }
}

impl<'s, 'r, T> Iterator for Reader<'s, 'r, T>
where
    T: serde::de::Deserialize<'r>,
{
    type Item = Result<T, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        Some(T::deserialize(&mut self.de))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::value::Entry;
    use serde::Deserialize;
    use std::collections::HashMap;

    // Anonymous field names and flexible receiver type
    #[derive(Debug, Deserialize, PartialEq)]
    enum Tok<'a> {
        #[serde(rename = "Abbrev")]
        A(&'a str),
        #[serde(rename = "Text")]
        T(&'a str),
    }

    #[derive(Deserialize, Debug, PartialEq)]
    struct TestEntryMap<'a> {
        entry_type: &'a str,
        citation_key: &'a str,
        #[serde(borrow)]
        fields: HashMap<&'a str, Vec<Tok<'a>>>,
    }

    #[test]
    fn test_read() {
        // @string{and = { and }}
        let input = r#"

            @article{key:1,
              author = {One} # and # {Two},
              title = "Title",
              year = 2012,
            }

            @article{key:2,
              author = {Three} # and # {Four},
              title = "Title 2",
              year = 2015,
            }
        "#;

        let mut abbrevs = Abbreviations::default();

        let rdr = Reader::from_str(input, &mut abbrevs);

        let mut ct = 0;
        for result in rdr {
            if ct == 2 {
                break;
            };
            let entry: Entry = result.unwrap();
            ct += 1;
        }
        // assert!(false);
    }
}
