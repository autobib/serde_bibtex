use serde::de::IgnoredAny;
use serde::Deserialize;
use serde_bibtex::Error;
use serde_bibtex::{SliceReader, StrReader};

use std::collections::HashMap;

// Anonymous field names and flexible receiver type
#[derive(Debug, Deserialize, PartialEq)]
enum Tok<'a> {
    #[serde(rename = "Abbrev")]
    A(&'a [u8]),
    #[serde(rename = "Text")]
    T(&'a [u8]),
}

#[derive(Deserialize, Debug, PartialEq)]
struct TestEntryMap<'a> {
    entry_type: &'a str,
    citation_key: &'a str,
    #[serde(borrow)]
    fields: HashMap<&'a str, Vec<Tok<'a>>>,
}

#[derive(Deserialize, Debug, PartialEq)]
enum TestEntry<'a> {
    #[serde(borrow)]
    Entry(TestEntryMap<'a>),
    #[serde(borrow)]
    Abbreviation(Option<(&'a str, Vec<Tok<'a>>)>),
    #[serde(borrow)]
    Comment(&'a str),
    #[serde(borrow)]
    Preamble(Vec<Tok<'a>>),
}

macro_rules! test_file_slice {
    ($fname:expr) => {
        let input_bytes = std::fs::read($fname).unwrap();

        let mut de = SliceReader::new(&input_bytes).deserialize();
        let data: Result<IgnoredAny, Error> = IgnoredAny::deserialize(&mut de);
        assert!(data.is_ok(), "{:?}", data);

        let mut de = SliceReader::new(&input_bytes).deserialize();
        let data: Result<TestBib, Error> = TestBib::deserialize(&mut de);
        assert!(data.is_ok(), "{:?}", data);
    };
}

macro_rules! test_file_str {
    ($fname:expr) => {
        let input_bytes = std::fs::read($fname).unwrap();
        let input_str = std::str::from_utf8(&input_bytes).unwrap();

        let mut de = StrReader::new(&input_str).deserialize();
        let data: Result<IgnoredAny, Error> = IgnoredAny::deserialize(&mut de);
        assert!(data.is_ok(), "{:?}", data);

        let mut de = StrReader::new(&input_str).deserialize();
        let data: Result<TestBib, Error> = TestBib::deserialize(&mut de);
        assert!(data.is_ok(), "{:?}", data);
    };
}

type TestBib<'a> = Vec<TestEntry<'a>>;

#[test]
fn test_syntax_tugboat() {
    test_file_slice!("assets/tugboat.bib");
    test_file_str!("assets/tugboat.bib");
}

#[test]
fn test_syntax_biber() {
    test_file_slice!("assets/biber_test.bib");
    test_file_str!("assets/biber_test.bib");
}

#[test]
fn test_syntax_large() {
    let paths = std::fs::read_dir("assets/syntax").unwrap();
    for path in paths {
        println!("Testing: {:?}", path.as_ref().unwrap().path());
        test_file_slice!(path.as_ref().unwrap().path());
    }
}
