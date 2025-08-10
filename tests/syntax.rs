use pest::Parser;
use serde::Deserialize;
use serde::de::IgnoredAny;
use serde_bibtex::{
    Result,
    entry::{OwnedBibliography, RawBibliography},
    syntax::{BibtexParser, Rule},
    {MacroDictionary, de::Deserializer},
};

use std::collections::HashMap;

// Anonymous field names and flexible receiver type
#[derive(Debug, Deserialize, PartialEq)]
enum Tok<'a> {
    #[serde(rename = "Variable")]
    V(&'a [u8]),
    #[serde(rename = "Text")]
    T(&'a [u8]),
}

#[derive(Deserialize, Debug, PartialEq)]
struct TestEntryMap<'a> {
    entry_type: &'a str,
    entry_key: &'a str,
    #[serde(borrow)]
    fields: HashMap<&'a str, Vec<Tok<'a>>>,
}

#[derive(Deserialize, Debug, PartialEq)]
enum TestEntry<'a> {
    #[serde(borrow)]
    Regular(TestEntryMap<'a>),
    #[serde(borrow)]
    Macro(Option<(&'a str, Vec<Tok<'a>>)>),
    #[serde(borrow)]
    Comment(&'a str),
    #[serde(borrow)]
    Preamble(Vec<Tok<'a>>),
}

macro_rules! test_file_types {
    ($fname:expr) => {
        let input_bytes = std::fs::read($fname).unwrap();

        let mut macros = MacroDictionary::<&str, &[u8]>::default();
        macros.set_month_macros();

        let mut de = Deserializer::from_slice_with_macros(&input_bytes, macros);
        let data: Result<OwnedBibliography> = OwnedBibliography::deserialize(&mut de);
        assert!(data.is_ok(), "{:?}", data);

        let mut de = Deserializer::from_slice(&input_bytes);
        let data: Result<RawBibliography> = RawBibliography::deserialize(&mut de);
        assert!(data.is_ok(), "{:?}", data);
    };
}

macro_rules! test_file_slice {
    ($fname:expr) => {
        let input_bytes = std::fs::read($fname).unwrap();

        let mut de = Deserializer::from_slice(&input_bytes);
        let data: Result<IgnoredAny> = IgnoredAny::deserialize(&mut de);
        assert!(data.is_ok(), "{:?}", data);

        let mut de = Deserializer::from_slice(&input_bytes);
        let data: Result<TestBib> = TestBib::deserialize(&mut de);
        assert!(data.is_ok(), "{:?}", data);
    };
}

macro_rules! test_file_str {
    ($fname:expr) => {
        let input_bytes = std::fs::read($fname).unwrap();
        let input_str = std::str::from_utf8(&input_bytes).unwrap();

        let mut de = Deserializer::from_str(&input_str);
        let data: Result<IgnoredAny> = IgnoredAny::deserialize(&mut de);
        assert!(data.is_ok(), "{:?}", data);

        let mut de = Deserializer::from_str(&input_str);
        let data: Result<TestBib> = TestBib::deserialize(&mut de);
        assert!(data.is_ok(), "{:?}", data);

        let parsed = BibtexParser::parse(Rule::bib, input_str);
        assert!(parsed.is_ok());
    };
}

type TestBib<'a> = Vec<TestEntry<'a>>;

#[test]
fn test_syntax_tugboat() {
    test_file_types!("assets/tugboat.bib");
    test_file_slice!("assets/tugboat.bib");
    test_file_str!("assets/tugboat.bib");
}

#[test]
fn test_syntax_biber() {
    test_file_types!("assets/biber_test.bib");
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
