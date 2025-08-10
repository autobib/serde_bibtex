use criterion::{Criterion, criterion_group, criterion_main};

pub fn criterion_benchmark(c: &mut Criterion) {
    use serde::Deserialize;
    use serde::de::IgnoredAny;
    use serde_bibtex::de::Deserializer;
    use serde_bibtex::entry::{BorrowEntry, Entry};

    type OwnedBibliography = Vec<Entry>;
    type RawBibliography<'r> = Vec<BorrowEntry<'r>>;

    let input_bytes = std::fs::read("assets/biber_test.bib").unwrap();
    let input_str = std::str::from_utf8(&input_bytes).unwrap();

    c.bench_function("biber ignored slice", |b| {
        b.iter(|| IgnoredAny::deserialize(&mut Deserializer::from_slice(&input_bytes)))
    });

    c.bench_function("biber ignored str", |b| {
        b.iter(|| IgnoredAny::deserialize(&mut Deserializer::from_str(input_str)))
    });

    c.bench_function("biber owned slice", |b| {
        b.iter(|| OwnedBibliography::deserialize(&mut Deserializer::from_slice(&input_bytes)))
    });

    c.bench_function("biber owned str", |b| {
        b.iter(|| OwnedBibliography::deserialize(&mut Deserializer::from_str(input_str)))
    });

    c.bench_function("biber borrowed slice", |b| {
        b.iter(|| RawBibliography::deserialize(&mut Deserializer::from_slice(&input_bytes)))
    });

    c.bench_function("biber borrowed str", |b| {
        b.iter(|| RawBibliography::deserialize(&mut Deserializer::from_str(input_str)))
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
