use criterion::{criterion_group, criterion_main, Criterion};

pub fn criterion_benchmark(c: &mut Criterion) {
    use serde::de::IgnoredAny;
    use serde::Deserialize;
    use serde_bibtex::entry::{BorrowEntry, Entry};
    use serde_bibtex::{de::Deserializer, MacroDictionary};
    use std::error::Error;

    use biblatex::{Bibliography, RawBibliography as RawBib};
    use bibparser::{BibEntry, Parser};
    use nom_bibtex::Bibtex;
    use std::str::FromStr;

    type OwnedBibliography = Vec<Entry>;
    type RawBibliography<'r> = Vec<BorrowEntry<'r>>;

    let input_bytes = std::fs::read("assets/tugboat.bib").unwrap();
    let input_str = std::str::from_utf8(&input_bytes).unwrap();

    c.bench_function("tugboat ignored slice", |b| {
        b.iter(|| IgnoredAny::deserialize(&mut Deserializer::from_slice(&input_bytes)))
    });

    c.bench_function("tugboat owned str", |b| {
        b.iter(|| {
            let mut macros = MacroDictionary::default();
            macros.set_month_macros();
            OwnedBibliography::deserialize(&mut Deserializer::from_str_with_macros(
                &input_str, macros,
            ))
        })
    });

    c.bench_function("tugboat raw slice", |b| {
        b.iter(|| RawBibliography::deserialize(&mut Deserializer::from_slice(&input_bytes)))
    });

    c.bench_function("tugboat raw str", |b| {
        b.iter(|| RawBibliography::deserialize(&mut Deserializer::from_str(&input_str)))
    });

    // c.bench_function("tugboat bibparser", |b| {
    //     b.iter(|| {
    //         let mut p = Parser::from_str(input_str).unwrap();
    //         let _result: Vec<Result<BibEntry, Box<dyn Error>>> = p.iter().collect();
    //     })
    // });

    c.bench_function("tugboat biblatex", |b| {
        b.iter(|| Bibliography::parse(&input_str).unwrap())
    });

    c.bench_function("tugboat biblatex raw", |b| {
        b.iter(|| RawBib::parse(&input_str).unwrap())
    });

    c.bench_function("tugboat nom", |b| {
        b.iter(|| Bibtex::parse(&input_str).unwrap())
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
