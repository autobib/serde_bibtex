//! # Serde BibTex
//!
//! <div class="warning">This crate is under active development and the public API will change
//! frequently and without warning. Until this is stabilized, use at your own risk!</div>
//!
//! The `.bib` format is a common format for storing bibliographic data originally popularized by
//! the [BibTex](https://en.wikipedia.org/wiki/BibTeX) bibliography management software.
//! ```bib
//! @article{key,
//!   title = {Title},
//!   author = {One, Author},
//!   year = 2024,
//! }
//! ```
//! This module provides a [serde](https://docs.rs/serde/latest/serde/) interface for deserializing
//! `.bib` files into strongly typed data structures.
//!
//! ## Basic Example
//! ```
//! use serde::Deserialize;
//! use serde_bibtex::StrReader;
//! use std::collections::HashMap;
//!
//! #[derive(Debug, Deserialize, PartialEq)]
//! struct Entry {
//!     entry_type: String,
//!     citation_key: String,
//!     fields: HashMap<String, String>
//! }
//!
//! let input = r#"
//!     @article{key,
//!       title = {Title},
//!       author = {One, Author},
//!       year = 2024,
//!     }
//! "#;
//!
//! let de = StrReader::new(input).deserialize();
//!
//! let mut entry_iter = de.into_iter_entry();
//!
//! let mut expected_fields = HashMap::new();
//! expected_fields.insert("title".into(), "Title".into());
//! expected_fields.insert("author".into(), "One, Author".into());
//! expected_fields.insert("year".into(), "2024".into());
//!
//! assert_eq!(
//!     entry_iter.next(),
//!     Some(Ok(Entry {
//!         entry_type: "article".into(),
//!         citation_key: "key".into(),
//!         fields: expected_fields
//!     }))
//! );
//! ```
pub mod de;
pub mod error;

pub(crate) mod naming;

pub(crate) mod parse;
pub use parse::SliceReader;
pub use parse::StrReader;

pub use de::{from_bytes, from_str, Deserializer};
pub use error::Error;
