use criterion::{criterion_group, criterion_main, Criterion};

pub fn criterion_benchmark(c: &mut Criterion) {
    use serde::de::IgnoredAny;
    use serde::Deserialize;
    use serde_bibtex::entry::{BorrowEntry, Entry};
    use serde_bibtex::{de::Deserializer, MacroDictionary};

    type OwnedBibliography = Vec<Entry>;
    type RawBibliography<'r> = Vec<BorrowEntry<'r>>;

    let input_bytes = std::fs::read("assets/tugboat.bib").unwrap();
    let input_str = std::str::from_utf8(&input_bytes).unwrap();

    c.bench_function("tugboat ignored str", |b| {
        b.iter(|| IgnoredAny::deserialize(&mut Deserializer::from_str(&input_str)))
    });

    c.bench_function("tugboat borrowed str", |b| {
        b.iter(|| RawBibliography::deserialize(&mut Deserializer::from_str(&input_str)))
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
