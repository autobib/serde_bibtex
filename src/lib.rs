//! # Serde BibTex
//!
//! <div class="warning">This crate is under active development and the public API may change substantially on every minor version change.
//! The (de)serialization API is relatively stable, but some of the publicly-exposed internal state, particularly concerning error handling, may change or be removed in the future.
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
//!
//! ## Basic Deserialization
//!
//! The most convenient entrypoint is to construct a
//! [`Deserializer`](de/struct.Deserializer.)
//! and use the API provided by
//! [`into_iter_regular_entry`](de/struct.Deserializer.html#method.into_iter_regular_entry).
//! For more complex deserialization use-cases, and a full description of available deserialization
//! features, see the documentation for the [de module](de).
//! ```
//! use serde::Deserialize;
//! use serde_bibtex::de::Deserializer;
//! use std::collections::BTreeMap;
//!
//! #[derive(Debug, PartialEq, Deserialize)]
//! struct Record {
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
//! let mut entry_iter = de.into_iter_regular_entry();
//!
//! let expected_fields = BTreeMap::from([
//!     ("title".into(), "Title".into()),
//!     ("author".into(), "One, Author".into()),
//!     ("year".into(), "2024".into()),
//! ]);
//!
//! assert_eq!(
//!     entry_iter.next(),
//!     Some(Ok(Record {
//!         entry_type: "article".into(),
//!         entry_key: "key".into(),
//!         fields: expected_fields
//!     }))
//! );
//! ```
//!
//! ## Basic Serialization
//! The most convenient entrypoint is use one of the convenience methods, such as [`to_string`] or [`to_writer`], or one of the 'compact' or 'unchecked' variants.
//! Since BibTeX is a relatively rigid output format, your types must be in a relatively rigid
//! format. For examples, and for a detailed description of the conventions used for serialization,
//! see the [ser module](ser).
//! ```
//! use std::collections::BTreeMap;
//! use serde::Serialize;
//! use serde_bibtex::to_string;
//!
//! #[derive(Debug, Serialize)]
//! struct Record {
//!     entry_type: String,
//!     entry_key: String,
//!     fields: BTreeMap<String, String>,
//! }
//!
//! let mut fields = BTreeMap::new();
//! fields.insert("author".to_owned(), "Last, First".to_owned());
//! fields.insert("year".to_owned(), "2023".to_owned());
//!
//! let bibliography = vec![
//!     Record {
//!         entry_type: "article".to_owned(),
//!         entry_key: "FirstLast2023".to_owned(),
//!         fields,
//!     },
//! ];
//!
//! let output = to_string(&bibliography).unwrap();
//!
//! assert_eq!(
//!     output,
//!     "@article{FirstLast2023,\n  author = {Last, First},\n  year = {2023},\n}\n"
//! );
//! ```
//!
//!
//! ## Validation of types
//!
//! If you only wish to check for certain syntax errors independent of serialization and
//! deserialization, see the [validate module](validate).

#![cfg_attr(docsrs, feature(doc_cfg))]

pub mod de;
#[cfg(feature = "entry")]
#[cfg_attr(docsrs, doc(cfg(feature = "entry")))]
pub mod entry;
pub mod error;
pub(crate) mod naming;
pub(crate) mod parse;
pub mod ser;
#[cfg(feature = "syntax")]
#[cfg_attr(docsrs, doc(cfg(feature = "syntax")))]
pub mod syntax;
pub mod token;

use std::io;

use serde::{Deserialize, Serialize};

use crate::{de::Deserializer, ser::Serializer};
pub use crate::{
    error::{Error, Result},
    // parse::token,
    parse::{MacroDictionary, SliceReader, StrReader},
};

/// Deserialize an instance of type `D` from string of BibTeX.
pub fn from_str<'r, D>(s: &'r str) -> Result<D>
where
    D: Deserialize<'r>,
{
    let reader = StrReader::new(s);
    let mut deserializer = Deserializer::new(reader);
    D::deserialize(&mut deserializer)
}

/// Deserialize an instance of type `D` from bytes of BibTeX.
pub fn from_bytes<'r, D>(s: &'r [u8]) -> Result<D>
where
    D: Deserialize<'r>,
{
    let reader = SliceReader::new(s);
    let mut deserializer = Deserializer::new(reader);
    D::deserialize(&mut deserializer)
}

/// Serialize the given data structure as BibTeX into the I/O stream.
#[inline]
pub fn to_writer<W, T>(writer: W, value: &T) -> Result<()>
where
    W: io::Write,
    T: ?Sized + Serialize,
{
    let mut ser = Serializer::new(writer);
    value.serialize(&mut ser)
}

/// Serialize the given data structure as BibTeX into the I/O stream without checking that the
/// output is valid BibTex.
#[inline]
pub fn to_writer_unchecked<W, T>(writer: W, value: &T) -> Result<()>
where
    W: io::Write,
    T: ?Sized + Serialize,
{
    let mut ser = Serializer::unchecked(writer);
    value.serialize(&mut ser)
}

/// Serialize the given data structure as BibTeX into the I/O stream with no extra whitespace.
#[inline]
pub fn to_writer_compact<W, T>(writer: W, value: &T) -> Result<()>
where
    W: io::Write,
    T: ?Sized + Serialize,
{
    let mut ser = Serializer::compact(writer);
    value.serialize(&mut ser)
}

/// Serialize the given data structure as BibTeX into a byte vector.
#[inline]
pub fn to_vec<T>(value: &T) -> Result<Vec<u8>>
where
    T: ?Sized + Serialize,
{
    let mut writer = Vec::with_capacity(128);
    to_writer(&mut writer, value)?;
    Ok(writer)
}

/// Serialize the given data structure as BibTeX into a byte vector without checking that the
/// output is valid BibTeX.
#[inline]
pub fn to_vec_unchecked<T>(value: &T) -> Result<Vec<u8>>
where
    T: ?Sized + Serialize,
{
    let mut writer = Vec::with_capacity(128);
    to_writer_unchecked(&mut writer, value)?;
    Ok(writer)
}

/// Serialize the given data structure as BibTeX into a byte vector with no extra whitespace.
#[inline]
pub fn to_vec_compact<T>(value: &T) -> Result<Vec<u8>>
where
    T: ?Sized + Serialize,
{
    let mut writer = Vec::with_capacity(128);
    to_writer_compact(&mut writer, value)?;
    Ok(writer)
}

/// Serialize the given data structure as BibTeX into a string.
#[inline]
pub fn to_string<T>(value: &T) -> Result<String>
where
    T: ?Sized + Serialize,
{
    let vec = to_vec(value)?;
    let string = unsafe {
        // We do not emit invalid UTF-8.
        String::from_utf8_unchecked(vec)
    };
    Ok(string)
}

/// Serialize the given data structure as BibTeX into a string without checking that the output is
/// valid BibTeX.
#[inline]
pub fn to_string_unchecked<T>(value: &T) -> Result<String>
where
    T: ?Sized + Serialize,
{
    let vec = to_vec_unchecked(value)?;
    let string = unsafe {
        // We do not emit invalid UTF-8.
        String::from_utf8_unchecked(vec)
    };
    Ok(string)
}

/// Serialize the given data structure as BibTeX into a string with no extra whitespace.
#[inline]
pub fn to_string_compact<T>(value: &T) -> Result<String>
where
    T: ?Sized + Serialize,
{
    let vec = to_vec_compact(value)?;
    let string = unsafe {
        // We do not emit invalid UTF-8.
        String::from_utf8_unchecked(vec)
    };
    Ok(string)
}
