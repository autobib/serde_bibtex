use crate::bib::{Identifier, Token};
use serde::de::{
    self, value::BorrowedStrDeserializer, value::CowStrDeserializer, DeserializeSeed,
    IntoDeserializer, SeqAccess, VariantAccess, Visitor,
};

use crate::error::Error;
use serde::forward_to_deserialize_any;

/// A deserializer for an [`Identifier`].
///
/// Since [`Identifier`] is just a newtype wrapper over a borrowed `str`, this is essentially a wrapper around [`BorrowedStrDeserializer`].
/// In addition, we also support deserialization as as a newtype struct.
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
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char
        bytes byte_buf option unit unit_struct seq tuple
        tuple_struct map struct ignored_any identifier string str
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Deserialize;

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
}

// pub struct TokenDeserializer<'de> {
//     value: Token<'de>,
// }

// impl<'de> de::Deserializer<'de> for TokenDeserializer<'de> {
//     type Error = Error;

//     #[inline]
//     fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
//     where
//         V: de::Visitor<'de>,
//     {
//         self.deserialize_enum("", &[], visitor)
//     }

//     fn deserialize_enum<V>(
//         self,
//         _name: &str,
//         _variants: &'static [&'static str],
//         visitor: V,
//     ) -> Result<V::Value, Self::Error>
//     where
//         V: de::Visitor<'de>,
//     {
//         visitor.visit_enum(self)
//     }

//     forward_to_deserialize_any! {
//         bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
//         bytes byte_buf option unit unit_struct newtype_struct seq tuple
//         tuple_struct map struct identifier ignored_any
//     }
// }

// impl<'de> VariantAccess<'de> for TokenDeserializer<'de> {
//     type Error = Error;

//     fn unit_variant(self) -> Result<(), Self::Error> {
//         Err(Error::Message("Can only deserialize Token as newtype variant".into()))
//     }

//     fn newtype_variant_seed<T>(self, seed: T) -> Result<T::Value, Self::Error>
//     where
//         T: DeserializeSeed<'de> {
//         let cow = match self.value {
//             Token::Abbrev(identifier) => identifier.into_raw(),
//             Token::Text(cow) => cow
//         };
//         match cow {
//             Cow::Borrowed(s) => seed.deserialize(BorrowedStrDeserializer::new(s)),
//             Cow::Owned(s) => seed.deserialize(s.into_deserializer())
//         }
//     }

//     fn tuple_variant<V>(self, len: usize, visitor: V) -> Result<V::Value, Self::Error>
//     where
//         V: Visitor<'de> {
//         Err(Error::Message("Can only deserialize Token as newtype variant".into()))
//     }

//     fn struct_variant<V>(
//         self,
//         fields: &'static [&'static str],
//         visitor: V,
//     ) -> Result<V::Value, Self::Error>
//     where
//         V: Visitor<'de> {
//         Err(Error::Message("Can only deserialize Token as newtype variant".into()))
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
