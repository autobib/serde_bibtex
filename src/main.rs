use serde::de::IgnoredAny;
use serde::Deserialize;

use serde_bibtex::entry::{BorrowEntry, Entry};
use serde_bibtex::error::Result;
use serde_bibtex::{de::Deserializer, MacroDictionary};

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

    let input_bytes = std::fs::read("assets/tugboat.bib").unwrap();
    let input_str = std::str::from_utf8(&input_bytes).unwrap();
    let args: Vec<String> = std::env::args().collect();

    match args.get(1) {
        Some(arg) => match arg.as_str() {
            "ignore" => {
                let _ = IgnoredAny::deserialize(&mut Deserializer::from_str(&input_str));
            }
            "borrow" => {
                let _ = RawBibliography::deserialize(&mut Deserializer::from_str(&input_str));
            }
            "struct" => {
                let de_iter = Deserializer::from_str(&input_str).into_iter_regular_entry();
                let _result: Vec<Result<TugboatEntry>> = de_iter.collect();
            }
            "copy" => {
                let mut macros = MacroDictionary::default();
                macros.set_month_macros();
                let _ = OwnedBibliography::deserialize(&mut Deserializer::from_str_with_macros(
                    &input_str, macros,
                ));
            }
            other => eprintln!(
                "Invalid argument '{other}': provide argument 'ignore' 'borrow' struct' 'copy'"
            ),
        },
        None => eprintln!("Error: provide argument 'ignore' 'borrow' struct' 'copy'"),
    }
}
