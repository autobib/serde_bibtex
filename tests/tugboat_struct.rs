use serde::Deserialize;
use serde_bibtex::de::Deserializer;

use std::borrow::Cow;

#[derive(Debug, PartialEq, Deserialize)]
struct Fields<'r> {
    #[serde(borrow)]
    author: Cow<'r, str>,
    #[serde(borrow)]
    title: Cow<'r, str>,
    #[serde(borrow)]
    journal: Cow<'r, str>,
    #[serde(borrow)]
    volume: Cow<'r, str>,
    #[serde(borrow)]
    number: Cow<'r, str>,
    #[serde(borrow)]
    pages: Cow<'r, str>,
    #[serde(borrow)]
    year: Cow<'r, str>,
    #[serde(borrow)]
    #[serde(rename = "ISSN")]
    issn: Cow<'r, str>,
    #[serde(borrow)]
    #[serde(rename = "ISSN-L")]
    issn_l: Option<Cow<'r, str>>,
    #[serde(borrow)]
    bibdate: Cow<'r, str>,
    #[serde(borrow)]
    bibsource: Cow<'r, str>,
    #[serde(borrow)]
    #[serde(rename = "URL")]
    url: Option<Cow<'r, str>>,
    #[serde(borrow)]
    acknowledgement: Cow<'r, str>,
    #[serde(borrow)]
    issue: Cow<'r, str>,
    #[serde(borrow)]
    #[serde(rename = "journal-URL")]
    journal_url: Cow<'r, str>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct TugboatEntry<'r> {
    entry_key: &'r str,
    #[serde(borrow)]
    fields: Fields<'r>,
}

#[test]
fn test_tugboat_struct() {
    let input_bytes = std::fs::read("assets/tugboat.bib").unwrap();
    let de_iter = Deserializer::from_slice(&input_bytes).into_iter_entry::<TugboatEntry>();
    for res in de_iter {
        assert!(res.is_ok(), "{:?}", res)
    }
}
