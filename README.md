[![Current crates.io release](https://img.shields.io/crates/v/serde_bibtex)](https://crates.io/crates/serde_bibtex)
[![Documentation](https://img.shields.io/badge/docs.rs-serde__bibtex-66c2a5?labelColor=555555&logoColor=white&logo=data:image/svg+xml;base64,PHN2ZyByb2xlPSJpbWciIHhtbG5zPSJodHRwOi8vd3d3LnczLm9yZy8yMDAwL3N2ZyIgdmlld0JveD0iMCAwIDUxMiA1MTIiPjxwYXRoIGZpbGw9IiNmNWY1ZjUiIGQ9Ik00ODguNiAyNTAuMkwzOTIgMjE0VjEwNS41YzAtMTUtOS4zLTI4LjQtMjMuNC0zMy43bC0xMDAtMzcuNWMtOC4xLTMuMS0xNy4xLTMuMS0yNS4zIDBsLTEwMCAzNy41Yy0xNC4xIDUuMy0yMy40IDE4LjctMjMuNCAzMy43VjIxNGwtOTYuNiAzNi4yQzkuMyAyNTUuNSAwIDI2OC45IDAgMjgzLjlWMzk0YzAgMTMuNiA3LjcgMjYuMSAxOS45IDMyLjJsMTAwIDUwYzEwLjEgNS4xIDIyLjEgNS4xIDMyLjIgMGwxMDMuOS01MiAxMDMuOSA1MmMxMC4xIDUuMSAyMi4xIDUuMSAzMi4yIDBsMTAwLTUwYzEyLjItNi4xIDE5LjktMTguNiAxOS45LTMyLjJWMjgzLjljMC0xNS05LjMtMjguNC0yMy40LTMzLjd6TTM1OCAyMTQuOGwtODUgMzEuOXYtNjguMmw4NS0zN3Y3My4zek0xNTQgMTA0LjFsMTAyLTM4LjIgMTAyIDM4LjJ2LjZsLTEwMiA0MS40LTEwMi00MS40di0uNnptODQgMjkxLjFsLTg1IDQyLjV2LTc5LjFsODUtMzguOHY3NS40em0wLTExMmwtMTAyIDQxLjQtMTAyLTQxLjR2LS42bDEwMi0zOC4yIDEwMiAzOC4ydi42em0yNDAgMTEybC04NSA0Mi41di03OS4xbDg1LTM4Ljh2NzUuNHptMC0xMTJsLTEwMiA0MS40LTEwMi00MS40di0uNmwxMDItMzguMiAxMDIgMzguMnYuNnoiPjwvcGF0aD48L3N2Zz4K)](https://docs.rs/serde_bibtex/)

# WARNING
This crate is under active development and the public API may change substantially on every minor version change.
The deserialization API is relatively stable, but serialization is not yet implemented and some of the publicly-exposed internal state may change.
Until this is stabilized, use at your own risk!

# Serde bibtex
A [Rust](https://www.rust-lang.org/) library providing a [serde](https://serde.rs/) interface for `.bib` file (de)serialization.
The implementation is minimally opinionated and feature-rich for convenient downstream consumption by other libraries or binaries.

For examples and a thorough documentation of features, visit the [docs](https://docs.rs/serde_bibtex/latest/serde_bibtex).

## Deserializer
Here are the main features.

### Flexible
  - Structured: read into Rust types with automatic `@string` macro expansion and other convenience features.
  - Unstructured: do not expand macros or collect fields values to preserve the structure of the original bibtex.
  - Deserialize from bytes to defer UTF-8 conversion, or even pass-through raw bytes.
  - Error-tolerant `Iterator` API that allows skipping malformed entries.

### Explicit and unambiguous syntax
  - Aims for compatibility with and tested against an independently implemented [pest grammar](/src/syntax/bibtex.pest).
  - Aim for compatibility with [biber](https://github.com/plk/biber) but without some of biber's [undocumented idiosyncracies](https://docs.rs/serde_bibtex/latest/serde_bibtex/syntax/index.html#differences-from-biber) or [unfixable parsing bugs](https://github.com/plk/biber/issues/456).

### Fast
  - Low overhead manual parser implementation (see [benchmarks](#benchmarks)).
  - Zero-copy deserialization.
  - Selective capturing of contents (see [benchmarks](#benchmarks) for speed differences)


## Serializer
TODO: not yet implemented


## Comparison with other crates
### [typst/biblatex](https://github.com/typst/biblatex)
We do not attempt to interpret the contents of the entries in the `.bib` file and instead defer interpretation for downstream consumption.
On the other hand, [biblatex](https://github.com/typst/biblatex) is intended to support [typst](https://github.com/typst/typst), which requires interpreting the contents of the fields (for example, parsing of `$math$` in field values).
In this sense, we might consider our implementation closer to the `biblatex::RawBibliography` entrypoint, but with the substantial extra flexibility of reading into any type implementing an appropriate `Deserialize`.

### [charlesvdv/nom-bibtex](https://github.com/charlesvdv/nom-bibtex)
The functionality in this crate essentially supercedes [nom-bibtex](https://github.com/charlesvdv/nom-bibtex).
The only feature of `nom-bibtex` that we do not support is the capturing of comments not explicitly contained in a `@comment` entry.

### [typho/bibparser](https://github.com/typho/bibparser)
The functionality in this crate essentially supercedes [bibparser](https://github.com/typho/bibparser).

## Benchmarks
The benchmark code can be find in [`benches/compare.rs`](/benches/compare.rs).
The bibliography file used is [`assets/tugboat.bib`](/assets/tugboat.bib), which is part of the testing data used by biber.
It is a 2.64 MB 73,993-line `.bib` file.

1. `ignore`: Deserialize using `serde::de::IgnoredAny` to parse the file but ignore the contents.
2. `struct`: Deserialize using a struct with entries capturing every field present in `assets/tugboat.bib` (15 fields total), expanding macros and collapsing field values.
3. `borrow`: Deserialize into a fully borrowed Rust type which captures all data in the file but does not expand macros or collapse field values.
4. `biblatex`: Parse using `biblatex::RawBibliography::parse` (most similar to `borrow`).
5. `copy`: Deserialize into an owned Rust type with macro expansion, field value collapsing, and case-insensitive comparison where appropriate.
6. `nom-bibtex`: Parse using `nom-bibtex::Bibtex::parse` (most similar to `copy`).

The benchmarks were performed on an Intel(R) Core(TM) i7-9750H CPU @ 2.60 GHz (2019 MacBook Pro).

| benchmark  | factor | runtime                           | throughput |
|------------|--------|-----------------------------------|------------|
| ignore     | 0.18x  | `[3.3923 ms 3.3987 ms 3.4058 ms]` | 660 MB/s   |
| struct     | 0.67x  | `[8.5496 ms 8.7481 ms 8.9924 ms]` | 300 MB/s   |
| borrow     | 1.0x   | `[12.932 ms 12.962 ms 12.992 ms]` | 200 MB/s   |
| biblatex   | 1.3x   | `[16.184 ms 16.224 ms 16.266 ms]` | 160 MB/s   |
| copy       | 1.7x   | `[21.455 ms 21.690 ms 21.935 ms]` | 120 MB/s   |
| nom-bibtex | 5.5x   | `[71.607 ms 71.912 ms 72.343 ms]` | 40 MB/s    |

The [bibparser](https://github.com/typho/bibparser) crate is not included in this benchmark as it is unable to parse the input file.

## Safety
This crate uses some `unsafe` for string conversions when we can guarantee for other reasons that a string slice is at a valid codepoint.
