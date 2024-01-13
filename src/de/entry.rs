use serde::de::{self, value::BorrowedStrDeserializer, DeserializeSeed, MapAccess, SeqAccess};
use serde::forward_to_deserialize_any;

use crate::error::Error;
use crate::naming::{CITATION_KEY_NAME, ENTRY_TYPE_NAME, FIELDS_NAME};
use crate::value::Identifier;

use super::value::{IdentifierDeserializer, KeyValueDeserializer, ValueDeserializer};
use super::BibtexDeserializer;
use crate::parse::BibtexReader;

pub struct RegularEntryDeserializer<'a, 'r, R>
where
    R: BibtexReader<'r>,
{
    de: &'a mut BibtexDeserializer<'r, R>,
    entry_type: Identifier<'r>,
}

impl<'a, 'r, R> RegularEntryDeserializer<'a, 'r, R>
where
    R: BibtexReader<'r>,
{
    pub fn new(de: &'a mut BibtexDeserializer<'r, R>, entry_type: Identifier<'r>) -> Self {
        Self { de, entry_type }
    }
}

impl<'a, 'de: 'a, R> de::Deserializer<'de> for RegularEntryDeserializer<'a, 'de, R>
where
    R: BibtexReader<'de>,
{
    type Error = Error;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_map(EntryAccess::new(&mut *self.de, self.entry_type))
    }

    fn deserialize_seq<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        Err(Error::Message(
            "Entry can only be deserialized into a tuple of length 3".to_string(),
        ))
    }

    fn deserialize_tuple<V>(self, len: usize, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        if len == 3 {
            visitor.visit_seq(EntryAccess::new(&mut *self.de, self.entry_type))
        } else {
            Err(Error::Message(
                "Entry can only be deserialized into a tuple of length 3".to_string(),
            ))
        }
    }

    #[inline]
    fn deserialize_tuple_struct<V>(
        self,
        _name: &'static str,
        len: usize,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        self.deserialize_tuple(len, visitor)
    }

    fn deserialize_ignored_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        self.de.reader.ignore_entry()?;
        visitor.visit_unit()
    }

    #[inline]
    fn deserialize_unit<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        self.deserialize_ignored_any(visitor)
    }

    #[inline]
    fn deserialize_unit_struct<V>(
        self,
        _name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        self.deserialize_ignored_any(visitor)
    }

    forward_to_deserialize_any!(
        bool i8 i16 i32 i64 u8 u16 u32 u64 f32 f64 char str
        string bytes byte_buf option newtype_struct
        map struct enum identifier);
}

#[derive(Debug, Copy, Clone)]
enum EntryPosition {
    EntryType,
    CitationKey,
    Fields,
    EndOfEntry,
}

/// Deserialize an Entry.
///
/// This deserializes the contents from `>` to `<`
/// ```bib
/// @article>{key,
///   title = {Title},
/// }<
/// ```
/// We assume that `article` is passed as the `entry_type` argument. The reason for this is that
/// when we determine which Entry variant to deserialize, we need to parse the `entry_type` and add
/// special cases to handle `@string`, `@preamble`, and `@comment`.
struct EntryAccess<'a, 'r, R>
where
    R: BibtexReader<'r>,
{
    /// The top-level deserializer holding a reader.
    de: &'a mut BibtexDeserializer<'r, R>,
    /// The previously parsed entry type
    entry_type: Identifier<'r>,
    /// The current position inside the Entry
    pos: EntryPosition,
    /// What closing bracket to expect.
    closing_bracket: char,
}

impl<'a, 'r, R> EntryAccess<'a, 'r, R>
where
    R: BibtexReader<'r>,
{
    fn new(de: &'a mut BibtexDeserializer<'r, R>, entry_type: Identifier<'r>) -> Self {
        Self {
            de,
            entry_type,
            pos: EntryPosition::EndOfEntry,
            closing_bracket: '}',
        }
    }

    fn step_position(&mut self) {
        self.pos = match self.pos {
            EntryPosition::EntryType => EntryPosition::CitationKey,
            EntryPosition::CitationKey => EntryPosition::Fields,
            EntryPosition::Fields => EntryPosition::EndOfEntry,
            EntryPosition::EndOfEntry => EntryPosition::EntryType,
        };
    }
}

impl<'a, 'de: 'a, R> MapAccess<'de> for EntryAccess<'a, 'de, R>
where
    R: BibtexReader<'de>,
{
    type Error = Error;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>, Self::Error>
    where
        K: DeserializeSeed<'de>,
    {
        self.step_position();
        match self.pos {
            EntryPosition::EntryType => seed
                .deserialize(BorrowedStrDeserializer::new(ENTRY_TYPE_NAME))
                .map(Some),
            EntryPosition::CitationKey => seed
                .deserialize(BorrowedStrDeserializer::new(CITATION_KEY_NAME))
                .map(Some),
            EntryPosition::Fields => seed
                .deserialize(BorrowedStrDeserializer::new(FIELDS_NAME))
                .map(Some),
            EntryPosition::EndOfEntry => Ok(None),
        }
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value, Self::Error>
    where
        V: DeserializeSeed<'de>,
    {
        match self.pos {
            EntryPosition::EntryType => {
                seed.deserialize(IdentifierDeserializer::new(self.entry_type))
            }
            EntryPosition::CitationKey => {
                self.closing_bracket = self.de.reader.take_initial()?;
                seed.deserialize(IdentifierDeserializer::new_from_str(
                    self.de.reader.take_citation_key()?,
                ))
            }
            EntryPosition::Fields => {
                let val = seed.deserialize(FieldDeserializer::new(&mut *self.de))?;
                self.de.reader.opt_comma()?;
                self.de.reader.take_terminal(self.closing_bracket)?;
                Ok(val)
            }
            // SAFETY: MapAccess ends when Parsed::EndOfEntry is reached in `self.next_key_seed`
            EntryPosition::EndOfEntry => unreachable!(),
        }
    }
}

impl<'a, 'de: 'a, R> SeqAccess<'de> for EntryAccess<'a, 'de, R>
where
    R: BibtexReader<'de>,
{
    type Error = Error;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, Self::Error>
    where
        T: DeserializeSeed<'de>,
    {
        self.step_position();
        match self.pos {
            EntryPosition::EntryType => seed
                .deserialize(IdentifierDeserializer::new(self.entry_type))
                .map(Some),
            EntryPosition::CitationKey => {
                self.closing_bracket = self.de.reader.take_initial()?;
                seed.deserialize(IdentifierDeserializer::new_from_str(
                    self.de.reader.take_citation_key()?,
                ))
                .map(Some)
            }
            EntryPosition::Fields => {
                let val = seed
                    .deserialize(FieldDeserializer::new(&mut *self.de))
                    .map(Some)?;
                self.de.reader.opt_comma()?;
                self.de.reader.take_terminal(self.closing_bracket)?;
                Ok(val)
            }
            // SAFETY: We only permit deserialization into a tuple of length 3
            EntryPosition::EndOfEntry => unreachable!(),
        }
    }
}

/// Used to deserialize the fields key = value, ..
struct FieldDeserializer<'a, 'r, R>
where
    R: BibtexReader<'r>,
{
    de: &'a mut BibtexDeserializer<'r, R>,
}

impl<'a, 'r, R> FieldDeserializer<'a, 'r, R>
where
    R: BibtexReader<'r>,
{
    pub fn new(de: &'a mut BibtexDeserializer<'r, R>) -> Self {
        Self { de }
    }
}

impl<'a, 'de: 'a, R> de::Deserializer<'de> for FieldDeserializer<'a, 'de, R>
where
    R: BibtexReader<'de>,
{
    type Error = Error;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_map(self)
    }

    fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_seq(self)
    }

    #[inline]
    fn deserialize_tuple<V>(self, _len: usize, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        self.deserialize_seq(visitor)
    }

    #[inline]
    fn deserialize_tuple_struct<V>(
        self,
        _name: &'static str,
        _len: usize,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        self.deserialize_seq(visitor)
    }

    fn deserialize_ignored_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        self.de.reader.ignore_fields()?;
        visitor.visit_unit()
    }

    #[inline]
    fn deserialize_unit<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        self.deserialize_ignored_any(visitor)
    }

    #[inline]
    fn deserialize_unit_struct<V>(
        self,
        _name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        self.deserialize_ignored_any(visitor)
    }

    forward_to_deserialize_any!(
        bool i8 i16 i32 i64 u8 u16 u32 u64 f32 f64 char str string
        bytes byte_buf identifier option newtype_struct enum map struct);
}

impl<'a, 'de: 'a, R> MapAccess<'de> for FieldDeserializer<'a, 'de, R>
where
    R: BibtexReader<'de>,
{
    type Error = Error;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>, Self::Error>
    where
        K: DeserializeSeed<'de>,
    {
        match self.de.reader.take_comma_and_field_key()? {
            Some(identifier) => seed
                .deserialize(IdentifierDeserializer::new(identifier))
                .map(Some),
            None => Ok(None),
        }
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value, Self::Error>
    where
        V: DeserializeSeed<'de>,
    {
        self.de.reader.ignore_field_sep()?;
        seed.deserialize(ValueDeserializer::try_from_de_resolved(&mut *self.de)?)
    }
}

impl<'a, 'de: 'a, R> SeqAccess<'de> for FieldDeserializer<'a, 'de, R>
where
    R: BibtexReader<'de>,
{
    type Error = Error;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, Self::Error>
    where
        T: DeserializeSeed<'de>,
    {
        let field_key = match self.de.reader.take_comma_and_field_key()? {
            Some(field_key) => field_key,
            None => return Ok(None),
        };
        self.de.reader.ignore_field_sep()?;
        seed.deserialize(KeyValueDeserializer::new(&mut *self.de, field_key))
            .map(Some)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::reader::StrReader;
    use serde::Deserialize;
    use std::collections::HashMap;

    use std::borrow::Cow;

    // The basic target struct for testing
    #[derive(Deserialize, Debug, Hash, PartialEq, Eq)]
    #[serde(rename_all = "lowercase")]
    enum TestEntryType {
        Article,
        Book,
    }
    #[derive(Deserialize, Debug, PartialEq, Eq)]
    struct TestEntryStruct<'a> {
        entry_type: TestEntryType,
        citation_key: &'a str,
        #[serde(borrow)]
        fields: TestFields<'a>,
    }

    #[derive(Deserialize, Debug, PartialEq, Eq)]
    struct TestFields<'a> {
        #[serde(borrow)]
        author: Cow<'a, str>,
        #[serde(borrow)]
        title: Cow<'a, str>,
        year: u64,
    }

    // Anonymous field names and flexible receiver type
    #[derive(Debug, Deserialize, PartialEq)]
    enum Tok<'a> {
        #[serde(rename = "Abbrev")]
        A(&'a str),
        #[serde(rename = "Text")]
        T(&'a str),
    }

    #[derive(Deserialize, Debug, PartialEq)]
    struct TestEntryMap<'a> {
        entry_type: &'a str,
        citation_key: &'a str,
        #[serde(borrow)]
        fields: HashMap<&'a str, Vec<Tok<'a>>>,
    }

    #[test]
    fn test_entry_as_struct() {
        let reader = StrReader::new(
            r#"
            {key:0,
              author = {Auth} # {or},
              title = "Title",
              year = 2012,
            }"#,
        );
        let mut bib_de = BibtexDeserializer::new(reader);
        let deserializer =
            RegularEntryDeserializer::new(&mut bib_de, Identifier::from_str_unchecked("article"));

        let data: TestEntryStruct = TestEntryStruct::deserialize(deserializer).unwrap();
        let expected_data = TestEntryStruct {
            entry_type: TestEntryType::Article,
            citation_key: "key:0",
            fields: TestFields {
                author: "Author".into(),
                title: "Title".into(),
                year: 2012,
            },
        };

        assert_eq!(data, expected_data);
        assert!(matches!(data.fields.author, Cow::Owned(_)));
        assert!(matches!(data.fields.title, Cow::Borrowed(_)));
    }

    macro_rules! assert_de_entry {
        ($input:expr, $identifier: expr, $expected:expr, $target:tt) => {
            let reader = StrReader::new($input);
            let mut bib_de = BibtexDeserializer::new(reader);
            let deserializer = RegularEntryDeserializer::new(
                &mut bib_de,
                Identifier::from_str_unchecked($identifier),
            );
            assert_eq!(Ok($expected), $target::deserialize(deserializer));
        };
    }

    #[test]
    fn test_entry_as_map() {
        let mut expected_fields = HashMap::new();
        expected_fields.insert("author", vec![Tok::T("Auth"), Tok::T("or")]);
        expected_fields.insert("title", vec![Tok::A("title")]);
        expected_fields.insert("year", vec![Tok::T("2012")]);
        let expected_data = TestEntryMap {
            entry_type: "article",
            citation_key: "key:0",
            fields: expected_fields,
        };

        assert_de_entry!(
            r#"
            {key:0,
              author = {Auth} # {or},
              title = title,
              year = 2012,
            }"#,
            "article",
            expected_data,
            TestEntryMap
        );
    }

    #[test]
    fn test_entry_as_seq() {
        #[derive(Deserialize, Debug, PartialEq)]
        struct TupleEntry<'a>(&'a str, &'a str, TestFields<'a>);

        let expected_field_data = TestFields {
            author: "Author".into(),
            title: "Title".into(),
            year: 2012,
        };
        assert_de_entry!(
            r#"
            {key:0,
              author = {Auth} # {or},
              title = "Title",
              year = 2012,
            }"#,
            "article",
            TupleEntry("article", "key:0", expected_field_data),
            TupleEntry
        );

        type EntryT<'a> = (&'a str, &'a str, TestFields<'a>);

        let reader = StrReader::new(
            r#"
            {key:0,
              year = 2012,
              author = {Auth} # {or},
              title = "Title",
            }"#,
        );
        let mut bib_de = BibtexDeserializer::new(reader);
        let deserializer =
            RegularEntryDeserializer::new(&mut bib_de, Identifier::from_str_unchecked("article"));

        let data: EntryT = EntryT::deserialize(deserializer).unwrap();
        let expected_field_data = TestFields {
            author: "Author".into(),
            title: "Title".into(),
            year: 2012,
        };

        assert_eq!(data, ("article", "key:0", expected_field_data));
        assert_eq!(bib_de.reader.input, "");

        type Short<'a> = (&'a str, &'a str);
        let reader = StrReader::new("{k,a=b}");
        let mut bib_de = BibtexDeserializer::new(reader);
        let deserializer =
            RegularEntryDeserializer::new(&mut bib_de, Identifier::from_str_unchecked("a"));
        assert!(Short::deserialize(deserializer).is_err());

        type Long<'a> = (&'a str, &'a str, &'a str, &'a str);
        let reader = StrReader::new("{k,a=b}");
        let mut bib_de = BibtexDeserializer::new(reader);
        let deserializer =
            RegularEntryDeserializer::new(&mut bib_de, Identifier::from_str_unchecked("a"));
        assert!(Long::deserialize(deserializer).is_err());

        type Inf<'a> = Vec<&'a str>;
        let reader = StrReader::new("{k,a=b}");
        let mut bib_de = BibtexDeserializer::new(reader);
        let deserializer =
            RegularEntryDeserializer::new(&mut bib_de, Identifier::from_str_unchecked("a"));
        assert!(Inf::deserialize(deserializer).is_err());
    }

    #[test]
    fn test_entry_ignore() {
        use serde::de::IgnoredAny;

        let reader = StrReader::new(r#"(k,b="c",d=e # f,)"#);
        let mut bib_de = BibtexDeserializer::new(reader);
        let deserializer =
            RegularEntryDeserializer::new(&mut bib_de, Identifier::from_str_unchecked("a"));
        let res = IgnoredAny::deserialize(deserializer);
        assert!(res.is_ok())
    }

    #[test]
    fn test_ignore_unit() {
        #[derive(Deserialize, Debug, PartialEq)]
        struct Unit;
        let reader = StrReader::new("{k,a=b}");
        let mut bib_de = BibtexDeserializer::new(reader);
        let deserializer =
            RegularEntryDeserializer::new(&mut bib_de, Identifier::from_str_unchecked("article"));
        let data = Unit::deserialize(deserializer);
        assert!(data.is_ok(), "{:?}", data)
    }

    #[test]
    fn test_ignore_entry_meta() {
        #[derive(Deserialize, Debug, Hash, PartialEq, Eq)]
        struct TestSkipEntry<'a> {
            entry_type: TestEntryType,
            #[serde(borrow)]
            fields: TestSkipFields<'a>,
        }

        #[derive(Deserialize, Debug, Hash, PartialEq, Eq)]
        struct TestSkipFields<'a> {
            #[serde(borrow)]
            title: Cow<'a, str>,
            year: u64,
        }

        let reader = StrReader::new(
            r#"
            {key:0,
              author = {Author},
              title = "Title",
              year = 2012,
            }"#,
        );
        let mut bib_de = BibtexDeserializer::new(reader);
        let deserializer =
            RegularEntryDeserializer::new(&mut bib_de, Identifier::from_str_unchecked("article"));

        let data: TestSkipEntry = TestSkipEntry::deserialize(deserializer).unwrap();
        let expected_data = TestSkipEntry {
            entry_type: TestEntryType::Article,
            fields: TestSkipFields {
                title: Cow::Borrowed("Title"),
                year: 2012,
            },
        };

        assert_eq!(data, expected_data);
    }

    #[test]
    fn test_fields_as_map() {
        let reader = StrReader::new(", author = {Alex Rutar}, title = {A nice title},}");
        let mut bib_de = BibtexDeserializer::new(reader);
        let deserializer = FieldDeserializer::new(&mut bib_de);

        let data: HashMap<&str, &str> = HashMap::deserialize(deserializer).unwrap();
        let mut expected_data = HashMap::new();
        expected_data.insert("author", "Alex Rutar");
        expected_data.insert("title", "A nice title");

        assert_eq!(data, expected_data);
    }

    #[test]
    fn test_fields_as_seq() {
        let reader = StrReader::new(", author = {Alex Rutar}, title = {A nice title},}");
        let mut bib_de = BibtexDeserializer::new(reader);
        let deserializer = FieldDeserializer::new(&mut bib_de);

        type VecTupleFields<'a> = Vec<(&'a str, String)>;

        let data = VecTupleFields::deserialize(deserializer).unwrap();

        assert_eq!(
            data,
            vec![
                ("author", "Alex Rutar".to_string()),
                ("title", "A nice title".to_string())
            ]
        );

        let reader = StrReader::new(", author = {Alex Rutar}, title = {A nice title},}");
        let mut bib_de = BibtexDeserializer::new(reader);
        let deserializer = FieldDeserializer::new(&mut bib_de);

        #[derive(Deserialize, Debug, PartialEq)]
        struct TestField<'a> {
            #[serde(rename = "field_key")]
            k: &'a str,
            #[serde(rename = "field_value")]
            v: String,
        }
        type VecStructFields<'a> = Vec<TestField<'a>>;

        let data = VecStructFields::deserialize(deserializer).unwrap();

        assert_eq!(
            data,
            vec![
                TestField {
                    k: "author",
                    v: "Alex Rutar".to_string()
                },
                TestField {
                    k: "title",
                    v: "A nice title".to_string()
                },
            ]
        );
    }

    #[test]
    fn test_fields_as_map_enum() {
        let reader = StrReader::new(", year = 2012, month = 11, day = 5,}");
        let mut bib_de = BibtexDeserializer::new(reader);
        let deserializer = FieldDeserializer::new(&mut bib_de);

        #[derive(Deserialize, Debug, Hash, PartialEq, Eq)]
        #[serde(rename_all = "lowercase")]
        enum Date {
            Year,
            Month,
            Day,
        }

        let data: HashMap<Date, u16> = HashMap::deserialize(deserializer).unwrap();
        let mut expected_data = HashMap::new();
        expected_data.insert(Date::Year, 2012);
        expected_data.insert(Date::Month, 11);
        expected_data.insert(Date::Day, 5);

        assert_eq!(data, expected_data);
    }

    #[test]
    fn test_fields_as_struct() {
        #[derive(Deserialize, Debug, PartialEq)]
        struct MyFields<'a> {
            author: &'a str,
            title: &'a str,
            year: u32,
        }

        let reader =
            StrReader::new(", year = 20 # 12, author = {Alex Rutar}, title = {A nice title}}");
        let mut bib_de = BibtexDeserializer::new(reader);
        let deserializer = FieldDeserializer::new(&mut bib_de);

        assert_eq!(
            Ok(MyFields {
                author: "Alex Rutar",
                title: "A nice title",
                year: 2012
            }),
            MyFields::deserialize(deserializer)
        );
    }

    #[test]
    fn test_optional_struct_field() {
        // test optional fields
        #[derive(Deserialize, Debug, PartialEq)]
        struct OptionFields<'a> {
            author: &'a str,
            title: &'a str,
            year: Option<u32>,
        }
        let reader = StrReader::new(", author = {Alex Rutar}, title = {A nice title}}");
        let mut bib_de = BibtexDeserializer::new(reader);
        let deserializer = FieldDeserializer::new(&mut bib_de);

        assert_eq!(
            Ok(OptionFields {
                author: "Alex Rutar",
                title: "A nice title",
                year: None
            }),
            OptionFields::deserialize(deserializer)
        );
    }
}
