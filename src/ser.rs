//! # Serializer implementation
//! This section contains a full description, with examples, of the [`Serializer`] interface
//! provided by this crate. For more context conerning the output syntax, visit the
//! [syntax](crate::syntax) module.
//!
//! Jump to:
//! - [Serializing a bibliography](#serializing-a-bibliography)
//!   - [Basic serialization](#basic-serialization)
//!   - [Checking output validity](#checking-output-validity)
//!   - [Serializing values](#serializing-values)
//! - [Serialization variants](#serialization-variants)
//!
//! ## Serializing a bibliography
//!
//! ### Basic serialization
//! A bibliography is a sequence of entries, which fall into four categories:
//!
//! 1. Regular entries
//! 2. Macros
//! 3. Comments
//! 4. Preamble
//!
//! A basic type which can be serialized by this crate is the following.
//! ```
//! use std::collections::BTreeMap;
//! use serde::Serialize;
//! use serde_bibtex::to_string;
//!
//! #[derive(Debug, Serialize)]
//! enum Entry {
//!     Macro(String, String),
//!     Preamble(String),
//!     Comment(String),
//!     Regular(Record),
//! }
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
//!     Entry::Comment("A comment".to_owned()),
//!     Entry::Regular(Record {
//!         entry_type: "article".to_owned(),
//!         entry_key: "FirstLast2023".to_owned(),
//!         fields,
//!     }),
//! ];
//!
//! let output = to_string(&bibliography).unwrap();
//! // @comment{A comment}
//! //
//! // @article{FirstLast2023,
//! //   author = {Last, First},
//! //   year = {2023},
//! // }
//! # assert_eq!(
//! #     output,
//! #     "@comment{A comment}\n\n@article{FirstLast2023,\n  author = {Last, First},\n  year = {2023},\n}\n"
//! # );
//! ```
//! There are a few supported variants.
//!
//! 1. It is also possible to inline the record type directly as a tuple. In this case, the first
//!    field is the entry type, the second field is the entry key, and the third field is the entry
//!    fields.
//! 2. The `fields` can also be serialized directly from a vector of `(key, value)` pairs.
//! 3. For compatibility with the deserializer implementation, the `Macro` variant can be optional.
//!    If the value is `None`, the corresponding macro entry will be skipped.
//! 4. Variants with valid names can be omitted and the corresponding entry will not be written.
//!    To omit other names, use the serde [`skip_serializing_field`](https://serde.rs/attr-skip-serializing.html)
//!    macro attribute.
//! 5. Of course, you can simply not include a variant in the enum.
//! ```
//! # use serde::Serialize;
//! #[derive(Debug, Serialize)]
//! enum Entry {
//!     Macro(Option<(String, String)>),
//!     Comment,
//!     Regular(String, String, Vec<(String, String)>),
//! }
//! ```
//! If you only wish to serialize regular entries, the `Record` struct can be passed
//! directly in place of the `Entry` enum. The tuple format is also supported.
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
//! // @article{FirstLast2023,
//! //   author = {Last, First},
//! //   year = {2023},
//! // }
//! # assert_eq!(
//! #     output,
//! #     "@article{FirstLast2023,\n  author = {Last, First},\n  year = {2023},\n}\n"
//! # );
//! ```
//! This can also be done with a vector of tuples, and also with an arbitrary sequence type for the
//! fields.
//! ```
//! use serde_bibtex::to_string;
//! let bib = vec![("article", "key", [("author", "Author"), ("year", "2023")])];
//! let output = to_string(&bib).unwrap();
//! // @article{key,
//! //   author = {Author},
//! //   year = {2023},
//! // }
//! # assert!(to_string(&bib).is_ok());
//! ```
//!
//! ### Checking output validity
//! Note that, by default, the validity of the emitted text is checked for validity. For instance,
//! passing an entry key containing invalid symbols results in an error:
//! ```
//! use serde_bibtex::to_string;
//! let bib = vec![("article", ",", [("author", "Author"), ("year", "2023")])];
//! assert!(to_string(&bib).is_err());
//! ```
//! If you explicitly do not want to validate output, you can use (for instance) the [`to_string_unchecked`](crate::to_string_unchecked) function. Note that this could result in invalid BibTeX. You can check validity of the types using functions in the [token](crate::token) module directly.
//! ```
//! use serde_bibtex::to_string_unchecked;
//! // multiple syntax errors: whitespace in entry type, invalid characters in entry key, empty
//! // field key
//! let bib = vec![("art icle", ",", [("", "Author")])];
//! let output = to_string_unchecked(&bib).unwrap();
//! assert_eq!(output, "@art icle{,,\n   = {Author},\n}\n");
//! ```
//!
//! ### Serializing values
//! To serialize unexpanded variables directly into the output, expanded value serialization is
//! supported.
//! A value is a list of `Token`, where a `Token` is an enum with two special variant names: `Text` and
//! `Variable`.
//! Values can appear anywhere that they are supported in the syntax: in field values, in
//! the preamble, and in macro values.
//! ```
//! use std::collections::BTreeMap;
//! use serde::Serialize;
//! use serde_bibtex::to_string;
//!
//! #[derive(Debug, Serialize)]
//! enum Token {
//!     Text(String),
//!     Variable(String),
//! }
//!
//! type Value = Vec<Token>;
//!
//! #[derive(Debug, Serialize)]
//! struct Record {
//!     entry_type: String,
//!     entry_key: String,
//!     fields: BTreeMap<String, Value>,
//! }
//!
//! #[derive(Debug, Serialize)]
//! enum Entry {
//!     Macro(String, Value),
//!     Preamble(Value),
//!     Regular(Record),
//! }
//!
//! let mut fields = BTreeMap::new();
//! fields.insert(
//!     "author".to_owned(),
//!     vec![
//!         Token::Text("Author One".to_owned()),
//!         Token::Variable("and".to_owned()),
//!         Token::Text("Author Two".to_owned())
//!     ],
//! );
//! fields.insert("month".to_owned(), vec![Token::Variable("apr".to_owned())]);
//!
//! let bibliography = vec![
//!     Entry::Macro("and".to_owned(), vec![Token::Text(" and ".to_owned())]),
//!     Entry::Regular(Record {
//!         entry_type: "article".to_owned(),
//!         entry_key: "OneTwo".to_owned(),
//!         fields,
//!     }),
//! ];
//! let output = to_string(&bibliography).unwrap();
//! // @string{and = { and }}
//! //
//! // @article{OneTwo,
//! //   author = {Author One} # and # {Author Two},
//! //   month = apr,
//! // }
//! # assert_eq!(
//! #     output,
//! #     "@string{and = { and }}\n\n@article{OneTwo,\n  author = {Author One} # and # {Author Two},\n  month = apr,\n}\n"
//! # );
//! ```
//!
//! ## Serialization variants
//! You can configure the [`Serializer`] with a custom formatter with the
//! [`Serializer::new_with_formatter`] method.
//!
//! There are two default formatters. Note that these formatters do not perform output
//! validation.
//!
//! - [`PrettyFormatter`]: Print the bibliograph with an appropriate amount of whitespace.
//! - [`CompactFormatter`]: Similar to [`PrettyFormatter`], but do not write any excess
//!   whitespace.
//!
//! In order to also verify that the output is valid, the wrapper struct [`ValidatingFormatter`]
//! adds a validation step to any type which implements [`Formatter`]. If you wish to check
//! validity in your own code, see the [token](crate::token) module.
//!
//! There are convenience entry points for built-in formatters; see for instance the
//! [`to_string`](crate::to_string) method, with variants [`to_string_unchecked`](crate::to_string)
//! and [`to_string_compact`](crate::to_string_compact)
//! You can also provide your own implementation of [`Formatter`] for even greater customization of the output.
mod entry;
mod formatter;
mod macros;
mod value;

use std::io;

use serde::ser;

pub use self::formatter::{CompactFormatter, Formatter, PrettyFormatter, ValidatingFormatter};
use self::{entry::EntrySerializer, formatter::FormatBuffer, macros::serialize_err};
use crate::error::{Error, Result};

/// The main serializer, when you already have a [`std::io::Write`] and a [`Formatter`].
pub struct Serializer<W, F = PrettyFormatter> {
    writer: W,
    buffer: FormatBuffer<F>,
}

impl<W, F> Serializer<W, F> {
    /// Create a new [`Serializer`] with the provided writer and [`Formatter`].
    pub fn new_with_formatter(writer: W, formatter: F) -> Self {
        Self {
            writer,
            buffer: FormatBuffer::new(formatter),
        }
    }

    /// Recover the interval writer.
    pub fn into_inner(self) -> W {
        let Self { writer, .. } = self;
        writer
    }
}

impl<W> Serializer<W, ValidatingFormatter<PrettyFormatter>>
where
    W: io::Write,
{
    /// Create a new [`Serializer`] with pretty printing and output validation.
    pub fn new(writer: W) -> Self {
        Self::new_with_formatter(writer, ValidatingFormatter::new(PrettyFormatter {}))
    }
}

impl<W> Serializer<W, PrettyFormatter>
where
    W: io::Write,
{
    /// Create a new [`Serializer`] with a formatter which does not perform output checking.
    pub fn unchecked(writer: W) -> Self {
        Self::new_with_formatter(writer, PrettyFormatter {})
    }
}

impl<W> Serializer<W, ValidatingFormatter<CompactFormatter>>
where
    W: io::Write,
{
    /// Create a new [`Serializer`] with a formatter with no extra whitespace.
    pub fn compact(writer: W) -> Self {
        Self::new_with_formatter(writer, ValidatingFormatter::new(CompactFormatter {}))
    }
}

/// The compound serializer type used for stateful serialization of a bibliograhy.
pub struct BibliographySerializer<'a, W, F> {
    ser: &'a mut Serializer<W, F>,
    skip_newline: bool,
}

impl<'a, W, F> BibliographySerializer<'a, W, F> {
    /// Create a new [`BibliographySerializer`].
    pub fn new(ser: &'a mut Serializer<W, F>) -> Self {
        Self {
            ser,
            skip_newline: true,
        }
    }
}

impl<'a, W, F> ser::Serializer for &'a mut Serializer<W, F>
where
    W: std::io::Write,
    F: Formatter,
{
    type Ok = ();

    type SerializeSeq = BibliographySerializer<'a, W, F>;
    type SerializeTuple = BibliographySerializer<'a, W, F>;
    type SerializeTupleStruct = BibliographySerializer<'a, W, F>;

    serialize_err!(
        "bibliography",
        i8,
        i16,
        i32,
        i64,
        u8,
        u16,
        u32,
        u64,
        f32,
        f64,
        char,
        str,
        bytes,
        bool,
        tuple_variant,
        map,
        option,
        struct,
        struct_variant,
        unit,
        unit_struct,
        unit_variant,
        newtype_variant
    );

    fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq> {
        Ok(Self::SerializeSeq::new(self))
    }

    fn serialize_tuple(
        self,
        _len: usize,
    ) -> std::result::Result<Self::SerializeTuple, Self::Error> {
        Ok(Self::SerializeSeq::new(self))
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> std::result::Result<Self::SerializeTupleStruct, Self::Error> {
        Ok(Self::SerializeSeq::new(self))
    }
}

macro_rules! bibliography_serializer_impl {
    ($fn:ident, $trait:ident) => {
        impl<'a, W, F> ser::$trait for BibliographySerializer<'a, W, F>
        where
            W: io::Write,
            F: Formatter,
        {
            type Ok = ();
            type Error = Error;

            fn $fn<T>(&mut self, value: &T) -> Result<Self::Ok>
            where
                T: ?Sized + serde::Serialize,
            {
                if self.skip_newline {
                    self.skip_newline = false;
                } else {
                    self.ser
                        .buffer
                        .write_entry_separator(&mut self.ser.writer)?;
                }
                self.skip_newline = value.serialize(EntrySerializer::new(&mut *self.ser))?;
                self.ser.buffer.write(&mut self.ser.writer)?;
                Ok(())
            }

            #[inline]
            fn end(self) -> std::result::Result<Self::Ok, Self::Error> {
                self.ser
                    .buffer
                    .write_bibliography_end(&mut self.ser.writer)?;
                Ok(())
            }
        }
    };
}

bibliography_serializer_impl!(serialize_element, SerializeSeq);
bibliography_serializer_impl!(serialize_element, SerializeTuple);
bibliography_serializer_impl!(serialize_field, SerializeTupleStruct);

#[cfg(test)]
mod tests {
    use serde::Serialize;
    use std::collections::BTreeMap;

    use crate::{to_string, to_string_compact};

    #[derive(Serialize)]
    struct Record {
        entry_key: &'static str,
        entry_type: &'static str,
        fields: Vec<(&'static str, &'static str)>,
    }

    #[derive(Serialize)]
    enum Entry {
        Regular(Record),
        Macro(&'static str, &'static str),
        Comment,
        Preamble(&'static str),
    }

    #[derive(Serialize)]
    enum EntryOptMacro {
        Macro(Option<(&'static str, &'static str)>),
        Comment(&'static str),
    }

    #[derive(Serialize)]
    enum Value {
        Text(&'static str),
        Variable(&'static str),
    }

    #[derive(Serialize)]
    enum EntryFullValue {
        Preamble(Vec<Value>),
        Regular(
            &'static str,
            &'static str,
            BTreeMap<&'static str, Vec<Value>>,
        ),
    }

    #[test]
    fn test_struct() {
        let bib = vec![
            Record {
                entry_type: "article",
                entry_key: "1",
                fields: vec![("author", "Auth"), ("year", "2022")],
            },
            Record {
                entry_type: "book",
                entry_key: "2",
                fields: Vec::new(),
            },
        ];

        let out = to_string(&bib).unwrap();
        assert_eq!(
            out,
            "@article{1,\n  author = {Auth},\n  year = {2022},\n}\n\n@book{2,\n}\n"
        );
    }

    #[test]
    fn test_enum_skip() {
        let bib = vec![
            Entry::Regular(Record {
                entry_type: "article",
                entry_key: "1",
                fields: vec![("author", "Auth"), ("year", "2022")],
            }),
            Entry::Comment,
            Entry::Comment,
            Entry::Preamble("preamble"),
            Entry::Macro("apr", "04"),
        ];

        let out = to_string(&bib).unwrap();
        assert_eq!(
            out,
            "@article{1,\n  author = {Auth},\n  year = {2022},\n}\n\n@preamble{{preamble}}\n\n@string{apr = {04}}\n"
        );
    }

    #[test]
    fn test_entry_opt() {
        let bib = vec![
            EntryOptMacro::Macro(Some(("apr", "04"))),
            EntryOptMacro::Macro(None),
            EntryOptMacro::Comment(""),
        ];

        let out = to_string(&bib).unwrap();
        assert_eq!(out, "@string{apr = {04}}\n\n@comment{}\n");
    }

    #[test]
    fn test_tuple() {
        let bib = vec![("article", "key", [("author", "Author"), ("year", "2023")])];

        let out = to_string(&bib).unwrap();
        assert_eq!(
            out,
            "@article{key,\n  author = {Author},\n  year = {2023},\n}\n"
        );
    }

    #[test]
    fn test_compact() {
        let bib = vec![
            ("article", "key", [("author", "Author"), ("year", "2023")]),
            ("book", "key2", [("a", "A"), ("b", "B")]),
        ];

        let out = to_string_compact(&bib).unwrap();
        assert_eq!(
            out,
            "@article{key,author={Author},year={2023}}@book{key2,a={A},b={B}}"
        );

        let bib: Vec<(&str, &str, [(&str, &str); 0])> = vec![("article", "key", [])];

        let out = to_string_compact(&bib).unwrap();
        assert_eq!(out, "@article{key}");
    }

    #[test]
    fn test_checking() {
        let bib = vec![("article", "", [("author", "Author"), ("year", "2023")])];
        assert!(to_string(&bib).is_err());

        let bib = vec![("article", ",,", [("author", "Author"), ("year", "2023")])];
        assert!(to_string(&bib).is_err());

        let bib = vec![("article", "1", [("author", ""), ("year", "")])];
        assert!(to_string(&bib).is_ok());

        let bib = vec![("article", "1", [("", "val")])];
        assert!(to_string(&bib).is_err());

        let bib = vec![("", "1", [("key", "val")])];
        assert!(to_string(&bib).is_err());
    }

    #[test]
    fn test_expanded_value() {
        let mut fields = BTreeMap::new();
        fields.insert(
            "author",
            vec![
                Value::Text("First"),
                Value::Variable("sep"),
                Value::Text("Last"),
            ],
        );

        let bib = vec![
            EntryFullValue::Preamble(vec![Value::Variable("a"), Value::Text("txt")]),
            EntryFullValue::Regular("preprint", "1", fields),
            EntryFullValue::Preamble(Vec::new()),
        ];

        let out = to_string(&bib).unwrap();
        assert_eq!(
            out,
            "@preamble{a # {txt}}\n\n@preprint{1,\n  author = {First} # sep # {Last},\n}\n\n@preamble{}\n"
        );
    }
}
