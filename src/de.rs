mod bibliography;
mod entry;
mod value;

pub use bibliography::{DeserializeEntriesIter, DeserializeIter, Deserializer};

use crate::error::Error;
use crate::parse::{SliceReader, StrReader};

use serde::Deserialize;

pub fn from_str<'r, D>(s: &'r str) -> Result<D, Error>
where
    D: Deserialize<'r>,
{
    let reader = StrReader::new(s);
    let mut deserializer = Deserializer::new(reader);
    D::deserialize(&mut deserializer)
}

pub fn from_bytes<'r, D>(s: &'r [u8]) -> Result<D, Error>
where
    D: Deserialize<'r>,
{
    let reader = SliceReader::new(s);
    let mut deserializer = Deserializer::new(reader);
    D::deserialize(&mut deserializer)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::borrow::Cow;
    use std::iter::zip;

    #[derive(Deserialize, Debug, PartialEq)]
    enum TestEntry<'a> {
        #[serde(borrow)]
        Entry(TestRegularEntry<'a>),
        Abbreviation,
        Comment,
        Preamble,
    }

    #[derive(Deserialize, Debug, PartialEq)]
    struct TestRegularEntry<'a> {
        entry_type: &'a str,
        citation_key: &'a str,
        #[serde(borrow)]
        fields: TestFields<'a>,
    }

    #[derive(Deserialize, Debug, PartialEq)]
    struct TestFields<'a> {
        #[serde(borrow)]
        author: Cow<'a, str>,
        #[serde(borrow)]
        title: Cow<'a, str>,
    }

    #[test]
    fn test_deserialize_iter() {
        let input = r#"
        @article{key,
           author = {One, Author} # { and } # {Two, Author},
           title = {Title},
           year = 2024,
           nonsense = 12,
        }

        @string(k={val})

        @preamble{"a value"}

        @comment{ignored}

        @book{key2,
           author = {Auth},
           title = k # 1 # {Year},
        }
        "#;

        let expected = vec![
            TestEntry::Entry(TestRegularEntry {
                entry_type: "article",
                citation_key: "key",
                fields: TestFields {
                    author: "One, Author and Two, Author".into(),
                    title: "Title".into(),
                },
            }),
            TestEntry::Abbreviation,
            TestEntry::Preamble,
            TestEntry::Comment,
            TestEntry::Entry(TestRegularEntry {
                entry_type: "book",
                citation_key: "key2",
                fields: TestFields {
                    author: "Auth".into(),
                    title: "val1Year".into(),
                },
            }),
        ];

        let reader = StrReader::new(input);
        for (expected, received) in zip(expected.into_iter(), Deserializer::new(reader).into_iter())
        {
            assert_eq!(Ok(expected), received);
        }
    }

    #[test]
    fn test_deserialize_iter_entries() {
        let input = r#"
        @string{k = {12}}

        @article{key,
           author = {Author},
           title = k
        }

        @string{k = k # k}

        @article{k2,
           author = {Author 2},
           title = k # k,
        }
        "#;

        let expected = vec![
            TestRegularEntry {
                entry_type: "article",
                citation_key: "key",
                fields: TestFields {
                    author: "Author".into(),
                    title: "12".into(),
                },
            },
            TestRegularEntry {
                entry_type: "article",
                citation_key: "k2",
                fields: TestFields {
                    author: "Author 2".into(),
                    title: "12121212".into(),
                },
            },
        ];

        let reader = StrReader::new(input);
        for (expected, received) in zip(
            expected.into_iter(),
            Deserializer::new(reader).into_iter_entry(),
        ) {
            assert_eq!(Ok(expected), received);
        }
    }
}
