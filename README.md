[![Current crates.io release](https://img.shields.io/crates/v/serde_bibtex)](https://crates.io/crates/serde_bibtex)
[![Documentation](https://img.shields.io/badge/docs.rs-serde__bibtex-66c2a5?labelColor=555555&logoColor=white&logo=data:image/svg+xml;base64,PHN2ZyByb2xlPSJpbWciIHhtbG5zPSJodHRwOi8vd3d3LnczLm9yZy8yMDAwL3N2ZyIgdmlld0JveD0iMCAwIDUxMiA1MTIiPjxwYXRoIGZpbGw9IiNmNWY1ZjUiIGQ9Ik00ODguNiAyNTAuMkwzOTIgMjE0VjEwNS41YzAtMTUtOS4zLTI4LjQtMjMuNC0zMy43bC0xMDAtMzcuNWMtOC4xLTMuMS0xNy4xLTMuMS0yNS4zIDBsLTEwMCAzNy41Yy0xNC4xIDUuMy0yMy40IDE4LjctMjMuNCAzMy43VjIxNGwtOTYuNiAzNi4yQzkuMyAyNTUuNSAwIDI2OC45IDAgMjgzLjlWMzk0YzAgMTMuNiA3LjcgMjYuMSAxOS45IDMyLjJsMTAwIDUwYzEwLjEgNS4xIDIyLjEgNS4xIDMyLjIgMGwxMDMuOS01MiAxMDMuOSA1MmMxMC4xIDUuMSAyMi4xIDUuMSAzMi4yIDBsMTAwLTUwYzEyLjItNi4xIDE5LjktMTguNiAxOS45LTMyLjJWMjgzLjljMC0xNS05LjMtMjguNC0yMy40LTMzLjd6TTM1OCAyMTQuOGwtODUgMzEuOXYtNjguMmw4NS0zN3Y3My4zek0xNTQgMTA0LjFsMTAyLTM4LjIgMTAyIDM4LjJ2LjZsLTEwMiA0MS40LTEwMi00MS40di0uNnptODQgMjkxLjFsLTg1IDQyLjV2LTc5LjFsODUtMzguOHY3NS40em0wLTExMmwtMTAyIDQxLjQtMTAyLTQxLjR2LS42bDEwMi0zOC4yIDEwMiAzOC4ydi42em0yNDAgMTEybC04NSA0Mi41di03OS4xbDg1LTM4Ljh2NzUuNHptMC0xMTJsLTEwMiA0MS40LTEwMi00MS40di0uNmwxMDItMzguMiAxMDIgMzguMnYuNnoiPjwvcGF0aD48L3N2Zz4K)](https://docs.rs/serde_bibtex/)

# WARNING
This crate is under active development and the public API may change substantially on every minor version change.
The (de)serialization API is relatively stable, but some of the publicly-exposed internal state may change, particularly concerning the handling of errors.
Until this is stabilized, use at your own risk!

# Serde bibtex
A [Rust](https://www.rust-lang.org/) library providing a [serde](https://serde.rs/) interface for `.bib` file (de)serialization.
The implementation is minimally opinionated and feature-rich for convenient downstream consumption by other libraries or binaries.

For examples and a thorough documentation of features, visit the [docs](https://docs.rs/serde_bibtex/latest/serde_bibtex).

## Deserializer
Here are the main features.
See the [deserializer docs](https://docs.rs/serde_bibtex/latest/serde_bibtex/de/index.html) for more detail.

### Flexible
- Structured: read into Rust types with automatic `@string` macro expansion and other convenience features.
- Unstructured: do not expand macros or collect fields values to preserve the structure of the original bibtex.
- Deserialize from bytes to defer UTF-8 conversion, or even pass-through raw bytes.
- Error-tolerant `Iterator` API that allows skipping malformed entries.

### Explicit and unambiguous syntax
- Aims for compatibility with and tested against an independently implemented [pest grammar](/src/syntax/bibtex.pest).
- Aim for compatibility with [biber](https://github.com/plk/biber) but without some of biber's [undocumented idiosyncracies](https://docs.rs/serde_bibtex/latest/serde_bibtex/syntax/index.html#differences-from-biber) or [unfixable parsing bugs](https://github.com/plk/biber/issues/456).

### Fast
- Low overhead parser implementation (see [benchmarks](#benchmarks)).
- Zero-copy deserialization.
- Selective capturing of contents (see [benchmarks](#benchmarks) for speed differences)


## Serializer
Here are the main features.
See the [serializer docs](https://docs.rs/serde_bibtex/latest/serde_bibtex/ser/index.html) for more detail.

### Flexible
- Flexibly serialize types which are vaguely structured like BibTeX entries.
- Sufficiently general to generate any valid BibTeX bibliography (up to syntactic equivalence), including all entry types such as `@string` macros, and out-putting unexpanded macros.
- Implementable `Formatter` trait which allows total customization of generated BibTeX.

### Convenient defaults
- Default `Formatter` implementations serialize in a standardized format to guarantee unambiguous parsing even by other tools.
- Compact formatter when serializing for consumption by non-humans.

### Robust
- Validate during serialization to guarantee generation of valid BibTeX.

## Comparison with other crates

### [typst/biblatex](https://github.com/typst/biblatex)
In short, `serde_bibtex` is less opinionated and more flexible than `biblatex`, whereas `biblatex` provides opinionated interpretation of the fields of a bibliography.

Use `serde_bibtex` if you want:

- to deserialize into your own types
- to parse from raw bytes to handle non-UTF8 ASCII-compatible encodings
- a faithful representation of a BibTeX file
- custom handling and expansion of macros (`@string`)
- finer control over the performance-convenience trade-off

Use `biblatex` if you want:

- a simple, opinionated interface for querying a bibliography
- interpretation of field contents (for example, parsing of `$math` in field values, or parsing of lists of author or editor names).

### [charlesvdv/nom-bibtex](https://github.com/charlesvdv/nom-bibtex)
The functionality in this crate essentially supercedes [nom-bibtex](https://github.com/charlesvdv/nom-bibtex).
The only feature of `nom-bibtex` that we do not support is the capturing of TeX-style comments (like `% comment`).

### [typho/bibparser](https://github.com/typho/bibparser)
The functionality in this crate essentially supercedes [bibparser](https://github.com/typho/bibparser).

## Benchmarks
The benchmark code can be found in [`benches/compare.rs`](/benches/compare.rs).
The bibliography file used is [`assets/tugboat.bib`](/assets/tugboat.bib), which is part of the testing data used by biber.
It is a 2.64 MB 73,993-line `.bib` file.

1. `ignore`: Deserialize using `serde::de::IgnoredAny` to parse the file but ignore the contents.
2. `struct`: Deserialize using a struct with entries capturing every field present in `assets/tugboat.bib` (15 fields total), expanding macros and collapsing field values.
3. `borrow`: Deserialize into a fully borrowed Rust type which captures all data in the file but does not expand macros or collapse field values.
4. `biblatex`: Parse using `biblatex::RawBibliography::parse` (like `borrow`, but with less capturing and more allocation).
5. `copy`: Deserialize into an owned Rust type with macro expansion, field value collapsing, and case-insensitive comparison where appropriate.
6. `nom-bibtex`: Parse using `nom-bibtex::Bibtex::parse` (like `copy`, but with extra TeX-style comment capturing).
7. `bibliography`: Parse using `biblatex::Bibliography::parse` (like `copy`, but with extra parsing of field contents into raw / verbatim / math and evaluation `xdata` and `crossref` fields).

The benchmarks were performed on an Apple M4 Pro 48 GB (2024 MacBook Pro).
The speedup factor is relative to `biblatex`.

| benchmark    | factor | runtime                           | throughput |
|--------------|--------|-----------------------------------|------------|
| ignore       | 3.6x   | `[1.4657 ms 1.4688 ms 1.4722 ms]` | 1797 MB/s  |
| struct       | 1.5x   | `[3.6063 ms 3.6128 ms 3.6208 ms]` | 731  MB/s  |
| borrow       | 1.2x   | `[4.3324 ms 4.3433 ms 4.3546 ms]` | 608  MB/s  |
| biblatex     | 1.0x   | `[5.2626 ms 5.2870 ms 5.3114 ms]` | 499  MB/s  |
| copy         | 0.75x  | `[6.9917 ms 7.0027 ms 7.0141 ms]` | 377  MB/s  |
| nom-bibtex   | 0.17x  | `[31.422 ms 31.474 ms 31.532 ms]` | 84   MB/s  |
| bibliography | 0.17x  | `[31.599 ms 31.653 ms 31.708 ms]` | 83   MB/s  |

The [bibparser](https://github.com/typho/bibparser) crate is not included in this benchmark as it is unable to parse the input file.

## Safety
This crate uses some `unsafe` for string conversions when we can guarantee for other reasons that a string slice is at a valid codepoint.
