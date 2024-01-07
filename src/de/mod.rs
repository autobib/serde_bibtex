mod reader;
mod value;

use reader::{Position, ResolvingReader};
use serde::de::{self, value::BorrowedStrDeserializer, DeserializeSeed, MapAccess};
use serde::forward_to_deserialize_any;

use crate::abbrev::Abbreviations;
use crate::error::Error;

use value::{IdentifierDeserializer, ValueDeserializer};

/// The top level deserializer.
pub struct EntryDeserializer<'s, 'r> {
    reader: ResolvingReader<'s, 'r>,
}

impl<'s, 'r> EntryDeserializer<'s, 'r> {
    pub fn new(input: &'r str, abbrevs: &'s Abbreviations<'r>) -> Self {
        EntryDeserializer {
            reader: ResolvingReader::new(input, abbrevs),
        }
    }
}

impl<'a, 's, 'de: 'a> de::Deserializer<'de> for &'a mut EntryDeserializer<'s, 'de> {
    type Error = Error;

    #[inline]
    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        self.deserialize_map(visitor)
    }

    fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        todo!()
    }

    fn deserialize_tuple<V>(self, len: usize, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        if len != 3 {
            Err(Error::Message(
                "Tuple deserialization requires exactly three fields".to_string(),
            ))
        } else {
            self.deserialize_seq(visitor)
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

    fn deserialize_map<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        let value = visitor.visit_map(EntryAccess::new(self))?;
        self.reader.take_terminal()?;
        Ok(value)
    }

    #[inline]
    fn deserialize_struct<V>(
        self,
        _name: &'static str,
        _fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        self.deserialize_map(visitor)
    }

    fn deserialize_ignored_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        self.reader.ignore_entry()?;
        visitor.visit_unit()
    }

    forward_to_deserialize_any!(
        bool i8 i16 i32 i64 u8 u16 u32 u64 f32 f64 char str
        string bytes byte_buf option unit unit_struct newtype_struct
        enum identifier);
}

struct EntryAccess<'a, 's, 'r> {
    de: &'a mut EntryDeserializer<'s, 'r>,
}

impl<'a, 's, 'de: 'a> EntryAccess<'a, 's, 'de> {
    fn new(de: &'a mut EntryDeserializer<'s, 'de>) -> Self {
        EntryAccess { de }
    }
}

impl<'a, 's, 'de: 'a> MapAccess<'de> for EntryAccess<'a, 's, 'de> {
    type Error = Error;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>, Self::Error>
    where
        K: DeserializeSeed<'de>,
    {
        match self.de.reader.update_position() {
            Position::EntryType => seed
                .deserialize(BorrowedStrDeserializer::new("entry_type"))
                .map(Some),
            Position::CitationKey => seed
                .deserialize(BorrowedStrDeserializer::new("entry_key"))
                .map(Some),
            Position::Fields => seed
                .deserialize(BorrowedStrDeserializer::new("fields"))
                .map(Some),
            Position::EndOfEntry => Ok(None),
        }
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value, Self::Error>
    where
        V: DeserializeSeed<'de>,
    {
        match self.de.reader.get_position() {
            Position::EntryType => seed.deserialize(IdentifierDeserializer::new(
                self.de.reader.take_entry_type()?,
            )),
            Position::CitationKey => seed.deserialize(BorrowedStrDeserializer::new(
                self.de.reader.take_citation_key()?,
            )),
            Position::Fields => seed.deserialize(FieldDeserializer::new(&mut *self.de)),
            // SAFETY: MapAccess ends when Parsed::EndOfEntry is reached in `self.next_key_seed`
            Position::EndOfEntry => unreachable!(),
        }
    }
}

/// Used to deserialize the fields key = value, ..
struct FieldDeserializer<'a, 's, 'r> {
    de: &'a mut EntryDeserializer<'s, 'r>,
}

impl<'a, 's, 'r> FieldDeserializer<'a, 's, 'r> {
    pub fn new(de: &'a mut EntryDeserializer<'s, 'r>) -> Self {
        FieldDeserializer { de }
    }
}

impl<'a, 's, 'de: 'a> de::Deserializer<'de> for FieldDeserializer<'a, 's, 'de> {
    type Error = Error;

    #[inline]
    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        self.deserialize_map(visitor)
    }

    fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        todo!()
    }

    fn deserialize_tuple<V>(self, _len: usize, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        self.deserialize_seq(visitor)
    }

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

    fn deserialize_map<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_map(self)
    }

    #[inline]
    fn deserialize_struct<V>(
        self,
        _name: &'static str,
        _fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        self.deserialize_map(visitor)
    }

    fn deserialize_identifier<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        self.deserialize_str(visitor)
    }

    fn deserialize_ignored_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        todo!()
    }

    forward_to_deserialize_any!(
        bool i8 i16 i32 i64 u8 u16 u32 u64 f32 f64 char str
        string bytes byte_buf option unit unit_struct newtype_struct enum);
}

impl<'a, 's, 'de: 'a> MapAccess<'de> for FieldDeserializer<'a, 's, 'de> {
    type Error = Error;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>, Self::Error>
    where
        K: DeserializeSeed<'de>,
    {
        match self.de.reader.take_field_key()? {
            Some(identifier) => seed
                .deserialize(IdentifierDeserializer::new(identifier))
                .map(Some),
            None => Ok(None),
        }
        // match self.de.reader.peek_flag()? {
        //     Flag::FieldKey => {
        //         self.de.reader.clear_buffered_flag();
        //         seed.deserialize(IdentifierDeserializer::new(
        //             self.de.reader.take_field_key()?,
        //         ))
        //         .map(Some)
        //     }
        //     Flag::EndOfEntry => Ok(None),
        //     _ => Err(Error::Message("Unexpected flag".to_string())),
        // }
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value, Self::Error>
    where
        V: DeserializeSeed<'de>,
    {
        seed.deserialize(ValueDeserializer::new(&mut *self.de))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
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
        entry_key: &'a str,
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
        entry_key: &'a str,
        #[serde(borrow)]
        fields: HashMap<&'a str, Vec<Tok<'a>>>,
    }

    #[test]
    fn test_entry() {
        let abbrevs = Abbreviations::default();
        let mut de_entry = EntryDeserializer::new(
            r#"
            @article{key:0,
              author = {Auth} # {or},
              title = "Title",
              year = 2012,
            }"#,
            &abbrevs,
        );

        let data: TestEntryStruct = TestEntryStruct::deserialize(&mut de_entry).unwrap();
        let expected_data = TestEntryStruct {
            entry_type: TestEntryType::Article,
            entry_key: "key:0",
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

    #[test]
    fn test_syntax() {
        use serde::de::IgnoredAny;

        macro_rules! assert_syntax {
            ($input:expr, $expect:ident) => {
                let abbrevs = Abbreviations::default();

                let mut de_entry = EntryDeserializer::new($input, &abbrevs);
                let data: Result<IgnoredAny, Error> = IgnoredAny::deserialize(&mut de_entry);
                assert!(data.$expect(), "{:?} : {:?}", data, de_entry.reader);

                let mut de_entry = EntryDeserializer::new($input, &abbrevs);
                let data: Result<TestEntryMap, Error> = TestEntryMap::deserialize(&mut de_entry);
                assert!(data.$expect(), "{:?} : {:?}", data, de_entry.reader);
            };
        }

        // basic example
        assert_syntax!(
            r#"@a{key:0,
              a= {A} # b,
              t= "T",
              y= 1,}"#,
            is_ok
        );

        // whitespace and unicode allowed in potentially surprising places
        assert_syntax!(
            r#"@   a🍄ticle {k🍄:0  ,
              au🍄hor ={A🍄th}
                #  
                {or}
                ,title =
              "Tit🍄e" # 🍄}"#,
            is_ok
        );

        // no fields, trailing comma
        assert_syntax!(r#"@a{k,}"#, is_ok);
        // no fields, no trailing comma
        assert_syntax!(r#"@a{k}"#, is_ok);
        // single field, trailing comma
        assert_syntax!(r#"@a{k,t=v,}"#, is_ok);
        // single field, no trailing comma
        assert_syntax!(r#"@a{k,t=v}"#, is_ok);

        // err: multiple trailing comma
        assert_syntax!(r#"@a{k,,}"#, is_err);
        // err: missing field value
        assert_syntax!(r#"@a{k,t=,}"#, is_err);
        // err: missing leading @
        assert_syntax!(r#"a{k,t=v}"#, is_err);
        // err: missing citation key
        assert_syntax!(r#"@a{,t=v}"#, is_err);
        // err: invalid char in citation key
        assert_syntax!(r#"@a{t=b}"#, is_err);
        assert_syntax!(r#"@a{t#b}"#, is_err);
        assert_syntax!(r#"@a{t\b}"#, is_err);

        // opening and closing brackets must match
        assert_syntax!("@a(k}", is_err);
        assert_syntax!("@a{k)", is_err);
        assert_syntax!("@a{k}", is_ok);
        assert_syntax!("@a(k)", is_ok);
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

        let abbrevs = Abbreviations::default();
        let mut de_entry = EntryDeserializer::new(
            r#"
            @article{key:0,
              author = {Author},
              title = "Title",
              year = 2012,
            }"#,
            &abbrevs,
        );

        let data: TestSkipEntry = TestSkipEntry::deserialize(&mut de_entry).unwrap();
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
        let abbrevs = Abbreviations::default();
        let mut de_entry = EntryDeserializer::new(
            ", author = {Alex Rutar}, title = {A nice title},}",
            &abbrevs,
        );
        let deserializer = FieldDeserializer::new(&mut de_entry);

        let data: HashMap<&str, &str> = HashMap::deserialize(deserializer).unwrap();
        let mut expected_data = HashMap::new();
        expected_data.insert("author", "Alex Rutar");
        expected_data.insert("title", "A nice title");

        assert_eq!(data, expected_data);
    }

    #[test]
    fn test_fields_as_map_enum() {
        let abbrevs = Abbreviations::default();
        let mut de_entry = EntryDeserializer::new(", year = 2012, month = 11, day = 5,}", &abbrevs);
        let deserializer = FieldDeserializer::new(&mut de_entry);

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

        let mut abbrevs = Abbreviations::default();
        let mut de_entry = EntryDeserializer::new(
            ", year = 20 # 12, author = {Alex Rutar}, title = {A nice title}}",
            &mut abbrevs,
        );
        let deserializer = FieldDeserializer::new(&mut de_entry);

        assert_eq!(
            Ok(MyFields {
                author: "Alex Rutar",
                title: "A nice title",
                year: 2012
            }),
            MyFields::deserialize(deserializer)
        );
    }
}
