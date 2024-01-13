use std::borrow::Cow;

use serde::de::{
    self, value::BorrowedStrDeserializer, DeserializeSeed, EnumAccess, IntoDeserializer, MapAccess,
    SeqAccess, Unexpected, VariantAccess, Visitor,
};
use serde::forward_to_deserialize_any;

use crate::error::Error;
use crate::naming::{
    ABBREV_TOKEN_VARIANT_NAME, FIELD_KEY_NAME, FIELD_VALUE_NAME, TEXT_TOKEN_VARIANT_NAME,
};
use crate::value::{Identifier, Token};

use super::BibtexDeserializer;
use crate::parse::BibtexReader;

enum KeyValuePosition {
    Start,
    FieldKey,
    FieldValue,
}

pub struct KeyValueDeserializer<'a, 'r, R>
where
    R: BibtexReader<'r>,
{
    de: &'a mut BibtexDeserializer<'r, R>,
    id: Identifier<'r>,
    pos: KeyValuePosition,
    capture: bool,
}

impl<'a, 'r, R> KeyValueDeserializer<'a, 'r, R>
where
    R: BibtexReader<'r>,
{
    pub fn new(de: &'a mut BibtexDeserializer<'r, R>, id: Identifier<'r>) -> Self {
        Self {
            id,
            de,
            pos: KeyValuePosition::Start,
            capture: false,
        }
    }

    pub fn new_captured(de: &'a mut BibtexDeserializer<'r, R>, id: Identifier<'r>) -> Self {
        Self {
            id,
            de,
            pos: KeyValuePosition::Start,
            capture: true,
        }
    }

    fn step_position(&mut self) {
        self.pos = match self.pos {
            KeyValuePosition::Start => KeyValuePosition::FieldKey,
            KeyValuePosition::FieldKey => KeyValuePosition::FieldValue,
            KeyValuePosition::FieldValue => KeyValuePosition::Start,
        };
    }
}

impl<'a, 'de: 'a, R> de::Deserializer<'de> for KeyValueDeserializer<'a, 'de, R>
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

    fn deserialize_tuple<V>(self, len: usize, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        if len == 2 {
            visitor.visit_seq(self)
        } else {
            Err(Error::Message(
                "Error: cannot deserialize key-value pair as tuple of length other than 2"
                    .to_string(),
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
        V: Visitor<'de>,
    {
        self.deserialize_tuple(len, visitor)
    }

    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
            bytes byte_buf option unit unit_struct newtype_struct seq
            map struct enum identifier ignored_any
    }
}

impl<'a, 'de: 'a, R> SeqAccess<'de> for KeyValueDeserializer<'a, 'de, R>
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
            KeyValuePosition::FieldKey => seed
                .deserialize(IdentifierDeserializer::new(self.id))
                .map(Some),
            KeyValuePosition::FieldValue => {
                if self.capture {
                    seed.deserialize(ValueDeserializer::try_from_de_captured(
                        self.id,
                        &mut *self.de,
                    )?)
                    .map(Some)
                } else {
                    seed.deserialize(ValueDeserializer::try_from_de_resolved(&mut *self.de)?)
                        .map(Some)
                }
            }
            // SAFETY: seq is only deserialized if length exactly 2
            _ => unreachable!(),
        }
    }
}

impl<'a, 'de: 'a, R> MapAccess<'de> for KeyValueDeserializer<'a, 'de, R>
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
            KeyValuePosition::FieldKey => seed
                .deserialize(BorrowedStrDeserializer::new(FIELD_KEY_NAME))
                .map(Some),
            KeyValuePosition::FieldValue => seed
                .deserialize(BorrowedStrDeserializer::new(FIELD_VALUE_NAME))
                .map(Some),
            _ => Ok(None),
        }
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value, Self::Error>
    where
        V: DeserializeSeed<'de>,
    {
        match self.pos {
            KeyValuePosition::FieldKey => seed.deserialize(IdentifierDeserializer::new(self.id)),
            KeyValuePosition::FieldValue => {
                if self.capture {
                    seed.deserialize(ValueDeserializer::try_from_de_captured(
                        self.id,
                        &mut *self.de,
                    )?)
                } else {
                    seed.deserialize(ValueDeserializer::try_from_de_resolved(&mut *self.de)?)
                }
            }
            // SAFETY: `self.pos == KeyValuePosition::Entry` causes next_key_seed to terminate
            _ => unreachable!(),
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
        Err(de::Error::invalid_type(
            Unexpected::NewtypeVariant,
            &"value as newtype variant",
        ))
    }

    fn tuple_variant<V>(self, _len: usize, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        Err(de::Error::invalid_type(
            Unexpected::TupleVariant,
            &"value as tuple variant",
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
        Err(de::Error::invalid_type(
            Unexpected::StructVariant,
            &"value as struct variant",
        ))
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

    pub fn new_from_str(s: &'r str) -> Self {
        IdentifierDeserializer {
            value: Identifier::from_str_unchecked(s),
        }
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
pub struct TokenDeserializer<'r> {
    value: Token<'r>,
}

// impl<'de> serde::de::IntoDeserializer<'de> for TokenDeserializer<'de> {
//     type Deserializer = Self;

//     fn into_deserializer(self) -> Self::Deserializer {
//         self
//     }
// }

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
            Token::Abbrev(_) => BorrowedStrDeserializer::new(ABBREV_TOKEN_VARIANT_NAME),
            Token::Text(_) => BorrowedStrDeserializer::new(TEXT_TOKEN_VARIANT_NAME),
        };
        Ok((seed.deserialize(de)?, self))
    }
}

pub struct ValueDeserializer<'a, 'r> {
    iter: std::vec::Drain<'a, Token<'r>>,
}

impl<'a, 'r> ValueDeserializer<'a, 'r> {
    pub fn new(scratch: &'a mut Vec<Token<'r>>) -> Self {
        Self {
            iter: scratch.drain(..),
        }
    }

    /// Create a new value directly from the tokens, without resolving macros.
    pub(crate) fn try_from_de<R>(de: &'a mut BibtexDeserializer<'r, R>) -> Result<Self, Error>
    where
        R: BibtexReader<'r>,
    {
        de.scratch.clear();
        de.reader.take_value_into(&mut de.scratch)?;
        Ok(Self {
            iter: de.scratch.drain(..),
        })
    }

    /// Create a new value from the tokens after resolving macros.
    pub(crate) fn try_from_de_resolved<R>(
        de: &'a mut BibtexDeserializer<'r, R>,
    ) -> Result<Self, Error>
    where
        R: BibtexReader<'r>,
    {
        de.scratch.clear();
        de.reader.take_value_into(&mut de.scratch)?;
        de.abbrev.resolve_tokens(&mut de.scratch);
        Ok(Self {
            iter: de.scratch.drain(..),
        })
    }

    /// Create a new value from the tokens after resolving macros and inserting into the
    /// abbreviations.
    pub(crate) fn try_from_de_captured<R>(
        id: Identifier<'r>,
        de: &'a mut BibtexDeserializer<'r, R>,
    ) -> Result<Self, Error>
    where
        R: BibtexReader<'r>,
    {
        de.scratch.clear();
        de.reader.take_value_into(&mut de.scratch)?;
        de.abbrev.resolve_tokens(&mut de.scratch);
        de.abbrev.insert_raw_tokens(id, de.scratch.clone());
        Ok(Self {
            iter: de.scratch.drain(..),
        })
    }

    fn as_cow(&mut self) -> Result<Cow<'r, str>, Error> {
        let mut init = loop {
            match self.iter.next() {
                Some(token) => {
                    let cow: Cow<'r, str> = token.try_into()?;
                    if cow.len() > 0 {
                        break cow;
                    }
                }
                None => return Ok(Cow::Borrowed("")),
            }
        };

        while let Some(token) = self.iter.next() {
            let cow: Cow<'r, str> = token.try_into()?;
            if cow.len() > 0 {
                init.to_mut().push_str(&cow)
            }
        }
        Ok(init)
    }

    fn as_char(&mut self) -> Result<char, Error> {
        let mut found_char: Option<char> = None;

        while let Some(token) = self.iter.next() {
            let cow: Cow<'r, str> = token.try_into()?;
            for char in cow.chars() {
                if let Some(_) = found_char {
                    return Err(Error::Message("Too many chars.".to_string()));
                } else {
                    found_char = Some(char);
                }
            }
        }

        found_char.ok_or(Error::Message("Expected char, got nothing.".to_string()))
    }
}

macro_rules! deserialize_parse {
    ($method:ident, $visit:ident) => {
        fn $method<V>(mut self, visitor: V) -> Result<V::Value, Self::Error>
        where
            V: Visitor<'de>,
        {
            visitor.$visit(self.as_cow()?.parse()?)
        }
    };
}

impl<'a, 'de: 'a> de::Deserializer<'de> for ValueDeserializer<'a, 'de> {
    type Error = Error;

    fn deserialize_any<V>(mut self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self.as_cow()? {
            Cow::Borrowed(s) => visitor.visit_borrowed_str(s),
            Cow::Owned(s) => visitor.visit_str(&s),
        }
    }

    fn deserialize_char<V>(mut self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_char(self.as_char()?)
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
        // TODO: figure this out, need to fix errors
        // use serde::de::value::SeqDeserializer;
        // visitor.visit_seq(SeqDeserializer::new(self.iter.map(TokenDeserializer::new)))
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
        visitor.visit_unit()
    }

    #[inline]
    fn deserialize_unit<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
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
        V: Visitor<'de>,
    {
        self.deserialize_ignored_any(visitor)
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

    forward_to_deserialize_any!(map struct str string identifier option);
}

impl<'a, 'de: 'a> SeqAccess<'de> for ValueDeserializer<'a, 'de> {
    type Error = Error;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, Self::Error>
    where
        T: DeserializeSeed<'de>,
    {
        match self.iter.next() {
            Some(token) => seed.deserialize(TokenDeserializer::new(token)).map(Some),
            None => Ok(None),
        }
    }
}

impl<'a, 'de: 'a> EnumAccess<'de> for ValueDeserializer<'a, 'de> {
    type Error = Error;
    type Variant = UnitEnumDeserializer;

    fn variant_seed<V>(self, seed: V) -> Result<(V::Value, Self::Variant), Self::Error>
    where
        V: DeserializeSeed<'de>,
    {
        Ok((seed.deserialize(self)?, UnitEnumDeserializer {}))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::abbrev::Abbreviations;
    use crate::de::BibtexDeserializer;
    use crate::reader::ResolvingReader;
    use crate::value::Value;
    use serde::Deserialize;

    #[derive(Debug, Deserialize, PartialEq)]
    enum Tok<'a> {
        #[serde(rename = "Abbrev")]
        A(&'a str),
        #[serde(rename = "Text")]
        T(&'a str),
    }

    macro_rules! assert_de {
        ($input:expr, $expected:expr, $target:tt) => {
            let reader = ResolvingReader::new($input);
            let mut bib_de = BibtexDeserializer::new(reader);
            let deserializer = ValueDeserializer::try_from_de_resolved(&mut bib_de).unwrap();
            assert_eq!(Ok($expected), $target::deserialize(deserializer));
        };
    }

    macro_rules! assert_de_err {
        ($input:expr, $target:tt) => {
            let reader = ResolvingReader::new($input);
            let mut bib_de = BibtexDeserializer::new(reader);
            let deserializer = ValueDeserializer::try_from_de_resolved(&mut bib_de).unwrap();
            assert!($target::deserialize(deserializer).is_err());
        };
    }

    #[test]
    fn test_value_string() {
        assert_de!("  {a} # { b}", "a b".to_string(), String);
        assert_de!(" {a}", "a".to_string(), String);
    }

    #[test]
    fn test_value_seq() {
        assert_de!(
            " {1} # a # {3}",
            vec![Tok::T("1"), Tok::A("a"), Tok::T("3")],
            Vec
        );

        type DoubleToken<'a> = (Tok<'a>, Tok<'a>);
        assert_de!(" {1} # a", (Tok::T("1"), Tok::A("a")), DoubleToken);
    }

    #[test]
    fn test_value_cow() {
        assert_de!("{a} # { b}", Cow::Borrowed("a b").to_owned(), Cow);
        assert_de!("{a}", Cow::Borrowed("a"), Cow);
    }

    #[test]
    fn test_value_str_borrowed() {
        #[derive(Deserialize, PartialEq, Eq, Debug)]
        struct Value<'a>(&'a str);

        assert_de!(" {a}", Value("a"), Value);
        assert_de_err!(" {a} # {b}", Value);
    }

    #[test]
    fn test_value_parsed() {
        // bool
        assert_de!("{tr}\n #{ue}", true, bool);

        // i64
        assert_de!("{0} # \"1\" # 234", 1234, i16);
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

        assert_de!(" {jan}", Month::Jan, Month);
    }

    #[test]
    fn test_unit_struct() {
        #[derive(Deserialize, Debug, PartialEq)]
        struct Unit;

        assert_de!("{} #{}", Unit, Unit);
    }

    #[test]
    fn test_identifier() {
        // As string
        let de = IdentifierDeserializer::new(Identifier::from_str_unchecked("key"));
        assert_eq!(Ok("key".to_string()), String::deserialize(de));

        // As newtype struct
        #[derive(Debug, Deserialize, PartialEq)]
        struct Id(String);

        let de = IdentifierDeserializer::new(Identifier::from_str_unchecked("key"));
        assert_eq!(Id::deserialize(de), Ok(Id("key".to_string())));

        // As newtype with &str
        #[derive(Debug, Deserialize, PartialEq)]
        struct IdRef<'r>(&'r str);

        let de = IdentifierDeserializer::new(Identifier::from_str_unchecked("key"));
        assert_eq!(IdRef::deserialize(de), Ok(IdRef("key")));

        // As enum
        #[derive(Debug, Deserialize, PartialEq)]
        #[serde(rename_all = "lowercase")]
        enum AllowedKeys {
            Key,
            Other,
        }

        let de = IdentifierDeserializer::new(Identifier::from_str_unchecked("key"));
        assert_eq!(AllowedKeys::deserialize(de), Ok(AllowedKeys::Key));
    }

    #[test]
    fn test_token() {
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
    fn test_value_abbrev_expansion() {
        // Test expansion of Abbreviations

        let mut abbrevs = Abbreviations::default();
        abbrevs.insert(
            Identifier::from_str_unchecked("a"),
            Value::from_iter([Token::text_from("1")]),
        );
        abbrevs.insert(
            Identifier::from_str_unchecked("b"),
            Value::from_iter([Token::text_from("2"), Token::text_from("3")]),
        );
        abbrevs.insert(Identifier::from_str_unchecked("c"), Value::default());
        abbrevs.insert(
            Identifier::from_str_unchecked("d"),
            Value::from_iter([Token::text_from("")]),
        );
        abbrevs.insert(
            Identifier::from_str_unchecked("e"),
            Value::from_iter([Token::abbrev_from("b")]),
        );

        macro_rules! assert_value_string {
            ($input:expr, $expected:expr) => {
                let reader = ResolvingReader::new($input);
                let mut bib_de = BibtexDeserializer::new_from_abbrev(reader, abbrevs.clone());
                let deserializer = ValueDeserializer::try_from_de_resolved(&mut bib_de).unwrap();
                let data = String::deserialize(deserializer);
                let expected = $expected.to_string();
                assert_eq!(data, Ok(expected));
            };
        }

        macro_rules! assert_value_fail {
            ($input:expr) => {
                let reader = ResolvingReader::new($input);
                let mut bib_de = BibtexDeserializer::new_from_abbrev(reader, abbrevs.clone());
                let deserializer = ValueDeserializer::try_from_de_resolved(&mut bib_de).unwrap();
                let data = String::deserialize(deserializer);
                assert!(data.is_err());
            };
        }

        macro_rules! assert_value_seq {
            ($input:expr, $expected:expr) => {
                let reader = ResolvingReader::new($input);
                let mut bib_de = BibtexDeserializer::new_from_abbrev(reader, abbrevs.clone());
                let deserializer = ValueDeserializer::try_from_de_resolved(&mut bib_de).unwrap();

                let data: Result<Vec<Tok>, _> = Vec::deserialize(deserializer);
                assert_eq!(data, Ok($expected));
            };
        }

        // basic expansion
        let input = " a";
        assert_value_string!(input, "1");
        assert_value_seq!(input, vec![Tok::T("1")]);

        // characters are inserted in order if they are multiple characters long
        let input = "{0} # a # b";
        assert_value_string!(input, "0123");
        assert_value_seq!(
            input,
            vec![Tok::T("0"), Tok::T("1"), Tok::T("2"), Tok::T("3")]
        );

        // abbreviations referencing other abbreviations are resolved, if
        // the previous abbreviation appeared
        let input = "e";
        assert_value_string!(input, "23");
        assert_value_seq!(input, vec![Tok::T("2"), Tok::T("3")]);

        // lenth 0 abbreviations are skipped...
        let input = "c # a # c";
        assert_value_string!(input, "1");
        // ...and they do not appear in the Token stream
        assert_value_seq!(input, vec![Tok::T("1")]);

        // abbreviations which expand to {} are skiped...
        let input = "{0} # d # {0}";
        assert_value_string!(input, "00");
        // ...but they do appear in the Token stream
        assert_value_seq!(input, vec![Tok::T("0"), Tok::T(""), Tok::T("0")]);

        // use same abbreviation repeatedly
        let input = "b # b # c # {1} # b";
        assert_value_string!(input, "2323123");
        assert_value_seq!(
            input,
            vec![
                Tok::T("2"),
                Tok::T("3"),
                Tok::T("2"),
                Tok::T("3"),
                Tok::T("1"),
                Tok::T("2"),
                Tok::T("3"),
            ]
        );

        // unresolved abbreviations fail, but still appear as raw tokens
        let input = " {} # f # b";
        assert_value_fail!(input);
        assert_value_seq!(
            input,
            vec![Tok::T(""), Tok::A("f"), Tok::T("2"), Tok::T("3"),]
        );
    }

    #[test]
    fn test_value_ownership() {
        // Test that we only take ownership when necessary.

        #[derive(Deserialize, Debug, PartialEq, Eq)]
        struct Val<'r>(#[serde(borrow)] Cow<'r, str>);

        let mut abbrevs = Abbreviations::default();

        abbrevs.insert(
            Identifier::from_str_unchecked("a"),
            Value::from_iter([Token::text_from("")]),
        );
        abbrevs.insert(Identifier::from_str_unchecked("b"), Value::default());
        abbrevs.insert(
            Identifier::from_str_unchecked("c"),
            Value::from_iter([Token::text_from("1")]),
        );

        macro_rules! assert_value_matching {
            ($input:expr, $expected:expr, $cow:pat) => {
                let reader = ResolvingReader::new($input);
                let mut bib_de = BibtexDeserializer::new_from_abbrev(reader, abbrevs.clone());
                let deserializer = ValueDeserializer::try_from_de_resolved(&mut bib_de).unwrap();
                let data = Val::deserialize(deserializer);
                let expected = Val($expected.into());
                assert_eq!(data, Ok(expected));
                assert!(matches!(data, Ok(Val($cow))));
            };
        }

        // separated Token::Text are merged
        assert_value_matching!(" {a} # {b} # {c}", "abc", Cow::Owned(_));
        assert_value_matching!(" {a} # {} # {b}", "ab", Cow::Owned(_));

        // a single Token::Text can be borrowed
        assert_value_matching!(" {a}", "a", Cow::Borrowed(_));

        // empty values also allow owning
        assert_value_matching!(" {} # {abc}", "abc", Cow::Borrowed(_));
        assert_value_matching!(" {} # {abc} # {} # {}", "abc", Cow::Borrowed(_));
        assert_value_matching!(" {abc} # {}", "abc", Cow::Borrowed(_));

        // empty abbreviations can be spliced in without owning
        assert_value_matching!(" a # b # {abc} # a", "abc", Cow::Borrowed(_));

        // can borrow from abbreviations, if possible
        assert_value_matching!(" c", "1", Cow::Borrowed(_));
        assert_value_matching!(" c # c", "11", Cow::Owned(_));
        assert_value_matching!(" {} # c # {} # a # b", "1", Cow::Borrowed(_));
    }
}
