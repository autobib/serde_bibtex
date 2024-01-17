use std::borrow::Cow;

use serde::de::{
    self,
    value::{BorrowedStrDeserializer, CowStrDeserializer},
    DeserializeSeed, EnumAccess, SeqAccess, Unexpected, VariantAccess, Visitor,
};
use serde::forward_to_deserialize_any;

use crate::error::Error;
use crate::naming::{MACRO_TOKEN_VARIANT_NAME, TEXT_TOKEN_VARIANT_NAME};
use crate::parse::{BibtexParse, Text, Token};

use super::Deserializer;

pub struct KeyValueDeserializer<'a, 'r> {
    k: Option<Cow<'r, str>>,
    tokens: &'a mut Vec<Token<'r>>,
    complete: bool,
}

impl<'a, 'r> KeyValueDeserializer<'a, 'r> {
    pub fn new(k: Cow<'r, str>, tokens: &'a mut Vec<Token<'r>>) -> Self {
        Self {
            k: Some(k),
            tokens,
            complete: false,
        }
    }
}

impl<'a, 'de: 'a> de::Deserializer<'de> for KeyValueDeserializer<'a, 'de> {
    type Error = Error;

    fn deserialize_any<V>(mut self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_seq(&mut self)
    }

    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
        bytes byte_buf option unit unit_struct newtype_struct seq tuple
        tuple_struct map struct enum identifier ignored_any
    }
}

impl<'a, 'de: 'a> SeqAccess<'de> for KeyValueDeserializer<'a, 'de> {
    type Error = Error;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, Self::Error>
    where
        T: DeserializeSeed<'de>,
    {
        match (self.k.take(), self.complete) {
            (Some(cow), false) => seed.deserialize(BorrowCowDeserializer::new(cow)).map(Some),
            (None, false) => {
                self.complete = true;
                seed.deserialize(ValueDeserializer::new(self.tokens))
                    .map(Some)
            }
            _ => Ok(None),
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

#[derive(Debug, Clone)]
pub struct BorrowCowDeserializer<'r> {
    cow: Cow<'r, str>,
}

impl<'r> BorrowCowDeserializer<'r> {
    pub fn new(cow: Cow<'r, str>) -> Self {
        Self { cow }
    }
}

impl<'de> de::Deserializer<'de> for BorrowCowDeserializer<'de> {
    type Error = Error;

    #[inline]
    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self.cow {
            Cow::Owned(s) => visitor.visit_string(s),
            Cow::Borrowed(s) => visitor.visit_borrowed_str(s),
        }
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
        visitor.visit_enum(CowStrDeserializer::new(self.cow))
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
            Token::Macro(var) => seed.deserialize(BorrowCowDeserializer::new(var.0.into_inner())),
            Token::Text(text) => seed.deserialize(TextDeserializer::new(text)),
        }
    }

    fn tuple_variant<V>(self, _len: usize, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        Err(de::Error::invalid_type(
            Unexpected::TupleVariant,
            &"token as tuple variant",
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
            &"token as struct variant",
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
            Token::Macro(_) => BorrowedStrDeserializer::new(MACRO_TOKEN_VARIANT_NAME),
            Token::Text(_) => BorrowedStrDeserializer::new(TEXT_TOKEN_VARIANT_NAME),
        };
        Ok((seed.deserialize(de)?, self))
    }
}

macro_rules! as_cow_impl {
    ($fname:ident, $target:ty, $push:ident, $null:expr) => {
        fn $fname(&mut self) -> Result<Cow<'r, $target>, Error> {
            let mut init = loop {
                match self.iter.next() {
                    Some(token) => {
                        let cow: Cow<'r, $target> = token.try_into()?;
                        if cow.len() > 0 {
                            break cow;
                        }
                    }
                    None => return Ok(Cow::Borrowed($null)),
                }
            };

            for token in self.iter.by_ref() {
                let cow: Cow<'r, $target> = token.try_into()?;
                if cow.len() > 0 {
                    init.to_mut().$push(&cow)
                }
            }
            Ok(init)
        }
    };
}

#[derive(Debug)]
pub struct ValueDeserializer<'a, 'r> {
    iter: std::vec::Drain<'a, Token<'r>>,
}

impl<'a, 'r> ValueDeserializer<'a, 'r> {
    pub fn new(scratch: &'a mut Vec<Token<'r>>) -> Self {
        Self {
            iter: scratch.drain(..),
        }
    }

    /// Create a new value from the tokens after resolving macros.
    pub(crate) fn try_from_de_resolved<R>(de: &'a mut Deserializer<'r, R>) -> Result<Self, Error>
    where
        R: BibtexParse<'r>,
    {
        de.parser.value_into(&mut de.scratch)?;
        de.macros.resolve(&mut de.scratch);
        Ok(Self {
            iter: de.scratch.drain(..),
        })
    }

    as_cow_impl!(as_cow_str, str, push_str, "");

    as_cow_impl!(as_cow_bytes, [u8], extend_from_slice, b"");

    fn as_char(&mut self) -> Result<char, Error> {
        let mut found_char: Option<char> = None;

        for token in self.iter.by_ref() {
            let cow: Cow<'r, str> = token.try_into()?;
            for char in cow.chars() {
                if found_char.is_some() {
                    return Err(Error::TooManyChars);
                } else {
                    found_char = Some(char);
                }
            }
        }

        found_char.ok_or(Error::NoChars)
    }
}

impl<'a, 'de: 'a> de::Deserializer<'de> for ValueDeserializer<'a, 'de> {
    type Error = Error;

    fn deserialize_any<V>(mut self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self.as_cow_str()? {
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

    fn deserialize_bytes<V>(mut self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self.as_cow_bytes()? {
            Cow::Borrowed(b) => visitor.visit_borrowed_bytes(b),
            Cow::Owned(b) => visitor.visit_byte_buf(b),
        }
    }

    fn deserialize_byte_buf<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_bytes(visitor)
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

    forward_to_deserialize_any!(
        bool i8 i16 i32 i64 u8 u16 u32 u64 f32 f64
        map struct str string identifier option);
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

pub struct TextDeserializer<'r> {
    text: Text<'r>,
}

impl<'r> TextDeserializer<'r> {
    pub fn new(text: Text<'r>) -> Self {
        Self { text }
    }
}

impl<'de> de::Deserializer<'de> for TextDeserializer<'de> {
    type Error = Error;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        match self.text.into_cow_str()? {
            Cow::Borrowed(s) => visitor.visit_borrowed_str(s),
            Cow::Owned(s) => visitor.visit_string(s),
        }
    }

    fn deserialize_bytes<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self.text.into_bytes() {
            Cow::Borrowed(b) => visitor.visit_borrowed_bytes(b),
            Cow::Owned(b) => visitor.visit_byte_buf(b),
        }
    }

    fn deserialize_byte_buf<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_bytes(visitor)
    }

    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
        option unit unit_struct newtype_struct seq tuple
        tuple_struct map struct enum identifier ignored_any
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::de::Deserializer;
    use crate::parse::MacroDictionary;
    use crate::parse::StrReader;
    use crate::parse::Variable;
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
            let reader = StrReader::new($input);
            let mut bib_de = Deserializer::new(reader);
            let deserializer = ValueDeserializer::try_from_de_resolved(&mut bib_de).unwrap();
            assert_eq!(Ok($expected), $target::deserialize(deserializer));
        };
    }

    macro_rules! assert_de_err {
        ($input:expr, $target:tt) => {
            let reader = StrReader::new($input);
            let mut bib_de = Deserializer::new(reader);
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
    fn test_value_cow_bytes() {
        #[derive(Debug, Deserialize, PartialEq)]
        enum TokBytes<'a> {
            #[serde(rename = "Abbrev")]
            A(&'a str),
            #[serde(rename = "Text")]
            T(&'a [u8]),
        }

        let reader = StrReader::new("{a} # { b} # C");
        let mut bib_de = Deserializer::new(reader);
        let deserializer = ValueDeserializer::try_from_de_resolved(&mut bib_de).unwrap();
        assert_eq!(
            Ok(vec![
                TokBytes::T(b"a"),
                TokBytes::T(b" b"),
                TokBytes::A("C")
            ]),
            Vec::<TokBytes>::deserialize(deserializer)
        );

        let reader = StrReader::new("{u} # {v}");
        let mut bib_de = Deserializer::new(reader);
        let deserializer = ValueDeserializer::try_from_de_resolved(&mut bib_de).unwrap();
        assert_eq!(
            vec![b'u', b'v'],
            serde_bytes::ByteBuf::deserialize(deserializer)
                .unwrap()
                .into_vec()
        );
    }

    #[test]
    fn test_value_str_borrowed() {
        #[derive(Deserialize, PartialEq, Eq, Debug)]
        struct Value<'a>(&'a str);

        assert_de!(" {a}", Value("a"), Value);
        assert_de_err!(" {a} # {b}", Value);
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
    fn test_text() {
        let de = TextDeserializer::new(Text::Str(Cow::Borrowed("inside")));
        let res = String::deserialize(de).unwrap();
        assert_eq!(res, "inside".to_string());

        let de = TextDeserializer::new(Text::Raw(Cow::Borrowed(b"inside")));
        let res = String::deserialize(de).unwrap();
        assert_eq!(res, "inside".to_string());
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

        let de = TokenDeserializer::new(Token::macro_from("key"));
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

        let de = TokenDeserializer::new(Token::macro_from("key"));
        assert_eq!(ReToken::deserialize(de), Ok(ReToken::A(I("key"))));

        let de = TokenDeserializer::new(Token::text_from("key"));
        let data = ReToken::deserialize(de).unwrap();
        assert_eq!(data, ReToken::T(Cow::Borrowed("key")));
        assert!(matches!(data, ReToken::T(Cow::Borrowed(_))));
    }

    #[test]
    fn test_value_abbrev_expansion() {
        // Test expansion of Abbreviations

        let mut abbrevs = MacroDictionary::default();
        abbrevs.insert(
            Variable::from_str_unchecked("a"),
            vec![Token::text_from("1")],
        );
        abbrevs.insert(
            Variable::from_str_unchecked("b"),
            vec![Token::text_from("2"), Token::text_from("3")],
        );
        abbrevs.insert(Variable::from_str_unchecked("c"), Vec::default());
        abbrevs.insert(
            Variable::from_str_unchecked("d"),
            vec![Token::text_from("")],
        );
        abbrevs.insert(
            Variable::from_str_unchecked("e"),
            vec![Token::macro_from("b")],
        );

        macro_rules! assert_value_string {
            ($input:expr, $expected:expr) => {
                let reader = StrReader::new($input);
                let mut bib_de = Deserializer::new_with_macros(reader, abbrevs.clone());
                let deserializer = ValueDeserializer::try_from_de_resolved(&mut bib_de).unwrap();
                let data = String::deserialize(deserializer);
                let expected = $expected.to_string();
                assert_eq!(data, Ok(expected));
            };
        }

        macro_rules! assert_value_fail {
            ($input:expr) => {
                let reader = StrReader::new($input);
                let mut bib_de = Deserializer::new_with_macros(reader, abbrevs.clone());
                let deserializer = ValueDeserializer::try_from_de_resolved(&mut bib_de).unwrap();
                let data = String::deserialize(deserializer);
                assert!(data.is_err());
            };
        }

        macro_rules! assert_value_seq {
            ($input:expr, $expected:expr) => {
                let reader = StrReader::new($input);
                let mut bib_de = Deserializer::new_with_macros(reader, abbrevs.clone());
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

        let mut abbrevs = MacroDictionary::default();

        abbrevs.insert(
            Variable::from_str_unchecked("a"),
            vec![Token::text_from("")],
        );
        abbrevs.insert(Variable::from_str_unchecked("b"), Vec::new());
        abbrevs.insert(
            Variable::from_str_unchecked("c"),
            vec![Token::text_from("1")],
        );

        macro_rules! assert_value_matching {
            ($input:expr, $expected:expr, $cow:pat) => {
                let reader = StrReader::new($input);
                let mut bib_de = Deserializer::new_with_macros(reader, abbrevs.clone());
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
