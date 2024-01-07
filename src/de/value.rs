use std::borrow::Cow;

use serde::de::{
    self, value::BorrowedStrDeserializer, DeserializeSeed, EnumAccess, IntoDeserializer, SeqAccess,
    VariantAccess, Visitor,
};
use serde::forward_to_deserialize_any;

use crate::bib::{Identifier, Token};
use crate::error::Error;
use crate::parse::Flag;

use super::EntryDeserializer;

/// Deserialize a [`Flag::FieldValue`].
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

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.de.reader.take_flag()?.expect(Flag::FieldValue)?;
        match self.de.reader.take_unit()? {
            Cow::Borrowed(s) => visitor.visit_borrowed_str(s),
            Cow::Owned(s) => visitor.visit_str(&s),
        }
    }

    fn deserialize_char<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.de.reader.take_flag()?.expect(Flag::FieldValue)?;
        visitor.visit_char(self.de.reader.take_char()?)
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

    #[inline]
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

    #[inline]
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
        visitor.visit_seq(self)
    }

    #[inline]
    fn deserialize_tuple<V>(self, _len: usize, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
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
        V: Visitor<'de>,
    {
        self.deserialize_seq(visitor)
    }

    #[inline]
    fn deserialize_enum<V>(
        self,
        _name: &'static str,
        _variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_enum(self)
    }

    fn deserialize_ignored_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.de.reader.peek_flag()?.expect(Flag::FieldValue)?;
        self.de.reader.skip()?;
        visitor.visit_unit()
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

    forward_to_deserialize_any!(map struct str string identifier);
}

impl<'a, 's, 'de: 'a> SeqAccess<'de> for ValueDeserializer<'a, 's, 'de> {
    type Error = Error;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, Self::Error>
    where
        T: DeserializeSeed<'de>,
    {
        match self.de.reader.take_token()? {
            Some(token) => seed.deserialize(TokenDeserializer::new(token)).map(Some),
            None => Ok(None),
        }
    }
}

pub struct UnitEnumDeserializer;

impl<'de> VariantAccess<'de> for UnitEnumDeserializer {
    type Error = Error;

    fn unit_variant(self) -> Result<(), Self::Error> {
        Ok(())
    }

    fn newtype_variant_seed<T>(self, _seed: T) -> Result<T::Value, Self::Error>
    where
        T: DeserializeSeed<'de>,
    {
        Err(Error::Message("Unit variant only!".into()))
    }

    fn tuple_variant<V>(self, _len: usize, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        Err(Error::Message("Unit variant only!".into()))
    }

    fn struct_variant<V>(
        self,
        _fields: &'static [&'static str],
        _visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        Err(Error::Message("Unit variant only!".into()))
    }
}

impl<'a, 's, 'de: 'a> EnumAccess<'de> for ValueDeserializer<'a, 's, 'de> {
    type Error = Error;
    type Variant = UnitEnumDeserializer;

    fn variant_seed<V>(self, seed: V) -> Result<(V::Value, Self::Variant), Self::Error>
    where
        V: DeserializeSeed<'de>,
    {
        Ok((seed.deserialize(self)?, UnitEnumDeserializer {}))
    }
}

/// A deserializer for an [`Identifier`].
///
/// Since [`Identifier`] is just a newtype wrapper over a borrowed `str`, this is essentially a wrapper around [`BorrowedStrDeserializer`].
/// In addition, we also support deserialization as as a newtype struct.
#[derive(Debug, Copy, Clone)]
pub struct IdentifierDeserializer<'r> {
    value: Identifier<'r>,
}

impl<'r> IdentifierDeserializer<'r> {
    pub fn new(value: Identifier<'r>) -> Self {
        IdentifierDeserializer { value }
    }
}

impl<'de> de::Deserializer<'de> for IdentifierDeserializer<'de> {
    type Error = Error;

    #[inline]
    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_borrowed_str(self.value.into_raw())
    }

    #[inline]
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

    #[inline]
    fn deserialize_enum<V>(
        self,
        _name: &'static str,
        _variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_enum(BorrowedStrDeserializer::new(self.value.into_raw()))
    }

    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
        bytes byte_buf option unit unit_struct seq tuple tuple_struct
        map struct identifier ignored_any
    }
}

/// A deserializer for a [`Token`].
/// This only supports deserialization as an Enum.
pub struct TokenDeserializer<'r> {
    value: Token<'r>,
}

impl<'r> TokenDeserializer<'r> {
    pub fn new(value: Token<'r>) -> Self {
        Self { value }
    }
}

impl<'de> de::Deserializer<'de> for TokenDeserializer<'de> {
    type Error = Error;

    #[inline]
    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_enum(self)
    }

    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
        bytes byte_buf option unit unit_struct newtype_struct seq tuple
        tuple_struct map struct identifier ignored_any enum
    }
}

impl<'de> VariantAccess<'de> for TokenDeserializer<'de> {
    type Error = Error;

    fn unit_variant(self) -> Result<(), Self::Error> {
        Ok(())
    }

    fn newtype_variant_seed<T>(self, seed: T) -> Result<T::Value, Self::Error>
    where
        T: DeserializeSeed<'de>,
    {
        match self.value {
            Token::Abbrev(identifier) => seed.deserialize(IdentifierDeserializer::new(identifier)),
            Token::Text(Cow::Owned(s)) => seed.deserialize(s.into_deserializer()),
            Token::Text(Cow::Borrowed(s)) => seed.deserialize(BorrowedStrDeserializer::new(s)),
        }
    }

    fn tuple_variant<V>(self, _len: usize, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        Err(Error::Message("Cannot deserialize Token as tuple.".into()))
    }

    fn struct_variant<V>(
        self,
        _fields: &'static [&'static str],
        _visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        Err(Error::Message("Cannot deserialize Token as struct".into()))
    }
}

impl<'de> de::EnumAccess<'de> for TokenDeserializer<'de> {
    type Error = Error;
    type Variant = Self;

    fn variant_seed<T>(self, seed: T) -> Result<(T::Value, Self::Variant), Self::Error>
    where
        T: de::DeserializeSeed<'de>,
    {
        let de: BorrowedStrDeserializer<Self::Error> = match self.value {
            Token::Abbrev(_) => BorrowedStrDeserializer::new("Abbrev"),
            Token::Text(_) => BorrowedStrDeserializer::new("Text"),
        };
        Ok((seed.deserialize(de)?, self))
    }
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
        let mut entry_de = EntryDeserializer::new(" = {a} # { b}", &abbrevs);
        let deserializer = ValueDeserializer::new(&mut entry_de);

        assert_eq!(Ok("a b".to_string()), String::deserialize(deserializer),);

        let abbrevs = Abbreviations::default();
        let mut entry_de = EntryDeserializer::new(" = {a}", &abbrevs);
        let deserializer = ValueDeserializer::new(&mut entry_de);

        assert_eq!(Ok("a".to_string()), String::deserialize(deserializer),);
    }

    #[test]
    fn test_value_seq() {
        #[derive(Debug, Deserialize, PartialEq)]
        enum ShortToken<'a> {
            #[serde(rename = "Abbrev")]
            A(&'a str),
            #[serde(rename = "Text")]
            T(&'a str),
        }

        let abbrevs = Abbreviations::default();
        let mut entry_de = EntryDeserializer::new(" = {1} # a # {3}", &abbrevs);
        let deserializer = ValueDeserializer::new(&mut entry_de);

        let data: Vec<ShortToken> = Vec::deserialize(deserializer).unwrap();
        let expected_data = vec![ShortToken::T("1"), ShortToken::A("a"), ShortToken::T("3")];
        assert_eq!(data, expected_data);

        let abbrevs = Abbreviations::default();
        let mut entry_de = EntryDeserializer::new(" = {1} # a", &abbrevs);
        let deserializer = ValueDeserializer::new(&mut entry_de);

        type DoubleToken<'a> = (ShortToken<'a>, ShortToken<'a>);

        let data = DoubleToken::deserialize(deserializer).unwrap();
        let expected_data = (ShortToken::T("1"), ShortToken::A("a"));
        assert_eq!(data, expected_data);
    }

    #[test]
    fn test_value_cow() {
        let abbrevs = Abbreviations::default();
        let mut entry_de = EntryDeserializer::new(" = {a} # { b}", &abbrevs);
        let deserializer = ValueDeserializer::new(&mut entry_de);

        assert_eq!(
            Ok(Cow::Borrowed("a b").to_owned()),
            Cow::deserialize(deserializer),
        );

        let abbrevs = Abbreviations::default();
        let mut entry_de = EntryDeserializer::new(" = {a}", &abbrevs);
        let deserializer = ValueDeserializer::new(&mut entry_de);

        assert_eq!(Ok(Cow::Borrowed("a")), Cow::deserialize(deserializer),);
    }

    #[test]
    fn test_value_str_borrowed() {
        #[derive(Deserialize, PartialEq, Eq, Debug)]
        struct Value<'a>(&'a str);

        let abbrevs = Abbreviations::default();
        let mut entry_de = EntryDeserializer::new(" = {a}", &abbrevs);
        let deserializer = ValueDeserializer::new(&mut entry_de);

        assert_eq!(Ok(Value("a")), Value::deserialize(deserializer));

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
    fn test_value_enum() {
        #[derive(Deserialize, PartialEq, Debug)]
        #[serde(rename_all = "lowercase")]
        enum Month {
            Jan,
            Feb,
            Mar,
            // ...
        }

        let abbrevs = Abbreviations::default();
        let mut entry_de = EntryDeserializer::new(" = {jan}", &abbrevs);
        let deserializer = ValueDeserializer::new(&mut entry_de);
        assert_eq!(Ok(Month::Jan), Month::deserialize(deserializer));
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

    #[test]
    fn test_de_identifier() {
        // As string
        let de = IdentifierDeserializer::new(Identifier::from("key"));
        assert_eq!(Ok("key".to_string()), String::deserialize(de));

        // As newtype struct
        #[derive(Debug, Deserialize, PartialEq)]
        struct Id(String);

        let de = IdentifierDeserializer::new(Identifier::from("key"));
        assert_eq!(Id::deserialize(de), Ok(Id("key".to_string())));

        // As newtype with &str
        #[derive(Debug, Deserialize, PartialEq)]
        struct IdRef<'r>(&'r str);

        let de = IdentifierDeserializer::new(Identifier::from("key"));
        assert_eq!(IdRef::deserialize(de), Ok(IdRef("key")));

        // As enum
        #[derive(Debug, Deserialize, PartialEq)]
        #[serde(rename_all = "lowercase")]
        enum AllowedKeys {
            Key,
            Other,
        }

        let de = IdentifierDeserializer::new(Identifier::from("key"));
        assert_eq!(AllowedKeys::deserialize(de), Ok(AllowedKeys::Key));
    }

    #[test]
    fn test_de_token() {
        // Deserialize as a short version of Token
        #[derive(Debug, Deserialize, PartialEq)]
        enum ShortToken {
            #[serde(rename = "Abbrev")]
            A(String),
            #[serde(rename = "Text")]
            T(String),
        }

        let de = TokenDeserializer::new(Token::abbrev_from("key"));
        assert_eq!(ShortToken::deserialize(de), Ok(ShortToken::A("key".into())));
        let de = TokenDeserializer::new(Token::text_from("val"));
        assert_eq!(ShortToken::deserialize(de), Ok(ShortToken::T("val".into())));

        // Essentially the same enum as Token
        #[derive(Debug, Deserialize, PartialEq)]
        struct I<'r>(&'r str);

        #[derive(Debug, Deserialize, PartialEq)]
        enum ReToken<'r> {
            #[serde(rename = "Abbrev")]
            A(I<'r>),
            #[serde(borrow)]
            #[serde(rename = "Text")]
            T(Cow<'r, str>),
        }

        let de = TokenDeserializer::new(Token::abbrev_from("key"));
        assert_eq!(ReToken::deserialize(de), Ok(ReToken::A(I("key"))));

        let de = TokenDeserializer::new(Token::text_from("key"));
        let data = ReToken::deserialize(de).unwrap();
        assert_eq!(data, ReToken::T(Cow::Borrowed("key")));
        assert!(matches!(data, ReToken::T(Cow::Borrowed(_))));
    }

    #[test]
    fn test_value_de_owned() {
        // Test that we only take ownership when necessary.

        #[derive(Deserialize, Debug, PartialEq, Eq)]
        struct Val<'r>(#[serde(borrow)] Cow<'r, str>);

        macro_rules! assert_value_matching {
            ($input:expr, $expected:expr, $cow:pat) => {
                let abbrevs = Abbreviations::default();
                let mut entry_de = EntryDeserializer::new($input, &abbrevs);
                let deserializer = ValueDeserializer::new(&mut entry_de);
                let data = Val::deserialize(deserializer);
                let expected = Val($expected.into());
                assert_eq!(data, Ok(expected));
                assert!(matches!(data, Ok(Val($cow))));
            };
        }

        // separated Token::Text are merged
        assert_value_matching!(" = {a} # {b} # {c}", "abc", Cow::Owned(_));

        // a single Token::Text can be borrowed
        assert_value_matching!(" = {a}", "a", Cow::Borrowed(_));

        // empty values still allow owning
        assert_value_matching!(" = {} # {abc}", "abc", Cow::Borrowed(_));
        assert_value_matching!(" = {} # {abc} # {} # {}", "abc", Cow::Borrowed(_));
        assert_value_matching!(" = {abc} # {}", "abc", Cow::Borrowed(_));
        assert_value_matching!(" = {a} # {} # {b}", "ab", Cow::Owned(_));
    }
}
