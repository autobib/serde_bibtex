use std::marker::PhantomData;

use serde::de::{self, DeserializeSeed, SeqAccess};
use serde::forward_to_deserialize_any;

use crate::{
    SliceReader, StrReader,
    error::{Error, Result},
    parse::{BibtexParse, MacroDictionary},
    token::{EntryType, Token},
};

use super::entry::{EntryDeserializer, RegularEntryDeserializer};

/// The core `.bib` deserializer.
///
/// Construct using one of the following methods:
/// - [`Deserializer::from_str`]
/// - [`Deserializer::from_str_with_macros`]
/// - [`Deserializer::from_slice`]
/// - [`Deserializer::from_slice_with_macros`]
///
/// The type parameter `R` is the input type from which you are deserializing. If you construct a
/// [`Deserializer`] using one of the above methods, the type will be inferred automatically.
pub struct Deserializer<'r, R> {
    pub(crate) parser: R,
    pub(crate) macros: MacroDictionary<&'r str, &'r [u8]>,
    pub(crate) scratch: Vec<Token<&'r str, &'r [u8]>>,
}

impl<'r> Deserializer<'r, StrReader<'r>> {
    /// Construct a deserialier from a `&str`.
    #[allow(clippy::should_implement_trait)]
    pub fn from_str(s: &'r str) -> Self {
        Self::new(StrReader::new(s))
    }

    /// Construct a deserialier from a `&str` and the provided [`MacroDictionary`].
    pub fn from_str_with_macros(s: &'r str, macros: MacroDictionary<&'r str, &'r [u8]>) -> Self {
        Self::new_with_macros(StrReader::new(s), macros)
    }
}

impl<'r> Deserializer<'r, SliceReader<'r>> {
    /// Construct a deserialier from a `&[u8]`.
    pub fn from_slice(s: &'r [u8]) -> Self {
        Self::new(SliceReader::new(s))
    }

    /// Construct a deserialier from a `&[u8]` and the provided [`MacroDictionary`].
    pub fn from_slice_with_macros(s: &'r [u8], macros: MacroDictionary<&'r str, &'r [u8]>) -> Self {
        Self::new_with_macros(SliceReader::new(s), macros)
    }
}

impl<'r, R> Deserializer<'r, R>
where
    R: BibtexParse<'r>,
{
    /// Construct a new [`Deserializer`] from any [`BibtexParse`] implementation.
    pub(crate) fn new(parser: R) -> Self {
        Self {
            parser,
            macros: MacroDictionary::default(),
            scratch: Vec::new(),
        }
    }

    /// Construct a new [`Deserializer`] from any [`BibtexParse`] implementation and pre-defined
    /// macros in [`MacroDictionary`].
    pub(crate) fn new_with_macros(parser: R, macros: MacroDictionary<&'r str, &'r [u8]>) -> Self {
        Self {
            parser,
            macros,
            scratch: Vec::new(),
        }
    }

    /// Returns an iterator over the entries in the underlying BibTeX data.
    ///
    /// Note that a [`Deserializer`] does not implement [`IntoIterator`] because of lifetime
    /// restrictions.
    #[allow(clippy::should_implement_trait)]
    pub fn into_iter<D: de::Deserialize<'r>>(self) -> DeserializeIter<'r, R, D> {
        // We cannot implement Iterator since the Item is not known in advance.
        DeserializeIter {
            de: self,
            _output: PhantomData,
        }
    }

    /// Returns an iterator over the regular entries of the underlying BibTeX data, ignoring
    /// entries which are not regular entries but automatically capturing and expanding macros.
    pub fn into_iter_regular_entry<D: de::Deserialize<'r>>(
        self,
    ) -> DeserializeRegularEntryIter<'r, R, D> {
        DeserializeRegularEntryIter {
            de: self,
            _output: PhantomData,
        }
    }

    /// Drop the deserializer, returning the underlying [`MacroDictionary`].
    pub fn finish(self) -> MacroDictionary<&'r str, &'r [u8]> {
        let Self { macros, .. } = self;
        macros
    }
}

impl<'de, R> de::Deserializer<'de> for &mut Deserializer<'de, R>
where
    R: BibtexParse<'de>,
{
    type Error = Error;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_seq(self)
    }

    #[inline]
    fn deserialize_unit<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        self.deserialize_ignored_any(visitor)
    }

    #[inline]
    fn deserialize_unit_struct<V>(self, _name: &'static str, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        self.deserialize_ignored_any(visitor)
    }

    fn deserialize_ignored_any<V>(self, visitor: V) -> Result<V::Value>
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

impl<'de, R> SeqAccess<'de> for &mut Deserializer<'de, R>
where
    R: BibtexParse<'de>,
{
    type Error = Error;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>>
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

/// A lazy iterator over BibTeX entries.
///
/// The recommended way to construct this struct is to use the [`Deserializer::into_iter`] method.
/// To only iterate over regular entries, see [`DeserializeRegularEntryIter`].
/// To deserialize into an arbitrary wrapper type, see [`Deserializer`].
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
    type Item = Result<D>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.de.parser.entry_type() {
            Ok(Some(entry)) => Some(D::deserialize(EntryDeserializer::new(&mut self.de, entry))),
            Ok(None) => None,
            Err(err) => Some(Err(err)),
        }
    }
}

/// A lazy iterator over BibTeX regular entries.
///
/// Note that macros are automatically captured and expanded, when possible.
///
/// The recommended way to construct this struct is to use the
/// [`Deserializer::into_iter_regular_entry`] method.
/// To also iterate over preamble, comment, or macro entries, see [`DeserializeIter`].
/// To deserialize into an arbitrary wrapper type, see [`Deserializer`].
pub struct DeserializeRegularEntryIter<'r, R, D>
where
    R: BibtexParse<'r>,
    D: de::Deserialize<'r>,
{
    de: Deserializer<'r, R>,
    _output: PhantomData<D>,
}

impl<'de, R, D> Iterator for DeserializeRegularEntryIter<'de, R, D>
where
    R: BibtexParse<'de>,
    D: de::Deserialize<'de>,
{
    type Item = Result<D>;

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
                        )));
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
    use crate::{
        parse::StrReader,
        syntax::{BibtexParser, Rule},
        token::Variable,
    };

    use pest::Parser;
    use serde::Deserialize;
    use serde::de::IgnoredAny;

    use std::collections::HashMap;

    // Anonymous field names and flexible receiver type
    #[derive(Debug, Deserialize, PartialEq)]
    enum Tok<'a> {
        #[serde(rename = "Variable")]
        V(&'a str),
        #[serde(rename = "Text")]
        T(&'a str),
    }

    #[derive(Deserialize, Debug, PartialEq)]
    struct TestEntryMap<'a> {
        entry_type: &'a str,
        entry_key: &'a str,
        #[serde(borrow)]
        fields: HashMap<&'a str, Vec<Tok<'a>>>,
    }

    #[derive(Deserialize, Debug, PartialEq)]
    enum TestEntry<'a> {
        #[serde(borrow)]
        Regular(TestEntryMap<'a>),
        #[serde(borrow)]
        Macro(Option<(&'a str, Vec<Tok<'a>>)>),
        #[serde(borrow)]
        Comment(&'a str),
        #[serde(borrow)]
        Preamble(Vec<Tok<'a>>),
    }

    type TestBib<'a> = Vec<TestEntry<'a>>;

    #[derive(Deserialize, Debug, PartialEq)]
    enum TestEntryCaptureMacro<'a> {
        #[serde(borrow)]
        Entry(TestEntryMap<'a>),
        Macro,
        #[serde(borrow)]
        Comment(&'a str),
        #[serde(borrow)]
        Preamble(Vec<Tok<'a>>),
    }

    type TestBibCaptureMacro<'a> = Vec<TestEntryCaptureMacro<'a>>;

    #[derive(Deserialize, Debug, PartialEq)]
    enum TestEntryIgnoreMacro<'a> {
        #[serde(borrow)]
        Entry(TestEntryMap<'a>),
        Macro(IgnoredAny),
        #[serde(borrow)]
        Comment(&'a str),
        #[serde(borrow)]
        Preamble(Vec<Tok<'a>>),
    }

    type TestBibIgnoreMacro<'a> = Vec<TestEntryIgnoreMacro<'a>>;

    #[derive(Deserialize, Debug, PartialEq)]
    enum BareEntry {
        Regular,
        Macro,
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

        let data: Result<TypeOnlyBib> = TypeOnlyBib::deserialize(&mut bib_de);
        let expected = vec![
            BareEntry::Macro,
            BareEntry::Macro,
            BareEntry::Regular,
            BareEntry::Preamble,
            BareEntry::Regular,
            BareEntry::Comment,
        ];
        assert!(data.is_ok());
        assert_eq!(data.unwrap(), expected);
    }

    #[test]
    fn test_comment_raw() {
        #[derive(Deserialize, Debug, PartialEq)]
        enum OnlyComment<'a> {
            Entry,
            Macro,
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

        let data: Result<CommentOnlyBib> = CommentOnlyBib::deserialize(&mut bib_de);
        let expected = vec![OnlyComment::Comment(b"com")];
        assert!(data.is_ok());
        assert_eq!(data.unwrap(), expected);
    }

    #[test]
    fn test_string_capturing() {
        let reader = StrReader::new("@string{a = {1}}@string{a = a # a}@string{a = a # a}");
        let mut bib_de = Deserializer::new(reader);

        let _ = TestBibCaptureMacro::deserialize(&mut bib_de).unwrap();
        assert!(
            bib_de
                .macros
                .get(&Variable::new_unchecked("a"))
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
                .get(&Variable::new_unchecked("a"))
                .unwrap()
                .len()
                == 4
        );

        let reader = StrReader::new("@string{a = {1}}@string{a = a # a}@string{a = a # a}");
        let mut bib_de = Deserializer::new(reader);

        let _ = TestBibIgnoreMacro::deserialize(&mut bib_de).unwrap();
        println!("{:?}", bib_de.macros);
        assert!(bib_de.macros.get(&Variable::new_unchecked("a")).is_none());
    }

    #[test]
    fn test_entry() {
        let reader = StrReader::new("@string{}@string{u={v}}@a{k,a=b}");
        let mut bib_de = Deserializer::new(reader);

        let data: Result<TestBib> = TestBib::deserialize(&mut bib_de);
        let mut fields = HashMap::new();
        fields.insert("a", vec![Tok::V("b")]);

        let expected = vec![
            TestEntry::Macro(None),
            TestEntry::Macro(Some(("u", vec![Tok::T("v")]))),
            TestEntry::Regular(TestEntryMap {
                entry_type: "a",
                entry_key: "k",
                fields,
            }),
        ];
        assert!(data.is_ok());
        assert_eq!(data.unwrap(), expected);
    }

    macro_rules! syntax {
        ($input:expr, $expect:ident) => {
            let reader = StrReader::new($input);
            let mut bib_de = Deserializer::new(reader);
            let data: Result<IgnoredAny> = IgnoredAny::deserialize(&mut bib_de);
            assert!(data.$expect(), "{:?} : {:?}", data, bib_de.parser);

            let reader = StrReader::new($input);
            let mut bib_de = Deserializer::new(reader);
            let data: Result<TestBib> = TestBib::deserialize(&mut bib_de);
            assert!(data.$expect(), "{:?} : {:?}", data, bib_de.parser);

            let reader = SliceReader::new($input.as_bytes());
            let mut bib_de = Deserializer::new(reader);
            let data: Result<TestBib> = TestBib::deserialize(&mut bib_de);
            assert!(data.$expect(), "{:?} : {:?}", data, bib_de.parser);

            let parsed = BibtexParser::parse(Rule::bib, $input);
            assert!(parsed.$expect(), "{:?} : {:?}", data, parsed);
        };
    }

    #[test]
    fn test_string_syntax() {
        syntax!(r"@string{k=v}", is_ok);
        syntax!(r"@sTring{k=v,}", is_ok);

        syntax!(r"@string()", is_ok);
        syntax!(r"@string(,)", is_err);
        syntax!(r"@string{}", is_ok);
        syntax!(r"@string{,}", is_err);
        syntax!(r#"@string{1b={3}}"#, is_err);
        syntax!(r#"@string{@={3}}"#, is_ok);
    }

    #[test]
    fn test_preamble_syntax() {
        syntax!(r"@preamble()", is_err);
        syntax!(r"@preamble{}", is_err);
        syntax!(r"@ pREamble {{any} # a #{allowed}}", is_ok);
        syntax!(r"@preamble({})", is_ok);
        syntax!(r"@preamble( {} # {} # {} )", is_ok);

        syntax!(r"@preamble(", is_err);
        syntax!(r"@preamble)", is_err);
        syntax!(r"@preamble({{})", is_err);
        syntax!(r"@preamble(})", is_err);
    }

    #[test]
    fn test_comment_round_syntax() {
        syntax!(r"@comment(@anything#)", is_ok);
        syntax!(r"@comment({(}))", is_ok);
        syntax!(r"@comment({(})", is_ok);
        syntax!(r"@comment(})", is_err);
    }

    #[test]
    fn test_comment_syntax() {
        syntax!(r"@comment{{}}", is_ok);
        syntax!(r"@comment { @anything#}", is_ok);
        syntax!(r"@coMment {}", is_ok);
        syntax!("@\n CommEnt  { }", is_ok);

        syntax!(r"@comment({)", is_err);
    }

    #[test]
    fn test_regular_entry_syntax() {
        // basic example
        syntax!(
            r#"@a{key:0,
              a= {A} # b,
              t= "T",
              y= 1,}"#,
            is_ok
        );

        syntax!(
            r#"@article{1key,
                 @ = t # {, Part } # n,
               }"#,
            is_ok
        );

        // numbers and @ allowed in entry type
        syntax!(
            r#"@1art@icle{k,
                title = t # {, Part } # n,
            }"#,
            is_ok
        );

        // whitespace and unicode allowed in potentially surprising places
        syntax!(
            r#"@   a🍄ticle {k🍄:0  ,
              author ={A🍄th}
                #  
                {or}
                ,1itle =
              "Tit🍄e" # 🍄
              }"#,
            is_ok
        );

        // no fields, trailing comma
        syntax!(r#"@a{k,}"#, is_ok);
        // no fields, no trailing comma
        syntax!(r#"@a{k}"#, is_ok);
        // single field, trailing comma
        syntax!(r#"@a{k,t=v,}"#, is_ok);
        // single field, no trailing comma
        syntax!(r#"@a{k,t=v}"#, is_ok);
        // identifiers can have weird chars, e.g. `@🍄`
        syntax!(r#"@ 1@🍄{2🍄@,t=v}"#, is_ok);
        // no @, so it is junk
        syntax!(r#"a{k,t=v}"#, is_ok);
        // unicode in field keys
        syntax!(r"@article{k,auth🍄={v}}", is_ok);

        // err: multiple trailing comma
        syntax!(r#"@a{k,,}"#, is_err);
        // err: missing field value
        syntax!(r#"@a{k,t=,}"#, is_err);
        // err: missing citation key
        syntax!(r#"@a{,t=v}"#, is_err);
        // err: invalid char in citation key
        syntax!(r#"@a{t=b}"#, is_err);
        syntax!(r#"@a{t#b}"#, is_err);
        syntax!(r#"@a{t\b}"#, is_err);
        // err: extra chars before start of entry
        syntax!(r#"@ @ @{k,t=v}"#, is_err);

        // opening and closing brackets must match
        syntax!("@a(k}", is_err);
        syntax!("@a{k)", is_err);
        syntax!("@a{k}", is_ok);
        syntax!("@a(k)", is_ok);
    }
}
