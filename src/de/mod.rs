mod reader;
mod value;

use reader::ResolvingReader;
use serde::de::{self, value::BorrowedStrDeserializer, DeserializeSeed, MapAccess};
use serde::forward_to_deserialize_any;

use crate::abbrev::Abbreviations;
use crate::error::Error;
use crate::parse::Flag;

use value::ValueDeserializer;

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
        // TODO: check length three
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
        visitor.visit_map(EntryAccess::new(self))
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
        todo!()
    }

    forward_to_deserialize_any!(
        bool i8 i16 i32 i64 u8 u16 u32 u64 f32 f64 char str
        string bytes byte_buf option unit unit_struct newtype_struct
        enum identifier);
}

struct EntryAccess<'a, 's, 'de> {
    de: &'a mut EntryDeserializer<'s, 'de>,
}

impl<'a, 's, 'de> EntryAccess<'a, 's, 'de> {
    fn new(de: &'a mut EntryDeserializer<'s, 'de>) -> Self {
        EntryAccess { de }
    }
}

impl<'a, 's, 'de> MapAccess<'de> for EntryAccess<'a, 's, 'de> {
    type Error = Error;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>, Self::Error>
    where
        K: DeserializeSeed<'de>,
    {
        match self.de.reader.peek_flag()? {
            Flag::EntryType => seed
                .deserialize(BorrowedStrDeserializer::new("entry_type"))
                .map(Some),
            Flag::EntryKey => seed
                .deserialize(BorrowedStrDeserializer::new("entry_key"))
                .map(Some),
            Flag::FieldKey => seed
                .deserialize(BorrowedStrDeserializer::new("fields"))
                .map(Some),
            Flag::EndOfEntry => Ok(None),
            _ => Err(Error::Message("Unexpected flag".to_string())),
        }
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value, Self::Error>
    where
        V: DeserializeSeed<'de>,
    {
        match self.de.reader.peek_flag()? {
            Flag::EntryType => {
                self.de.reader.clear_buffered_flag();
                seed.deserialize(BorrowedStrDeserializer::new(
                    self.de.reader.take_entry_type()?,
                ))
            }
            Flag::EntryKey => {
                self.de.reader.clear_buffered_flag();
                seed.deserialize(BorrowedStrDeserializer::new(
                    self.de.reader.take_entry_key()?,
                ))
            }
            Flag::FieldKey => seed.deserialize(FieldDeserializer::new(&mut *self.de)),
            _ => Err(Error::Message(
                "expected entry type entry key or field key".to_string(),
            )),
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

impl<'a, 's, 'de> MapAccess<'de> for FieldDeserializer<'a, 's, 'de> {
    type Error = Error;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>, Self::Error>
    where
        K: DeserializeSeed<'de>,
    {
        match self.de.reader.peek_flag()? {
            Flag::FieldKey => {
                self.de.reader.take_flag()?;
                seed.deserialize(BorrowedStrDeserializer::new(
                    self.de.reader.take_field_key()?,
                ))
                .map(Some)
            }
            Flag::EndOfEntry => Ok(None),
            _ => Err(Error::Message("Unexpected flag".to_string())),
        }
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value, Self::Error>
    where
        V: DeserializeSeed<'de>,
    {
        self.de.reader.peek_flag()?.expect(Flag::FieldValue)?;
        seed.deserialize(ValueDeserializer::new(&mut *self.de))
    }
}

// struct DeValueSeq<'a, 's, 'de> {
//     de: &'a mut EntryDeserializer<'s, 'de>,
//     first: bool,
// }

// impl<'a, 's, 'de> DeValueSeq<'a, 's, 'de> {
//     fn new(de: &'a mut EntryDeserializer<'s, 'de>) -> Self {
//         Self { de, first: true }
//     }
// }

// impl<'a, 's, 'de> SeqAccess<'de> for DeValueSeq<'a, 's, 'de> {
//     type Error = Error;

//     fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, Self::Error>
//     where
//         T: DeserializeSeed<'de>,
//     {
//         // if self.first {
//         // let token = self.de.reader.take_first_token()?;
//         // }
//         todo!()
//     }
// }

// pub struct TokenDeserializer<'de> {
//     value: Token<'de>,
// }

// impl<'de> de::Deserializer<'de> for TokenDeserializer<'de> {
//     type Error = Error;

//     fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
//     where
//         V: de::Visitor<'de>,
//     {
//         visitor.visit_borrowed_str(self.value)
//     }

//     fn deserialize_enum<V>(
//         self,
//         name: &str,
//         variants: &'static [&'static str],
//         visitor: V,
//     ) -> Result<V::Value, Self::Error>
//     where
//         V: de::Visitor<'de>,
//     {
//         let _ = name;
//         let _ = variants;
//         visitor.visit_enum(self)
//     }

//     forward_to_deserialize_any! {
//         bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
//         bytes byte_buf option unit unit_struct newtype_struct seq tuple
//         tuple_struct map struct identifier ignored_any
//     }
// }

// impl<'de> de::EnumAccess<'de> for TokenDeserializer<'de> {
//     type Error = Error;
//     type Variant = Self;

//     fn variant_seed<T>(self, seed: T) -> Result<(T::Value, Self::Variant), Self::Error>
//     where
//         T: de::DeserializeSeed<'de>,
//     {
//         seed.deserialize(self).map(private::unit_only)
//     }
// }

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Deserialize;
    use std::collections::HashMap;

    #[test]
    fn test_entry() {
        #[derive(Deserialize, Debug, Hash, PartialEq, Eq)]
        #[serde(rename_all = "lowercase")]
        enum EntryType {
            Article,
            Book,
        }
        #[derive(Deserialize, Debug, Hash, PartialEq, Eq)]
        struct Entry<'a> {
            entry_type: EntryType,
            entry_key: &'a str,
            fields: Fields<'a>,
        }

        #[derive(Deserialize, Debug, Hash, PartialEq, Eq)]
        struct Fields<'a> {
            author: &'a str,
            title: &'a str,
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

        let data: Entry = Entry::deserialize(&mut de_entry).unwrap();
        let expected_data = Entry {
            entry_type: EntryType::Article,
            entry_key: "key:0",
            fields: Fields {
                author: "Author",
                title: "Title",
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
