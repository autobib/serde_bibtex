use criterion::{criterion_group, criterion_main, Criterion};

pub fn criterion_benchmark(c: &mut Criterion) {
    use serde::de::IgnoredAny;
    use serde::Deserialize;
    use serde_bibtex::entry::{BorrowEntry, Entry};
    use serde_bibtex::{de::Deserializer, MacroDictionary};

    type OwnedBibliography = Vec<Entry>;
    type RawBibliography<'r> = Vec<BorrowEntry<'r>>;

    let input_bytes = std::fs::read("assets/tugboat.bib").unwrap();

    c.bench_function("tugboat ignored str-convert", |b| {
        b.iter(|| {
            let input_str = std::str::from_utf8(&input_bytes).unwrap();
            IgnoredAny::deserialize(&mut Deserializer::from_str(input_str))
        })
    });

    c.bench_function("tugboat ignored slice", |b| {
        b.iter(|| IgnoredAny::deserialize(&mut Deserializer::from_slice(&input_bytes)))
    });

    c.bench_function("tugboat borrowed str-convert", |b| {
        b.iter(|| {
            let input_str = std::str::from_utf8(&input_bytes).unwrap();
            RawBibliography::deserialize(&mut Deserializer::from_str(input_str))
        })
    });

    c.bench_function("tugboat borrowed slice", |b| {
        b.iter(|| RawBibliography::deserialize(&mut Deserializer::from_slice(&input_bytes)))
    });

    c.bench_function("tugboat owned str-convert", |b| {
        b.iter(|| {
            let mut macros = MacroDictionary::default();
            macros.set_month_macros();
            let input_str = std::str::from_utf8(&input_bytes).unwrap();
            OwnedBibliography::deserialize(&mut Deserializer::from_str_with_macros(
                input_str, macros,
            ))
        })
    });

    c.bench_function("tugboat owned slice", |b| {
        b.iter(|| {
            let mut macros = MacroDictionary::default();
            macros.set_month_macros();
            OwnedBibliography::deserialize(&mut Deserializer::from_slice_with_macros(
                &input_bytes,
                macros,
            ))
        })
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
