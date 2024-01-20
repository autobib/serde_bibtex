# WARNING
This crate is under active development and the public API will change frequently and without warning.
Until this is stabilized, use at your own risk!

# Serde bibtex
A [Rust](https://www.rust-lang.org/) library providing a [serde](https://serde.rs/) interface for `.bib` file (de)serialization.
The implementation is minimally opinionated and feature-rich for convenient downstream consumption by other libraries or binaries.

## Deserializer

1. Flexible deserialization:
  - Structured: read into rust structures with automatic `@string` macro expansion.
  - Unstructured: preserve the structure of the original bibtex.
  - Deserialize from bytes to defer UTF-8 conversion, or simply pass-through raw bytes.
  - Error-tolerant `Iterator` API that allows skipping malformed entries.
2. Well-defined and explicit syntax:
  - Tested against an independently implemented [pest grammar](/src/syntax/bibtex.pest).
  - Aim for compatibility with [biber](https://github.com/plk/biber) but without some of biber's [undocumented idiosyncracies](https://docs.rs/serde_bibtex/latest/serde_bibtex/syntax/index.html#differences-from-biber) or [unfixable parsing bugs](https://github.com/plk/biber/issues/456).
3. Fast
  - Low overhead manual parser implementation (see benchmarks).
  - Optional zero-copy or owned deserialization.


## Serializer
TODO: not yet implemented


## Comparison with other crates
### [biblatex](https://github.com/typst/biblatex)
We do not attempt to interpret the contents of the entries in the `.bib` file and instead defer interpretation for downstream consumption.
On the other hand, [biblatex](https://github.com/typst/biblatex) is intended to support [typst](https://github.com/typst/typst), which requires interpreting the contents of the fields (for example, parsing of `$math$` in field values).
In this sense, we might consider our implementation closer to the `biblatex::RawBibliography` entrypoint, but with the substantial extra flexibility of reading into any structured data implementing `Deserialize`, rather than into the specific `RawBibliography` struct.

### [nom-bibtex](https://github.com/charlesvdv/nom-bibtex)
The functionality in this crate essentially supercedes the implementation in [nom-bibtex](https://github.com/charlesvdv/nom-bibtex).
The only feature of `nom-bibtex` that we do not support is the capturing of comments not explicitly delimited by an `@comment` entry.

### [bibparser](https://github.com/typho/bibparser)
The functionality in this crate essentially supercedes the implementation in [bibparser](https://github.com/typho/bibparser).

## Benchmarks
The benchmark code can be find in [`benches/compare.rs`](/benches/compare.rs).

## Safety
This crate uses some `unsafe` for string conversions when we can guarantee for other reasons that the slice is at a valid codepoint.
