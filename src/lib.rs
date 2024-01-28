//! # Serde BibTex
//!
//! <div class="warning">This crate is under active development and the public API may change substantially on every minor version change.
//! The deserialization API is relatively stable, but serialization is not yet implemented, and some of the publicly-exposed internal state may change or removed in the future.
//! Until this is stabilized, use at your own risk!</div>
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
//! `.bib` files into strongly typed data structures. The interface is intentionally flexible to
//! support a variety of use-cases in a first-class manner.
//!
//! Unfortunately, `.bib` files do not have a well-defined syntax and while there are generally
//! agreed-upon conventions for syntax, different programs will treat your input in subtly
//! different ways. Visit the [syntax module](syntax) for more information as well as an explicit
//! [pest](https://docs.rs/pest/latest/pest/) grammar for the file format accepted by this crate.
//!
//! ## Basic Example
//! The most convenient entrypoint is to construct a
//! [`Deserializer`](de/struct.Deserializer.)
//! and use the API provided by
//! [`into_iter_entry`](de/struct.Deserializer.html#method.into_iter_entry).
//! For more complex deserialization use-cases, and a full description of available deserialization
//! features, see the documentation for the [de module](de).
//! ```
//! use serde::Deserialize;
//! use serde_bibtex::de::Deserializer;
//! use std::collections::BTreeMap;
//!
//! #[derive(Debug, PartialEq, Deserialize)]
//! struct Contents {
//!     entry_type: String,
//!     entry_key: String,
//!     fields: BTreeMap<String, String>
//! }
//!
//! let input = r#"
//!     @string{t = {Title}}
//!     @article{key,
//!       title = t,
//!       author = {One, Author},
//!       year = 2024,
//!     }
//! "#;
//!
//! let de = Deserializer::from_str(input);
//! let mut entry_iter = de.into_iter_entry();
//!
//! let expected_fields = BTreeMap::from([
//!     ("title".into(), "Title".into()),
//!     ("author".into(), "One, Author".into()),
//!     ("year".into(), "2024".into()),
//! ]);
//!
//! assert_eq!(
//!     entry_iter.next(),
//!     Some(Ok(Contents {
//!         entry_type: "article".into(),
//!         entry_key: "key".into(),
//!         fields: expected_fields
//!     }))
//! );
//! ```
pub mod de;
pub mod error;
pub mod ser;

#[cfg(feature = "entry")]
pub mod entry;

#[cfg(feature = "syntax")]
pub mod syntax;

pub(crate) mod naming;

pub(crate) mod parse;
pub use parse::token;
pub use parse::{MacroDictionary, SliceReader, StrReader};

pub use de::{from_bytes, from_str};
pub use error::Error;
