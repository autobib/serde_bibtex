use criterion::{criterion_group, criterion_main, Criterion};

pub fn criterion_benchmark(c: &mut Criterion) {
    use serde::de::IgnoredAny;
    use serde::Deserialize;
    use serde_bibtex::entry::{BorrowEntry, Entry};
    use serde_bibtex::error::Result;
    use serde_bibtex::{de::Deserializer, MacroDictionary};

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

    type OwnedBibliography = Vec<Entry>;
    type RawBibliography<'r> = Vec<BorrowEntry<'r>>;

    let input_bytes = std::fs::read("assets/tugboat.bib").unwrap();
    let input_str = std::str::from_utf8(&input_bytes).unwrap();

    c.bench_function("tugboat ignore str", |b| {
        b.iter(|| IgnoredAny::deserialize(&mut Deserializer::from_str(&input_str)))
    });

    c.bench_function("tugboat borrow str", |b| {
        b.iter(|| RawBibliography::deserialize(&mut Deserializer::from_str(&input_str)))
    });

    c.bench_function("tugboat struct str", |b| {
        b.iter(|| {
            let de_iter = Deserializer::from_str(&input_str).into_iter_entry();
            let _result: Vec<Result<TugboatEntry>> = de_iter.collect();
        })
    });

    c.bench_function("tugboat copy str", |b| {
        b.iter(|| {
            let mut macros = MacroDictionary::default();
            macros.set_month_macros();
            OwnedBibliography::deserialize(&mut Deserializer::from_str_with_macros(
                &input_str, macros,
            ))
        })
    });

    use biblatex::RawBibliography as RawBib;

    c.bench_function("tugboat biblatex", |b| {
        b.iter(|| RawBib::parse(&input_str).unwrap())
    });

    use nom_bibtex::Bibtex;

    c.bench_function("tugboat nom", |b| {
        b.iter(|| Bibtex::parse(&input_str).unwrap())
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
