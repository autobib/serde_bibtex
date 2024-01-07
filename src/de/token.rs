use serde::de::{
    self,
    value::{BorrowedStrDeserializer, CowStrDeserializer},
    DeserializeSeed, VariantAccess, Visitor,
};
use serde::forward_to_deserialize_any;

use crate::bib::{Identifier, Token};
use crate::error::Error;

/// A deserializer for an [`Identifier`].
///
/// Since [`Identifier`] is just a newtype wrapper over a borrowed `str`, this is essentially a wrapper around [`BorrowedStrDeserializer`].
/// In addition, we also support deserialization as as a newtype struct.
#[derive(Debug, Copy, Clone)]
pub struct IdentifierDeserializer<'de> {
    value: Identifier<'de>,
}

impl<'de> IdentifierDeserializer<'de> {
    pub fn new(value: Identifier<'de>) -> Self {
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

/// A deserializer for a [`Token`]. This only supports deserialization as an Enum.
pub struct TokenDeserializer<'de> {
    value: Token<'de>,
}

impl<'de> TokenDeserializer<'de> {
    pub fn new(value: Token<'de>) -> Self {
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
        Err(Error::Message(
            "Can only deserialize Token as newtype variant".into(),
        ))
    }

    fn newtype_variant_seed<T>(self, seed: T) -> Result<T::Value, Self::Error>
    where
        T: DeserializeSeed<'de>,
    {
        match self.value {
            Token::Abbrev(identifier) => seed.deserialize(IdentifierDeserializer::new(identifier)),
            Token::Text(cow) => seed.deserialize(CowStrDeserializer::new(cow)),
        }
    }

    fn tuple_variant<V>(self, _len: usize, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        Err(Error::Message(
            "Can only deserialize Token as newtype variant".into(),
        ))
    }

    fn struct_variant<V>(
        self,
        _fields: &'static [&'static str],
        _visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        Err(Error::Message(
            "Can only deserialize Token as newtype variant".into(),
        ))
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
    use serde::Deserialize;
    use std::borrow::Cow;

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
        assert_eq!(
            ReToken::deserialize(de),
            Ok(ReToken::T(Cow::Borrowed("key")))
        );
    }
}
