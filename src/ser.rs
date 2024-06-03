mod entry;
mod formatter;
mod macros;
mod value;

use std::io;

use serde::ser;

pub use self::formatter::{CompactFormatter, DefaultFormatter, Formatter, ValidatingFormatter};
use self::{
    entry::EntrySerializer,
    macros::{serialize_err, serialize_trait_impl},
};
use crate::error::{Error, Result};

#[inline]
pub fn to_writer<W, T>(writer: W, value: &T) -> Result<()>
where
    W: io::Write,
    T: ?Sized + ser::Serialize,
{
    let mut ser = Serializer::new(writer, DefaultFormatter {});
    value.serialize(&mut ser)
}

/// The main serializer, when you already have a [`std::io::Write`] and a [`Formatter`].
pub struct Serializer<W, F = DefaultFormatter> {
    writer: W,
    formatter: F,
}

impl<W, F> Serializer<W, F>
where
    W: io::Write,
{
    /// Create a new [`Serializer`].
    pub fn new(writer: W, formatter: F) -> Self {
        Self { writer, formatter }
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
        only_seq,
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
        serialize_trait_impl!(BibliographySerializer, $trait, {
            type Ok = ();
            fn $fn<T>(&mut self, value: &T) -> Result<Self::Ok>
            where
                T: ?Sized + serde::Serialize,
            {
                if self.skip_newline {
                    self.skip_newline = false;
                } else {
                    self.ser
                        .formatter
                        .write_entry_separator(&mut self.ser.writer)
                        .map_err(Error::io)?;
                }
                self.skip_newline = value.serialize(EntrySerializer::new(&mut *self.ser))?;
                Ok(())
            }
        });
    };
}

bibliography_serializer_impl!(serialize_element, SerializeSeq);
bibliography_serializer_impl!(serialize_element, SerializeTuple);
bibliography_serializer_impl!(serialize_field, SerializeTupleStruct);

#[cfg(test)]
mod tests {
    use serde::Serialize;
    use std::collections::BTreeMap;

    use super::*;

    #[derive(Serialize)]
    struct Record {
        entry_type: &'static str,
        entry_key: &'static str,
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

        let mut out: Vec<u8> = Vec::new();
        to_writer(&mut out, &bib).unwrap();
        assert_eq!(
            std::str::from_utf8(&out),
            Ok("@article{1,\n  author = {Auth},\n  year = {2022},\n}\n\n@book{2,\n}")
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

        let mut out: Vec<u8> = Vec::new();
        to_writer(&mut out, &bib).unwrap();
        assert_eq!(
            std::str::from_utf8(&out),
            Ok("@article{1,\n  author = {Auth},\n  year = {2022},\n}\n\n@preamble{{preamble}}\n\n@macro{apr = {04}}")
        );
    }

    #[test]
    fn test_entry_opt() {
        let bib = vec![
            EntryOptMacro::Macro(Some(("apr", "04"))),
            EntryOptMacro::Macro(None),
            EntryOptMacro::Comment(""),
        ];

        let mut out: Vec<u8> = Vec::new();
        to_writer(&mut out, &bib).unwrap();
        assert_eq!(
            std::str::from_utf8(&out),
            Ok("@macro{apr = {04}}\n\n@comment{{}}")
        );
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

        let mut out: Vec<u8> = Vec::new();
        to_writer(&mut out, &bib).unwrap();
        assert_eq!(
            std::str::from_utf8(&out),
            Ok("@preamble{a # {txt}}\n\n@preprint{1,\n  author = {First} # sep # {Last},\n}\n\n@preamble{}")
        );
    }
}
