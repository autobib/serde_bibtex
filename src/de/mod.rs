pub mod entry;
pub mod value;

use std::marker::PhantomData;

use serde::de::{self, DeserializeSeed, EnumAccess, SeqAccess, Unexpected, VariantAccess};
use serde::forward_to_deserialize_any;

use crate::abbrev::Abbreviations;
use crate::error::Error;
use crate::parse::core::ChunkType;
use crate::parse::BibtexReader;
use crate::value::Token;

use entry::EntryDeserializer;
use value::{IdentifierDeserializer, KeyValueDeserializer};

pub struct BibtexDeserializer<'r, R>
where
    R: BibtexReader<'r>,
{
    reader: R,
    abbrev: Abbreviations<'r>,
    scratch: Vec<Token<'r>>,
    _marker: PhantomData<&'r ()>,
}

/// The top level deserializer for a bibtex file.
///
/// The input is held by the stateful [`ResolvingReader`], which contains all of the methods for
/// incrementing.
///
/// Lifetimes:
/// - `'r`: underlying record
impl<'r, R> BibtexDeserializer<'r, R>
where
    R: BibtexReader<'r>,
{
    pub fn new(reader: R) -> Self {
        Self {
            reader,
            abbrev: Abbreviations::default(),
            scratch: Vec::new(),
            _marker: PhantomData,
        }
    }

    pub fn new_from_abbrev(reader: R, abbrev: Abbreviations<'r>) -> Self {
        Self {
            reader,
            abbrev,
            scratch: Vec::new(),
            _marker: PhantomData,
        }
    }

    /// destroy the deserializer, returning the underlying abbreviations
    pub fn finish(self) -> Abbreviations<'r> {
        let Self { abbrev, .. } = self;
        abbrev
    }
}

impl<'a, 'de: 'a, R> de::Deserializer<'de> for &'a mut BibtexDeserializer<'de, R>
where
    R: BibtexReader<'de>,
{
    type Error = Error;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_seq(self)
    }

    #[inline]
    fn deserialize_unit<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        // TODO: test
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
        // TODO: test
        self.deserialize_ignored_any(visitor)
    }

    fn deserialize_ignored_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        self.reader.ignore_bibliography()?;
        visitor.visit_unit()
    }

    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
        bytes byte_buf option newtype_struct seq tuple
        tuple_struct map struct enum identifier
    }
}

impl<'a, 'de: 'a, R> SeqAccess<'de> for &'a mut BibtexDeserializer<'de, R>
where
    R: BibtexReader<'de>,
{
    type Error = Error;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, Self::Error>
    where
        T: DeserializeSeed<'de>,
    {
        match self.reader.take_chunk_type()? {
            Some(chunk) => seed
                .deserialize(ChunkDeserializer::new(&mut *self, chunk))
                .map(Some),
            None => Ok(None),
        }
    }
}

pub struct ChunkDeserializer<'a, 'r, R>
where
    R: BibtexReader<'r>,
{
    de: &'a mut BibtexDeserializer<'r, R>,
    chunk: ChunkType<'r>,
}

impl<'a, 'de: 'a, R> de::Deserializer<'de> for ChunkDeserializer<'a, 'de, R>
where
    R: BibtexReader<'de>,
{
    type Error = Error;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_enum(self)
    }

    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
        bytes byte_buf option unit unit_struct newtype_struct seq tuple
        tuple_struct map struct enum identifier ignored_any
    }
}

impl<'a, 'de: 'a, R> VariantAccess<'de> for ChunkDeserializer<'a, 'de, R>
where
    R: BibtexReader<'de>,
{
    type Error = Error;

    fn unit_variant(self) -> Result<(), Self::Error> {
        self.de.reader.ignore_chunk(self.chunk)
    }

    fn newtype_variant_seed<T>(self, seed: T) -> Result<T::Value, Self::Error>
    where
        T: DeserializeSeed<'de>,
    {
        match self.chunk {
            ChunkType::Entry(entry_type) => {
                seed.deserialize(EntryDeserializer::new(&mut *self.de, entry_type))
            }
            ChunkType::Abbreviation => {
                seed.deserialize(AbbreviationDeserializer::new(&mut *self.de))
            }
            ChunkType::Comment => seed.deserialize(BracketedTextDeserializer::new(&mut *self.de)),
            ChunkType::Preamble => seed.deserialize(BracketedTextDeserializer::new(&mut *self.de)),
        }
    }

    fn tuple_variant<V>(self, _len: usize, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        Err(de::Error::invalid_type(
            Unexpected::TupleVariant,
            &"chunk as tuple variant",
        ))
    }

    fn struct_variant<V>(
        self,
        _fields: &'static [&'static str],
        _visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        Err(de::Error::invalid_type(
            Unexpected::StructVariant,
            &"chunk as struct variant",
        ))
    }
}

impl<'a, 'de: 'a, R> EnumAccess<'de> for ChunkDeserializer<'a, 'de, R>
where
    R: BibtexReader<'de>,
{
    type Error = Error;

    type Variant = Self;

    fn variant_seed<V>(self, seed: V) -> Result<(V::Value, Self::Variant), Self::Error>
    where
        V: DeserializeSeed<'de>,
    {
        let de = match self.chunk {
            ChunkType::Preamble => IdentifierDeserializer::new_from_str("Preamble"),
            ChunkType::Comment => IdentifierDeserializer::new_from_str("Comment"),
            ChunkType::Abbreviation => IdentifierDeserializer::new_from_str("Abbreviation"),
            ChunkType::Entry(_) => IdentifierDeserializer::new_from_str("Entry"),
        };
        Ok((seed.deserialize(de)?, self))
    }
}

impl<'a, 'r, R> ChunkDeserializer<'a, 'r, R>
where
    R: BibtexReader<'r>,
{
    pub fn new(de: &'a mut BibtexDeserializer<'r, R>, chunk: ChunkType<'r>) -> Self {
        Self { de, chunk }
    }
}

pub struct AbbreviationDeserializer<'a, 'r, R>
where
    R: BibtexReader<'r>,
{
    de: &'a mut BibtexDeserializer<'r, R>,
}

impl<'a, 'r, R> AbbreviationDeserializer<'a, 'r, R>
where
    R: BibtexReader<'r>,
{
    pub fn new(de: &'a mut BibtexDeserializer<'r, R>) -> Self {
        Self { de }
    }
}

/// Deserialization an abbreviation `@string{key = value}`.
///
/// Note that `@string` has already been matched by [`ChunkDeserializer`] and this method
/// deserializes the part `{key = value}`. Note two potentially surprising possibilities:
///
/// 1. The contents can be empty: `{}`.
/// 2. If the contents are non-empty, there can be a trailing comma `{key = value,}`.
///
/// As a result of 1, we deserialize as an `Option`.
impl<'a, 'de: 'a, R> de::Deserializer<'de> for AbbreviationDeserializer<'a, 'de, R>
where
    R: BibtexReader<'de>,
{
    type Error = Error;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        let closing_bracket = self.de.reader.take_initial()?;
        let key = self.de.reader.take_field_key()?;
        let val = match key {
            Some(identifier) => {
                self.de.reader.ignore_field_sep()?;
                visitor.visit_some(KeyValueDeserializer::new(&mut *self.de, identifier))
            }
            None => visitor.visit_none(),
        };
        match key {
            Some(_) => self.de.reader.opt_comma()?,
            _ => {}
        };
        self.de.reader.take_terminal(closing_bracket)?;
        val
    }

    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
        bytes byte_buf option unit unit_struct newtype_struct seq tuple
        tuple_struct map struct enum identifier ignored_any
    }
}

pub struct BracketedTextDeserializer<'a, 'r, R>
where
    R: BibtexReader<'r>,
{
    de: &'a mut BibtexDeserializer<'r, R>,
}

impl<'a, 'r, R> BracketedTextDeserializer<'a, 'r, R>
where
    R: BibtexReader<'r>,
{
    pub fn new(de: &'a mut BibtexDeserializer<'r, R>) -> Self {
        Self { de }
    }
}

impl<'a, 'de: 'a, R> de::Deserializer<'de> for BracketedTextDeserializer<'a, 'de, R>
where
    R: BibtexReader<'de>,
{
    type Error = Error;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_borrowed_str(self.de.reader.take_bracketed_text()?)
    }

    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
        bytes byte_buf option unit unit_struct newtype_struct seq tuple
        tuple_struct map struct enum identifier ignored_any
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::reader::ResolvingReader;
    use serde::Deserialize;

    use std::collections::HashMap;

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

    #[derive(Deserialize, Debug, PartialEq)]
    enum TestChunk<'a> {
        #[serde(borrow)]
        Entry(TestEntryMap<'a>),
        #[serde(borrow)]
        Abbreviation(Option<(&'a str, Vec<Tok<'a>>)>),
        #[serde(borrow)]
        Comment(&'a str),
        #[serde(borrow)]
        Preamble(&'a str),
    }

    #[test]
    fn test_abbreviation() {
        // test AbbreviationDeserializer
    }

    #[test]
    fn test_ignore() {
        #[derive(Deserialize, Debug, PartialEq)]
        enum BareChunk {
            Entry,
            Abbreviation,
            Comment,
            Preamble,
        }

        type TypeOnlyBib = Vec<BareChunk>;

        let reader = ResolvingReader::new(
            r#"
            @string{}
            @string{u={v}}
            @a{k,a=b}
            @preamble{@r#}
            @b(k)
            @comment(com)
            "#,
        );
        let mut bib_de = BibtexDeserializer::new(reader);

        let data: Result<TypeOnlyBib, Error> = TypeOnlyBib::deserialize(&mut bib_de);
        let expected = vec![
            BareChunk::Abbreviation,
            BareChunk::Abbreviation,
            BareChunk::Entry,
            BareChunk::Preamble,
            BareChunk::Entry,
            BareChunk::Comment,
        ];
        assert_eq!(data, Ok(expected));
    }

    type TestBib<'a> = Vec<TestChunk<'a>>;

    #[test]
    fn test_entry_chunk() {
        let reader = ResolvingReader::new("@string{}@string{u={v}}@a{k,a=b}");
        let mut bib_de = BibtexDeserializer::new(reader);

        let data: Result<TestBib, Error> = TestBib::deserialize(&mut bib_de);
        let mut fields = HashMap::new();
        fields.insert("a", vec![Tok::A("b")]);

        let expected = vec![
            TestChunk::Abbreviation(None),
            TestChunk::Abbreviation(Some(("u", vec![Tok::T("v")]))),
            TestChunk::Entry(TestEntryMap {
                entry_type: "a",
                citation_key: "k",
                fields,
            }),
        ];
        assert_eq!(data, Ok(expected));

        // TODO more tests
    }

    use serde::de::IgnoredAny;

    macro_rules! assert_syntax {
        ($input:expr, $expect:ident) => {
            let reader = ResolvingReader::new($input);
            let mut bib_de = BibtexDeserializer::new(reader);
            let data: Result<IgnoredAny, Error> = IgnoredAny::deserialize(&mut bib_de);
            assert!(data.$expect(), "{:?} : {:?}", data, bib_de.reader);

            let reader = ResolvingReader::new($input);
            let mut bib_de = BibtexDeserializer::new(reader);
            let data: Result<TestBib, Error> = TestBib::deserialize(&mut bib_de);
            assert!(data.$expect(), "{:?} : {:?}", data, bib_de.reader);
        };
    }

    #[test]
    fn test_string_syntax() {
        assert_syntax!(r"@string{k=v}", is_ok);
        assert_syntax!(r"@sTring{k=v,}", is_ok);
        assert_syntax!(r"@sTring{'a k=v,}", is_ok);

        assert_syntax!(r"@string()", is_ok);
        assert_syntax!(r"@string(,)", is_err);
        assert_syntax!(r"@string{}", is_ok);
        assert_syntax!(r"@string{,}", is_err);
    }

    #[test]
    fn test_preamble_syntax() {
        assert_syntax!(r"@preamble({})", is_ok);
        assert_syntax!(r"@preamble()", is_ok);
        assert_syntax!(r"@preamble{}", is_ok);
        assert_syntax!(r"@'5 '% pREamble {@any#}", is_ok);

        assert_syntax!(r"@preamble(", is_err);
        assert_syntax!(r"@preamble)", is_err);
        assert_syntax!(r"@preamble({{})", is_err);
        assert_syntax!(r"@preamble(})", is_err);
    }
    #[test]
    fn test_comment_syntax() {
        assert_syntax!(r"@comment{{}}", is_ok);
        assert_syntax!(r"@comment({})", is_ok);
        assert_syntax!(r"@comment(@anything#)", is_ok);
        assert_syntax!(r"@comment { @anything#}", is_ok);
        assert_syntax!(r"@coMment {}", is_ok);
        assert_syntax!("@\n'e CommEnt  { }", is_ok);

        assert_syntax!(r"@comment({)", is_err);
    }

    #[test]
    fn test_entry_syntax() {
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
        // identifiers can have weird chars, e.g. `@🍄`
        assert_syntax!(r#"@ @🍄{k,t=v}"#, is_ok);
        // no @, so it is junk
        assert_syntax!(r#"a{k,t=v}"#, is_ok);

        // err: multiple trailing comma
        assert_syntax!(r#"@a{k,,}"#, is_err);
        // err: missing field value
        assert_syntax!(r#"@a{k,t=,}"#, is_err);
        // err: missing citation key
        assert_syntax!(r#"@a{,t=v}"#, is_err);
        // err: invalid char in citation key
        assert_syntax!(r#"@a{t=b}"#, is_err);
        assert_syntax!(r#"@a{t#b}"#, is_err);
        assert_syntax!(r#"@a{t\b}"#, is_err);
        // err: junk
        assert_syntax!(r#"@ @ @{k,t=v}"#, is_err);

        // opening and closing brackets must match
        assert_syntax!("@a(k}", is_err);
        assert_syntax!("@a{k)", is_err);
        assert_syntax!("@a{k}", is_ok);
        assert_syntax!("@a(k)", is_ok);
    }
}
