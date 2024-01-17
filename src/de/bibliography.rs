use std::marker::PhantomData;

use serde::de::{self, DeserializeSeed, SeqAccess};
use serde::forward_to_deserialize_any;

use crate::error::Error;
use crate::parse::{BibtexParse, EntryType, MacroDictionary, Token};

use super::entry::{EntryDeserializer, RegularEntryDeserializer};

pub struct Deserializer<'r, R>
where
    R: BibtexParse<'r>,
{
    pub(crate) parser: R,
    pub(crate) macros: MacroDictionary<'r>,
    pub(crate) scratch: Vec<Token<'r>>,
}

/// The top level deserializer for a bibtex file.
///
/// The input is held by the stateful [`StrReader`], which contains all of the methods for
/// incrementing.
///
/// Lifetimes:
/// - `'r`: underlying record
impl<'r, R> Deserializer<'r, R>
where
    R: BibtexParse<'r>,
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

    #[allow(clippy::should_implement_trait)]
    pub fn into_iter<D: de::Deserialize<'r>>(self) -> DeserializeIter<'r, R, D> {
        // We cannot implement Iterator since the Item is not known in advance.
        DeserializeIter {
            de: self,
            _output: PhantomData,
        }
    }

    pub fn into_iter_entry<D: de::Deserialize<'r>>(self) -> DeserializeEntriesIter<'r, R, D> {
        DeserializeEntriesIter {
            de: self,
            _output: PhantomData,
        }
    }

    /// destroy the deserializer, returning the underlying abbreviations
    pub fn finish(self) -> MacroDictionary<'r> {
        let Self { macros, .. } = self;
        macros
    }
}

impl<'a, 'de: 'a, R> de::Deserializer<'de> for &'a mut Deserializer<'de, R>
where
    R: BibtexParse<'de>,
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

impl<'a, 'de: 'a, R> SeqAccess<'de> for &'a mut Deserializer<'de, R>
where
    R: BibtexParse<'de>,
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

pub struct DeserializeIter<'r, R, D>
where
    R: BibtexParse<'r>,
    D: de::Deserialize<'r>,
{
    de: Deserializer<'r, R>,
    _output: PhantomData<D>,
}

impl<'de, R, D> Iterator for DeserializeIter<'de, R, D>
where
    R: BibtexParse<'de>,
    D: de::Deserialize<'de>,
{
    type Item = Result<D, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.de.parser.entry_type() {
            Ok(Some(entry)) => Some(D::deserialize(EntryDeserializer::new(&mut self.de, entry))),
            Ok(None) => None,
            Err(err) => Some(Err(err)),
        }
    }
}

pub struct DeserializeEntriesIter<'r, R, D>
where
    R: BibtexParse<'r>,
    D: de::Deserialize<'r>,
{
    de: Deserializer<'r, R>,
    _output: PhantomData<D>,
}

impl<'de, R, D> Iterator for DeserializeEntriesIter<'de, R, D>
where
    R: BibtexParse<'de>,
    D: de::Deserialize<'de>,
{
    type Item = Result<D, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match self.de.parser.entry_type() {
                Ok(Some(entry)) => match entry {
                    EntryType::Macro => {
                        match self.de.parser.ignore_macro_captured(&mut self.de.macros) {
                            Ok(()) => {}
                            Err(err) => return Some(Err(err)),
                        }
                    }
                    EntryType::Comment => match self.de.parser.ignore_comment() {
                        Ok(()) => {}
                        Err(err) => return Some(Err(err)),
                    },
                    EntryType::Preamble => match self.de.parser.ignore_preamble() {
                        Ok(()) => {}
                        Err(err) => return Some(Err(err)),
                    },
                    EntryType::Regular(entry_type) => {
                        return Some(D::deserialize(RegularEntryDeserializer::new(
                            &mut self.de,
                            entry_type.into_inner(),
                        )))
                    }
                },
                Ok(None) => return None,
                Err(err) => return Some(Err(err)),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parse::{StrReader, Variable};

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
        let mut bib_de = Deserializer::new(reader);

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
        let mut bib_de = Deserializer::new(reader);

        let data: Result<CommentOnlyBib, Error> = CommentOnlyBib::deserialize(&mut bib_de);
        let expected = vec![OnlyComment::Comment(b"com")];
        assert_eq!(data, Ok(expected));
    }

    #[test]
    fn test_string_capturing() {
        let reader = StrReader::new("@string{a = {1}}@string{a = a # a}@string{a = a # a}");
        let mut bib_de = Deserializer::new(reader);

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
        let mut bib_de = Deserializer::new(reader);

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
        let mut bib_de = Deserializer::new(reader);

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
            let mut bib_de = Deserializer::new(reader);
            let data: Result<IgnoredAny, Error> = IgnoredAny::deserialize(&mut bib_de);
            assert!(data.$expect(), "{:?} : {:?}", data, bib_de.parser);

            let reader = StrReader::new($input);
            let mut bib_de = Deserializer::new(reader);
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
