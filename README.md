# WARNING
This crate is under active development and the public API will change frequently and without warning.
Until this is stabilized, use at your own risk!

# Serde bibtex
A [Rust](https://www.rust-lang.org/) library providing a [serde](https://serde.rs/) interface for `.bib` file (de)serialization.
The implementation is minimally opinionated and feature-rich for convenient downstream consumption by other libraries or binaries.

## Deserializer

### Flexible deserialization:
  - Structured: read into Rust types with automatic `@string` macro expansion and other convenience features.
  - Unstructured: do not expand macros or collect fields values to preserve the structure of the original bibtex.
  - Deserialize from bytes to defer UTF-8 conversion, or even pass-through raw bytes.
  - Error-tolerant `Iterator` API that allows skipping malformed entries.
### Well-defined and explicit syntax:
  - Tested against an independently implemented [pest grammar](/src/syntax/bibtex.pest).
  - Aim for compatibility with [biber](https://github.com/plk/biber) but without some of biber's [undocumented idiosyncracies](https://docs.rs/serde_bibtex/latest/serde_bibtex/syntax/index.html#differences-from-biber) or [unfixable parsing bugs](https://github.com/plk/biber/issues/456).
### Fast
  - Low overhead manual parser implementation (see benchmarks).
  - Optional zero-copy or owned deserialization.


## Serializer
TODO: not yet implemented


## Comparison with other crates
### [biblatex](https://github.com/typst/biblatex)
We do not attempt to interpret the contents of the entries in the `.bib` file and instead defer interpretation for downstream consumption.
On the other hand, [biblatex](https://github.com/typst/biblatex) is intended to support [typst](https://github.com/typst/typst), which requires interpreting the contents of the fields (for example, parsing of `$math$` in field values).
In this sense, we might consider our implementation closer to the `biblatex::RawBibliography` entrypoint, but with the substantial extra flexibility of reading into any type implementing an appropriate `Deserialize`.

### [nom-bibtex](https://github.com/charlesvdv/nom-bibtex)
The functionality in this crate essentially supercedes [nom-bibtex](https://github.com/charlesvdv/nom-bibtex).
The only feature of `nom-bibtex` that we do not support is the capturing of comments not explicitly contained in a `@comment` entry.

### [bibparser](https://github.com/typho/bibparser)
The functionality in this crate essentially supercedes [bibparser](https://github.com/typho/bibparser).

## Benchmarks
The benchmark code can be find in [`benches/compare.rs`](/benches/compare.rs).
The bibliography file used is [`assets/tugboat.bib`](/assets/tugboat.bib), which is part of the testing data used by biber.
It is a 2.6 MB 73,993-line `.bib` file.

1. `ignored`: Deserialize using `serde::de::IgnoredAny` to parse the file but ignore the contents.
2. `borrowed`: Deserialize into a fully borrowed struct which captures all data in the file but do not expand macros automatically or collapse field values.
3. `biblatex`: Parse using `biblatex::RawBibliography::parse` (most similar to `borrowed`).
4. `owned`: Parse into a fully owned Rust type with macro expansion and field value collapsing.
5. `nom-bibtex`: Parse using `nom-bibtex::Bibtex::parse` (most similar to `owned`).

| benchmark  | factor | runtime                         | 
|------------|--------|---------------------------------|
| ignored    | 1x     | [3.3923 ms 3.3987 ms 3.4058 ms] |
| borrowed   | 3.8x   | [12.932 ms 12.962 ms 12.992 ms] |
| biblatex   | 4.8x   | [16.184 ms 16.224 ms 16.266 ms] |
| owned      | 6.4x   | [21.455 ms 21.690 ms 21.935 ms] |
| nom-bibtex | 21x    | [71.607 ms 71.912 ms 72.343 ms] |

## Safety
This crate uses some `unsafe` for string conversions when we can guarantee for other reasons that a string slice is at a valid codepoint.
