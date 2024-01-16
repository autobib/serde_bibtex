pub mod entry;
pub mod value;

use serde::de::{
    self, value::BorrowedStrDeserializer, DeserializeSeed, EnumAccess, SeqAccess, Unexpected,
    VariantAccess,
};
use serde::forward_to_deserialize_any;

use crate::bib::macros::MacroDictionary;
use crate::bib::token::{EntryType, Token};
use crate::bib::BibtexParser;
use crate::error::Error;
use crate::naming::{
    COMMENT_ENTRY_VARIANT_NAME, MACRO_ENTRY_VARIANT_NAME, PREAMBLE_ENTRY_VARIANT_NAME,
    REGULAR_ENTRY_VARIANT_NAME,
};

use entry::RegularEntryDeserializer;
use value::{KeyValueDeserializer, TextDeserializer, ValueDeserializer};

pub struct BibtexDeserializer<'r, R>
where
    R: BibtexParser<'r>,
{
    parser: R,
    macros: MacroDictionary<'r>,
    scratch: Vec<Token<'r>>,
}

/// The top level deserializer for a bibtex file.
///
/// The input is held by the stateful [`StrReader`], which contains all of the methods for
/// incrementing.
///
/// Lifetimes:
/// - `'r`: underlying record
impl<'r, R> BibtexDeserializer<'r, R>
where
    R: BibtexParser<'r>,
{
    pub fn new(parser: R) -> Self {
        Self {
            parser,
            macros: MacroDictionary::default(),
            scratch: Vec::new(),
        }
    }

    pub fn new_with_macros(parser: R, macros: MacroDictionary<'r>) -> Self {
        Self {
            parser,
            macros,
            scratch: Vec::new(),
        }
    }

    /// destroy the deserializer, returning the underlying abbreviations
    pub fn finish(self) -> MacroDictionary<'r> {
        let Self { macros, .. } = self;
        macros
    }
}

impl<'a, 'de: 'a, R> de::Deserializer<'de> for &'a mut BibtexDeserializer<'de, R>
where
    R: BibtexParser<'de>,
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

    fn deserialize_ignored_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        self.parser.ignore_bibliography()?;
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
    R: BibtexParser<'de>,
{
    type Error = Error;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, Self::Error>
    where
        T: DeserializeSeed<'de>,
    {
        match self.parser.entry_type()? {
            Some(entry) => seed
                .deserialize(EntryDeserializer::new(*self, entry))
                .map(Some),
            None => Ok(None),
        }
    }
}

pub struct EntryDeserializer<'a, 'r, R>
where
    R: BibtexParser<'r>,
{
    de: &'a mut BibtexDeserializer<'r, R>,
    entry_type: EntryType<'r>,
}

impl<'a, 'de: 'a, R> de::Deserializer<'de> for EntryDeserializer<'a, 'de, R>
where
    R: BibtexParser<'de>,
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

impl<'a, 'de: 'a, R> VariantAccess<'de> for EntryDeserializer<'a, 'de, R>
where
    R: BibtexParser<'de>,
{
    type Error = Error;

    fn unit_variant(self) -> Result<(), Self::Error> {
        self.de
            .parser
            .ignore_entry_captured(self.entry_type, &mut self.de.macros)
    }

    fn newtype_variant_seed<T>(self, seed: T) -> Result<T::Value, Self::Error>
    where
        T: DeserializeSeed<'de>,
    {
        match self.entry_type {
            EntryType::Regular(entry_type) => seed.deserialize(RegularEntryDeserializer::new(
                &mut *self.de,
                entry_type.into_inner(),
            )),
            EntryType::Macro => seed.deserialize(MacroRuleDeserializer::new(&mut *self.de)),
            EntryType::Comment => {
                seed.deserialize(TextDeserializer::new(self.de.parser.comment_contents()?))
            }
            EntryType::Preamble => {
                let closing_bracket = self.de.parser.initial()?;
                let val = seed.deserialize(ValueDeserializer::try_from_de_resolved(&mut *self.de)?);
                self.de.parser.terminal(closing_bracket)?;
                val
            }
        }
    }

    fn tuple_variant<V>(self, _len: usize, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        Err(de::Error::invalid_type(
            Unexpected::TupleVariant,
            &"entry as tuple variant",
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
            &"entry as struct variant",
        ))
    }
}

impl<'a, 'de: 'a, R> EnumAccess<'de> for EntryDeserializer<'a, 'de, R>
where
    R: BibtexParser<'de>,
{
    type Error = Error;

    type Variant = Self;

    fn variant_seed<V>(self, seed: V) -> Result<(V::Value, Self::Variant), Self::Error>
    where
        V: DeserializeSeed<'de>,
    {
        let de = match self.entry_type {
            EntryType::Preamble => {
                BorrowedStrDeserializer::<Self::Error>::new(PREAMBLE_ENTRY_VARIANT_NAME)
            }
            EntryType::Comment => BorrowedStrDeserializer::new(COMMENT_ENTRY_VARIANT_NAME),
            EntryType::Macro => BorrowedStrDeserializer::new(MACRO_ENTRY_VARIANT_NAME),
            EntryType::Regular(_) => BorrowedStrDeserializer::new(REGULAR_ENTRY_VARIANT_NAME),
        };
        Ok((seed.deserialize(de)?, self))
    }
}

impl<'a, 'r, R> EntryDeserializer<'a, 'r, R>
where
    R: BibtexParser<'r>,
{
    pub fn new(de: &'a mut BibtexDeserializer<'r, R>, entry_type: EntryType<'r>) -> Self {
        Self { de, entry_type }
    }
}

pub struct MacroRuleDeserializer<'a, 'r, R>
where
    R: BibtexParser<'r>,
{
    de: &'a mut BibtexDeserializer<'r, R>,
}

impl<'a, 'r, R> MacroRuleDeserializer<'a, 'r, R>
where
    R: BibtexParser<'r>,
{
    pub fn new(de: &'a mut BibtexDeserializer<'r, R>) -> Self {
        Self { de }
    }
}

/// Deserialization an abbreviation `@string{key = value}`.
///
/// Note that `@string` has already been matched by [`EntryDeserializer`] and this method
/// deserializes the part `{key = value}`. Note two potentially surprising possibilities:
///
/// 1. The contents can be empty: `{}`.
/// 2. If the contents are non-empty, there can be a trailing comma `{key = value,}`.
///
/// As a result of 1, we deserialize as an `Option`.
impl<'a, 'de: 'a, R> de::Deserializer<'de> for MacroRuleDeserializer<'a, 'de, R>
where
    R: BibtexParser<'de>,
{
    type Error = Error;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        let closing_bracket = self.de.parser.initial()?;
        let key = self.de.parser.macro_variable_opt()?;
        let val = match key {
            Some(var) => {
                self.de.parser.field_sep()?;
                self.de.scratch.clear();
                self.de.parser.value_into(&mut self.de.scratch)?;
                self.de.macros.resolve(&mut self.de.scratch);
                self.de
                    .macros
                    .insert_raw_tokens(var.clone(), self.de.scratch.clone());
                let val = visitor.visit_some(KeyValueDeserializer::new(
                    var.0.into_inner(),
                    &mut self.de.scratch,
                ));
                self.de.parser.comma_opt();
                val
            }
            None => visitor.visit_none(),
        };

        self.de.parser.terminal(closing_bracket)?;
        val
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
    use crate::bib::token::Variable;
    use crate::read::StrReader;

    use serde::de::IgnoredAny;
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
    enum TestEntry<'a> {
        #[serde(borrow)]
        Entry(TestEntryMap<'a>),
        #[serde(borrow)]
        Abbreviation(Option<(&'a str, Vec<Tok<'a>>)>),
        #[serde(borrow)]
        Comment(&'a str),
        #[serde(borrow)]
        Preamble(Vec<Tok<'a>>),
    }

    type TestBib<'a> = Vec<TestEntry<'a>>;

    #[derive(Deserialize, Debug, PartialEq)]
    enum BareEntry {
        Entry,
        Abbreviation,
        Comment,
        Preamble,
    }

    type TypeOnlyBib = Vec<BareEntry>;

    #[test]
    fn test_ignore() {
        let reader = StrReader::new(
            r#"
            @string{}
            @string{u={v}}
            @a{k,a=b}
            @preamble{{a} # b # 1234}
            @b(k)
            @comment(com)
            "#,
        );
        let mut bib_de = BibtexDeserializer::new(reader);

        let data: Result<TypeOnlyBib, Error> = TypeOnlyBib::deserialize(&mut bib_de);
        let expected = vec![
            BareEntry::Abbreviation,
            BareEntry::Abbreviation,
            BareEntry::Entry,
            BareEntry::Preamble,
            BareEntry::Entry,
            BareEntry::Comment,
        ];
        assert_eq!(data, Ok(expected));
    }

    #[test]
    fn test_comment_raw() {
        #[derive(Deserialize, Debug, PartialEq)]
        enum OnlyComment<'a> {
            Entry,
            Abbreviation,
            #[serde(borrow)]
            Comment(&'a [u8]),
            Preamble,
        }

        type CommentOnlyBib<'a> = Vec<OnlyComment<'a>>;

        let reader = StrReader::new(
            r#"
            @comment(com)
            "#,
        );
        let mut bib_de = BibtexDeserializer::new(reader);

        let data: Result<CommentOnlyBib, Error> = CommentOnlyBib::deserialize(&mut bib_de);
        let expected = vec![OnlyComment::Comment(b"com")];
        assert_eq!(data, Ok(expected));
    }

    #[test]
    fn test_string_capturing() {
        let reader = StrReader::new("@string{a = {1}}@string{a = a # a}@string{a = a # a}");
        let mut bib_de = BibtexDeserializer::new(reader);

        let _ = TestBib::deserialize(&mut bib_de).unwrap();
        assert!(
            bib_de
                .macros
                .get(&Variable::from_str_unchecked("a"))
                .unwrap()
                .len()
                == 4
        );
    }

    #[test]
    fn test_string_capturing_ignore() {
        type TypeOnlyBib = Vec<BareEntry>;

        let reader = StrReader::new("@string{a = {1}}@string{a = a # a}@string{a = a # a}");
        let mut bib_de = BibtexDeserializer::new(reader);

        let _ = TypeOnlyBib::deserialize(&mut bib_de).unwrap();
        assert!(
            bib_de
                .macros
                .get(&Variable::from_str_unchecked("a"))
                .unwrap()
                .len()
                == 4
        );
    }

    #[test]
    fn test_entry() {
        let reader = StrReader::new("@string{}@string{u={v}}@a{k,a=b}");
        let mut bib_de = BibtexDeserializer::new(reader);

        let data: Result<TestBib, Error> = TestBib::deserialize(&mut bib_de);
        let mut fields = HashMap::new();
        fields.insert("a", vec![Tok::A("b")]);

        let expected = vec![
            TestEntry::Abbreviation(None),
            TestEntry::Abbreviation(Some(("u", vec![Tok::T("v")]))),
            TestEntry::Entry(TestEntryMap {
                entry_type: "a",
                citation_key: "k",
                fields,
            }),
        ];
        assert_eq!(data, Ok(expected));
    }

    macro_rules! assert_syntax {
        ($input:expr, $expect:ident) => {
            let reader = StrReader::new($input);
            let mut bib_de = BibtexDeserializer::new(reader);
            let data: Result<IgnoredAny, Error> = IgnoredAny::deserialize(&mut bib_de);
            assert!(data.$expect(), "{:?} : {:?}", data, bib_de.parser);

            let reader = StrReader::new($input);
            let mut bib_de = BibtexDeserializer::new(reader);
            let data: Result<TestBib, Error> = TestBib::deserialize(&mut bib_de);
            assert!(data.$expect(), "{:?} : {:?}", data, bib_de.parser);
        };
    }

    #[test]
    fn test_string_syntax() {
        assert_syntax!(r"@string{k=v}", is_ok);
        assert_syntax!(r"@sTring{k=v,}", is_ok);

        assert_syntax!(r"@string()", is_ok);
        assert_syntax!(r"@string(,)", is_err);
        assert_syntax!(r"@string{}", is_ok);
        assert_syntax!(r"@string{,}", is_err);
    }

    #[test]
    fn test_preamble_syntax() {
        assert_syntax!(r"@preamble()", is_err);
        assert_syntax!(r"@preamble{}", is_err);
        assert_syntax!(r"@ pREamble {{any} # a #{allowed}}", is_ok);
        assert_syntax!(r"@preamble({})", is_ok);
        assert_syntax!(r"@preamble( {} # {} # {} )", is_ok);

        assert_syntax!(r"@preamble(", is_err);
        assert_syntax!(r"@preamble)", is_err);
        assert_syntax!(r"@preamble({{})", is_err);
        assert_syntax!(r"@preamble(})", is_err);
    }

    #[test]
    fn test_comment_round_syntax() {
        assert_syntax!(r"@comment(@anything#)", is_ok);
        assert_syntax!(r"@comment({(}))", is_ok);
        assert_syntax!(r"@comment({(})", is_ok);
        assert_syntax!(r"@comment(})", is_err);
    }

    #[test]
    fn test_comment_syntax() {
        assert_syntax!(r"@comment{{}}", is_ok);
        assert_syntax!(r"@comment { @anything#}", is_ok);
        assert_syntax!(r"@coMment {}", is_ok);
        assert_syntax!("@\n CommEnt  { }", is_ok);

        assert_syntax!(r"@comment({)", is_err);
    }

    #[test]
    fn test_regular_entry_syntax() {
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
              author ={A🍄th}
                #  
                {or}
                ,title =
              "Tit🍄e" # 🍄
              }"#,
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

        // err: unicode in field keys
        assert_syntax!(r"@article{k,auth🍄={v}}", is_err);
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
        // err: extra @ chars
        assert_syntax!(r#"@ @ @{k,t=v}"#, is_err);

        // opening and closing brackets must match
        assert_syntax!("@a(k}", is_err);
        assert_syntax!("@a{k)", is_err);
        assert_syntax!("@a{k}", is_ok);
        assert_syntax!("@a(k)", is_ok);
    }
}
