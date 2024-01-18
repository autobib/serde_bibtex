use criterion::{criterion_group, criterion_main, Criterion};

pub fn criterion_benchmark(c: &mut Criterion) {
    use serde::de::IgnoredAny;
    use serde::Deserialize;
    use serde_bibtex::{SliceReader, StrReader};

    let input_bytes = std::fs::read("assets/tugboat.bib").unwrap();

    c.bench_function("tugboat ignored slice", |b| {
        b.iter(|| IgnoredAny::deserialize(&mut SliceReader::new(&input_bytes).deserialize()))
    });

    let input_str = std::str::from_utf8(&input_bytes).unwrap();

    c.bench_function("tugboat ignored str", |b| {
        b.iter(|| IgnoredAny::deserialize(&mut StrReader::new(&input_str).deserialize()))
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
