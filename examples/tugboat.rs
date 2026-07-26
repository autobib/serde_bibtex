//! # Example struct deserializartion
//! An example deserialization of the `assets/tugboat.bib` database
use serde::Deserialize;
use serde::de::IgnoredAny;

use serde_bibtex::Result;
use serde_bibtex::entry::{BorrowEntry, Entry};
use serde_bibtex::{MacroDictionary, de::Deserializer};

use std::borrow::Cow;

fn main() {
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

    type OwnedBibliography = Vec<Entry>;
    type RawBibliography<'r> = Vec<BorrowEntry<'r>>;

    let input = std::fs::read("assets/tugboat.bib").unwrap();
    let args: Vec<String> = std::env::args().collect();

    match args.get(1) {
        Some(arg) => match arg.as_str() {
            "ignore" => {
                let _ = IgnoredAny::deserialize(&mut Deserializer::from_slice(&input));
            }
            "borrow" => {
                let _ = RawBibliography::deserialize(&mut Deserializer::from_slice(&input));
            }
            "struct" => {
                let de_iter = Deserializer::from_slice(&input).into_iter_regular_entry();
                let _result: Result<Vec<TugboatEntry>> = de_iter.collect();
            }
            "copy" => {
                let mut macros = MacroDictionary::default();
                macros.set_month_macros();
                let _ = OwnedBibliography::deserialize(&mut Deserializer::from_slice_with_macros(
                    &input, macros,
                ));
            }
            other => eprintln!(
                "Invalid argument '{other}', must be one of: ignore, borrow, struct, copy"
            ),
        },
        None => eprintln!(
            "Error: provide argument one of the following arguments: ignore, borrow, struct, copy"
        ),
    }
}
