pub mod de;

pub mod bib;
pub mod error;
pub(crate) mod naming;
mod read;

#[cfg(test)]
mod tests {
    use crate::de::BibtexDeserializer;
    use crate::error::Error;
    use crate::read::StrReader;
    use serde::Deserialize;
    use std::borrow::Cow;

    #[derive(Deserialize, Debug, PartialEq)]
    enum TestChunk<'a> {
        #[serde(borrow)]
        Entry(TestEntry<'a>),
        Abbreviation,
        Comment,
        Preamble,
    }

    #[derive(Deserialize, Debug, PartialEq)]
    struct TestEntry<'a> {
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

    type TestBibliography<'a> = Vec<TestChunk<'a>>;

    #[test]
    fn example_capture_entries() {
        let input = r#"
        @article{key,
           author = {One, Author} # { and } # {Two, Author},
           title = {Title},
           year = 2024,
           nonsense = 12,
        }

        @string(k={v})

        @preamble{"a value"}

        @comment{ignored}

        @book{key2,
           author = {Auth},
           title = 1 # {Year},
        }
        "#;

        let reader = StrReader::new(input);
        let mut bib_de = BibtexDeserializer::new(reader);

        let data: Result<TestBibliography, Error> = TestBibliography::deserialize(&mut bib_de);
        assert!(data.is_ok());
    }
}
