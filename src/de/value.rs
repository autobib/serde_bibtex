use std::borrow::Cow;

use serde::de::{self, value::BorrowedStrDeserializer, DeserializeSeed, SeqAccess, Visitor};
use serde::forward_to_deserialize_any;

use crate::error::Error;
use crate::parse::Flag;

use super::EntryDeserializer;

/// Used to deserialize Value.
pub struct ValueDeserializer<'a, 's, 'r> {
    de: &'a mut EntryDeserializer<'s, 'r>,
}

impl<'a, 's, 'r> ValueDeserializer<'a, 's, 'r> {
    pub fn new(de: &'a mut EntryDeserializer<'s, 'r>) -> Self {
        ValueDeserializer { de }
    }
}

macro_rules! deserialize_parse {
    ($method:ident, $visit:ident) => {
        fn $method<V>(self, visitor: V) -> Result<V::Value, Self::Error>
        where
            V: Visitor<'de>,
        {
            self.de.reader.take_flag()?.expect(Flag::FieldValue)?;
            visitor.$visit(self.de.reader.take_unit()?.parse()?)
        }
    };
}

impl<'a, 's, 'de: 'a> de::Deserializer<'de> for ValueDeserializer<'a, 's, 'de> {
    type Error = Error;

    #[inline]
    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_str(visitor)
    }

    deserialize_parse!(deserialize_bool, visit_bool);
    deserialize_parse!(deserialize_i8, visit_i8);
    deserialize_parse!(deserialize_i16, visit_i16);
    deserialize_parse!(deserialize_i32, visit_i32);
    deserialize_parse!(deserialize_i64, visit_i64);
    deserialize_parse!(deserialize_u8, visit_u8);
    deserialize_parse!(deserialize_u16, visit_u16);
    deserialize_parse!(deserialize_u32, visit_u32);
    deserialize_parse!(deserialize_u64, visit_u64);
    deserialize_parse!(deserialize_f32, visit_f32);
    deserialize_parse!(deserialize_f64, visit_f64);

    fn deserialize_char<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.de.reader.take_flag()?.expect(Flag::FieldValue)?;
        visitor.visit_char(self.de.reader.take_char()?)
    }

    fn deserialize_str<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.de.reader.take_flag()?.expect(Flag::FieldValue)?;
        match self.de.reader.take_unit()? {
            Cow::Borrowed(s) => visitor.visit_borrowed_str(s),
            Cow::Owned(s) => visitor.visit_str(&s),
        }
    }

    #[inline]
    fn deserialize_string<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_str(visitor)
    }

    fn deserialize_bytes<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        unimplemented!()
    }

    fn deserialize_byte_buf<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        unimplemented!()
    }

    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.de.reader.peek_flag()?.expect(Flag::FieldValue)?;
        let unit = self.de.reader.peek_unit()?;
        if unit.len() == 0 {
            // Manually clear the buffer.
            self.de.reader.clear_buffered_unit();
            visitor.visit_none()
        } else {
            visitor.visit_some(self)
        }
    }

    fn deserialize_unit<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.de.reader.take_flag()?.expect(Flag::FieldValue)?;
        self.de.reader.take_null()?;
        visitor.visit_unit()
    }

    fn deserialize_unit_struct<V>(
        self,
        _name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_unit(visitor)
    }

    fn deserialize_newtype_struct<V>(
        self,
        _name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_newtype_struct(self)
    }

    fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.de.reader.take_flag()?.expect(Flag::FieldValue)?;
        todo!()
    }

    #[inline]
    fn deserialize_tuple<V>(self, _len: usize, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
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
        V: Visitor<'de>,
    {
        self.deserialize_seq(visitor)
    }

    fn deserialize_enum<V>(
        self,
        name: &'static str,
        variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        todo!()
    }

    fn deserialize_identifier<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_str(visitor)
    }

    fn deserialize_ignored_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.de.reader.peek_flag()?.expect(Flag::FieldValue)?;
        self.de.reader.skip()?;
        visitor.visit_unit()
    }

    forward_to_deserialize_any!(map struct);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::abbrev::Abbreviations;
    use crate::de::EntryDeserializer;
    use serde::Deserialize;

    #[test]
    fn test_value_string() {
        let abbrevs = Abbreviations::default();
        let mut entry_de = EntryDeserializer::new(" = {Alex} # { Rutar}", &abbrevs);
        let deserializer = ValueDeserializer::new(&mut entry_de);

        assert_eq!(
            Ok("Alex Rutar".to_string()),
            String::deserialize(deserializer),
        );

        let abbrevs = Abbreviations::default();
        let mut entry_de = EntryDeserializer::new(" = {Author}", &abbrevs);
        let deserializer = ValueDeserializer::new(&mut entry_de);

        assert_eq!(Ok("Author".to_string()), String::deserialize(deserializer),);
    }

    #[test]
    fn test_value_cow() {
        let abbrevs = Abbreviations::default();
        let mut entry_de = EntryDeserializer::new(" = {Alex} # { Rutar}", &abbrevs);
        let deserializer = ValueDeserializer::new(&mut entry_de);

        assert_eq!(
            Ok(Cow::Borrowed("Alex Rutar")),
            Cow::deserialize(deserializer),
        );

        let abbrevs = Abbreviations::default();
        let mut entry_de = EntryDeserializer::new(" = {Author}", &abbrevs);
        let deserializer = ValueDeserializer::new(&mut entry_de);

        assert_eq!(Ok(Cow::Borrowed("Author")), Cow::deserialize(deserializer),);
    }

    #[test]
    fn test_value_str_borrowed() {
        #[derive(Deserialize, PartialEq, Eq, Debug)]
        struct Value<'a>(&'a str);

        let abbrevs = Abbreviations::default();
        let mut entry_de = EntryDeserializer::new(" = {Alex Rutar}", &abbrevs);
        let deserializer = ValueDeserializer::new(&mut entry_de);

        assert_eq!(Ok(Value("Alex Rutar")), Value::deserialize(deserializer));

        let abbrevs = Abbreviations::default();
        let mut entry_de = EntryDeserializer::new(" = {a} # {b}", &abbrevs);
        let deserializer = ValueDeserializer::new(&mut entry_de);

        assert!(Value::deserialize(deserializer).is_err());
    }

    #[test]
    fn test_value_parsed() {
        // bool
        let abbrevs = Abbreviations::default();
        let mut entry_de = EntryDeserializer::new(" ={tr}\n #{ue}", &abbrevs);
        let deserializer = ValueDeserializer::new(&mut entry_de);
        assert_eq!(Ok(true), bool::deserialize(deserializer));

        // i64
        let abbrevs = Abbreviations::default();
        let mut entry_de = EntryDeserializer::new("= {0} # \"1\" # 234", &abbrevs);
        let deserializer = ValueDeserializer::new(&mut entry_de);
        assert_eq!(Ok(1234), i16::deserialize(deserializer));
    }

    #[test]
    fn test_unit_struct() {
        #[derive(Deserialize, Debug, PartialEq)]
        struct Unit;

        let abbrevs = Abbreviations::default();
        let mut entry_de = EntryDeserializer::new(" ={} #{}", &abbrevs);
        let deserializer = ValueDeserializer::new(&mut entry_de);

        assert_eq!(Ok(Unit), Unit::deserialize(deserializer));
    }
}
