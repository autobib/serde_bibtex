//! # Deserializer implementation
//! This section contains a full description, with examples, of the [`Deserializer`] interface
//! provided by this crate. It also provides an informal description of the `.bib` grammar accepted
//! by this crate. For a formal description and an explicit grammar, visit the [syntax](syntax/index.html)
//! module.
//!
//! Jump to:
//! - [Deserializing a bibliography](#deserializing-a-bibliography)
//!   - [Regular entries](#deserializing-regular-entries)
//!   - [Preamble entries](#deserializing-preamble-entries)
//!   - [Comment entries](#deserializing-comment-entries)
//!   - [Macro entries](#deserializing-macro-entries)
//! - [Macro capturing and expansion](#macro-capturing-and-expansion)
//!   - [Automatic expansion and capturing](#automatic-expansion-and-capturing)
//!   - [Manual capturing](#manual-capturing)
//! - [Deserializing values](#deserializing-values)
//! - [Borrowing and bytes](#borrowing-and-byte-deserialization)
//!
//! ## Deserializing a bibliography
//! A `.bib` file is a sequence of entries, each of which is declared by an identifier immediately
//! following the `@` symbol at the beginning of the entry. We refer to this identifier as the
//! *entry type*. There are four categories:
//!
//! 1. Macro entries, such as
//!    ```bib
//!    @string{var = {Expanded}},
//!    ```
//! 2. Preamble entries:
//!    ```bib
//!    @preamble{{\preamble}}
//!    ```
//! 3. Comment entries:
//!    ```bib
//!    @comment{Ignored}
//!    ```
//! 4. Regular entries, such as
//!    ```bib
//!    @article{key,
//!      title = {Title} # var,
//!    }
//!    ```
//! Note that the entry type is matched case-insentively. For example `@strING`, is treated as a
//! macro entry.
//!
//! In the simplest case, we can deserialize a bibliography as follows.
//! ```
//! use serde::Deserialize;
//! use serde_bibtex::de::Deserializer;
//!
//! #[derive(Debug, PartialEq, Deserialize)]
//! enum Entry {
//!     Macro,
//!     Preamble,
//!     Comment,
//!     Regular,
//! }
//!
//! let input = r#"
//!     @string{t = {Title}}
//!
//!     @preamble{{preamble}}
//!
//!     @article{key,
//!       author = {One, Author},
//!       title = t,
//!       year = 2024,
//!     }
//!
//!     @comment{Comment}
//! "#;
//!
//! type Bibliography = Vec<Entry>;
//!
//! let mut de = Deserializer::from_str(input);
//!
//! assert_eq!(
//!     Bibliography::deserialize(&mut de),
//!     Ok(vec![
//!         Entry::Macro,
//!         Entry::Preamble,
//!         Entry::Regular,
//!         Entry::Comment,
//!     ])
//! )
//! ```
//! Instead of deserializing the whole bibliography into a `Vec`, we can also iterate:
//! ```
//! # use serde::Deserialize;
//! # use serde_bibtex::de::Deserializer;
//! #
//! # #[derive(Debug, PartialEq, Deserialize)]
//! # enum Entry {
//! #     Macro,
//! #     Preamble,
//! #     Comment,
//! #     Regular,
//! # }
//! #
//! # let input = r#"
//! #     @string{t = {Title}}
//! #
//! #     @preamble{{preamble}}
//! #
//! #     @article{key,
//! #       author = {One, Author},
//! #       title = t,
//! #       year = 2024,
//! #     }
//! #
//! #     @comment{Comment}
//! # "#;
//! #
//! # let mut de = Deserializer::from_str(input);
//! #
//! let expected = vec![Entry::Macro, Entry::Preamble, Entry::Regular, Entry::Comment];
//!
//! for (entry, expect) in std::iter::zip(de.into_iter(), expected.into_iter()) {
//!     assert_eq!(entry, Ok(expect));
//! }
//! ```
//! In order to capture the contents of the individual entries, we must add contents to the enum
//! variants in our `Entry` enum.
//!
//! ### Deserializing regular entries
//! Let's begin by recalling the anatomy of a regular entry.
//! ```bib
//! @article{key,
//!   author = {One, Author},
//!   year = 2012,
//! }
//! ```
//! This entry has three components:
//! 1. The *entry type*: in this case, `article`.
//! 2. The *entry key*: in this case, `key`.
//! 3. The *fields*, which is a list of key-value pairs, in this case:
//!   - key `author`, value `One, Author`.
//!   - key `year`, value `2012`.
//!
//! For more details on *value* syntax and deserialization, go to the
//! [deserializing values](#deserializing-values) section.
//!
//! We may deserialize the `Entry` either as specially-named struct, or as a tuple of length
//! exactly three. The *fields* are deserialized either as a map of `key:value` pairs or as
//! a sequence of `(key, value)` tuples.
//!
//! For example, deserializing the above entry as input:
//! ```
//! # use serde::Deserialize;
//! # use serde_bibtex::de::Deserializer;
//! #
//! # let input = r#"
//! #     @article{key,
//! #       author = {One, Author},
//! #       year = 2012,
//! #     }
//! # "#;
//! use std::collections::BTreeMap;
//!
//! #[derive(Debug, PartialEq, Deserialize)]
//! struct Contents {
//!     entry_type: String,
//!     entry_key: String,
//!     fields: BTreeMap<String, String>,
//! }
//!
//! #[derive(Debug, PartialEq, Deserialize)]
//! enum Entry {
//!     Macro,
//!     Preamble,
//!     Comment,
//!     Regular(Contents),
//! }
//!
//! type Bibliography = Vec<Entry>;
//!
//! let mut de = Deserializer::from_str(input);
//!
//! let mut expected_fields = BTreeMap::from([
//!     ("author".into(), "One, Author".into()),
//!     ("year".into(), "2012".into()),
//! ]);
//!
//! assert_eq!(
//!     Bibliography::deserialize(&mut de),
//!     Ok(vec![Entry::Regular(Contents {
//!        entry_type: "article".into(),
//!        entry_key: "key".into(),
//!        fields: expected_fields,
//!     })])
//! );
//! ```
//! It is also possible to explicitly state which field keys you wish to capture, for instance
//! ```
//! # use serde::Deserialize;
//! #[derive(Debug, PartialEq, Deserialize)]
//! struct Fields {
//!     title: String,
//!     year: Option<String>,
//!     author: Option<String>,
//! }
//!
//! #[derive(Debug, PartialEq, Deserialize)]
//! struct Contents {
//!     entry_type: String,
//!     entry_key: String,
//!     fields: Fields
//! }
//!
//! #[derive(Debug, PartialEq, Deserialize)]
//! enum Entry {
//!     Macro,
//!     Preamble,
//!     Comment,
//!     Regular(Contents),
//! }
//! ```
//! In the above example, optional fields are set as `None` if not present, and fields which are not present
//! are automatically skipped.
//!
//! The following less strongly typed deserialization scheme would also be valid:
//! ```
//! # use serde::Deserialize;
//! #[derive(Debug, PartialEq, Deserialize)]
//! enum Entry {
//!     Macro,
//!     Preamble,
//!     Comment,
//!     Regular((String, String, Vec<(String, String)>)),
//! }
//! # use serde_bibtex::de::Deserializer;
//! # let input = r#"
//! #     @article{key,
//! #       title = {Title},
//! #     }
//! # "#;
//! #
//! # type Bibliography = Vec<Entry>;
//! #
//! # let mut de = Deserializer::from_str(input);
//! #
//! # assert_eq!(
//! #     Bibliography::deserialize(&mut de),
//! #     Ok(vec![Entry::Regular((
//! #         "article".into(),
//! #         "key".into(),
//! #         vec![("title".into(), "Title".into())]
//! #     ))])
//! # )
//! ```
//! Of course, it is also possible to explicitly ignore some parts of the entry. For example, if you only wish
//! to capture the entry key with a different struct field name:
//! ```
//! # use serde::Deserialize;
//! #[derive(Debug, PartialEq, Deserialize)]
//! struct CitationKeyOnly {
//!     #[serde(rename = "entry_key")]
//!     citation_key: String,
//! }
//!
//! #[derive(Debug, PartialEq, Deserialize)]
//! enum Entry {
//!     Macro,
//!     Preamble,
//!     Comment,
//!     Regular(CitationKeyOnly),
//! }
//! # use serde_bibtex::de::Deserializer;
//! # let input = r#"
//! #     @article{key,
//! #       title = {Title},
//! #     }
//! # "#;
//! #
//! # type Bibliography = Vec<Entry>;
//! #
//! # let mut de = Deserializer::from_str(input);
//! #
//! # assert_eq!(
//! #     Bibliography::deserialize(&mut de),
//! #     Ok(vec![Entry::Regular(CitationKeyOnly {
//! #         citation_key: "key".into(),
//! #     })])
//! # )
//! ```
//! This is not possible when deserializing a regular entry as a tuple.
//!
//! ### Deserializing preamble entries
//! A preamble consists of a single *value*:
//! ```bib
//! @preamble{{A } # {value}}
//! ```
//! We may deserialize a preamble entry in the same way that we deserialize a value, for instance.
//! ```
//! enum Entry {
//!     Macro,
//!     Preamble(String),
//!     Comment,
//!     Regular,
//! }
//! ```
//! ### Deserializing comment entries
//! A comment consists of a single *token*.
//! ```bib
//! @comment{Contents}
//! ```
//! which we can deserialize, for instance, as a string:
//! ```
//! enum Entry {
//!     Macro,
//!     Preamble,
//!     Comment(String),
//!     Regular,
//! }
//! ```
//! For more details on *token* syntax and deserialization, go to the
//! [deserializing values](#deserializing-values) section.
//!
//! ### Deserializing macro entries
//! Note that this crate performs automatic capturing and expansion of macros which are not deserialized. For
//! more details, go to the [macro capturing and expansion](#macro-capturing-and-expansion)
//! section.
//!
//! A macro entry has two components: a *variable* and a *value* separated by an `=`.
//! ```bib
//! @string{key = {Value } # 1234}
//! ```
//! For more details on *value* syntax and deserialization, go to the
//! [deserializing values](#deserializing-values) section.
//!
//!
//! For now, we deserialize this as a tuple `(String, String)`.
//! ```
//! # use serde::Deserialize;
//! # use serde_bibtex::de::Deserializer;
//! #
//! #[derive(Debug, PartialEq, Deserialize)]
//! struct Rule(String, String);
//!
//! #[derive(Debug, PartialEq, Deserialize)]
//! enum Entry {
//!     Macro(Rule),
//!     Preamble,
//!     Comment,
//!     Regular,
//! }
//!
//! type Bibliography = Vec<Entry>;
//!
//! let input = r#"@string{key = {Value } # 1234}"#;
//!
//! let mut de = Deserializer::from_str(input);
//! assert_eq!(
//!     Ok(vec![Entry::Macro(Rule("key".into(), "Value 1234".into()))]),
//!     Bibliography::deserialize(&mut de)
//! )
//! ```
//! Since macro entries can also be empty, for instance `@string{}`, one may alternatively
//! deserialize into an [`Option`]:
//! ```
//! # use serde::Deserialize;
//! # use serde_bibtex::de::Deserializer;
//! #
//! # #[derive(Debug, PartialEq, Deserialize)]
//! enum Entry {
//!     Macro(Option<(String, String)>),
//!     Preamble,
//!     Comment,
//!     Regular,
//! }
//! #
//! # type Bibliography = Vec<Entry>;
//! #
//! # let input = r#"@string{key = {Value } # 1234}"#;
//! #
//! # let mut de = Deserializer::from_str(input);
//! # assert_eq!(
//! #     Ok(vec![Entry::Macro(Some(("key".into(), "Value 1234".into())))]),
//! #     Bibliography::deserialize(&mut de)
//! # )
//! ```
//!
//! ## Macro capturing and expansion
//! This crate supports automatic macro capturing and expansion using the
//! [`MacroDictionary`](struct.MacroDictionary) struct.
//! 1. [Automatic expansion and capturing](#automatic-expansion-and-capturing)
//! 2. [Manual capturing](#manual-capturing)
//!
//! ### Automatic expansion and capturing
//! If the `Macro` variant of your `Entry` struct is a unit variant, then macros are
//! automatically captured by the [`Deserializer`] during deserialization. Any captured macros are used to
//! expand macros in subsequent values.
//!
//! In order to collect the captured macros after deserialization, use the
//! [`finish`](struct.Deserializer.html#method.finish) method.
//! ```
//! use serde::Deserialize;
//! use serde_bibtex::de::Deserializer;
//! use std::collections::BTreeMap;
//!
//! #[derive(Debug, PartialEq, Deserialize)]
//! struct Contents {
//!     fields: BTreeMap<String, String>
//! }
//!
//! #[derive(Debug, PartialEq, Deserialize)]
//! enum Entry {
//!     Macro,
//!     Comment(String),
//!     Preamble(String),
//!     Regular(Contents),
//! }
//!
//! type Bibliography = Vec<Entry>;
//!
//! let input = r#"
//!     @string{t = {Title}}
//!     @article{key,
//!       title = {A } # t,
//!     }
//! "#;
//!
//! let mut de = Deserializer::from_str(input);
//!
//! let mut expected_fields = BTreeMap::from([
//!     ("title".into(), "A Title".into()),
//! ]);
//!
//! assert_eq!(
//!     Bibliography::deserialize(&mut de),
//!     Ok(vec![
//!         Entry::Macro,
//!         Entry::Regular(Contents {
//!             fields: expected_fields
//!         })
//!     ]),
//! );
//!
//! use serde_bibtex::token::Variable;
//! assert!(de
//!     .finish()
//!     .into_inner()
//!     .contains_key(&Variable::new("t").unwrap())
//! );
//! ```
//! ### Manual capturing
//! If you explicitly capture the macro variables, as shown for example in the
//! [macro entries](#deserializing-macro-entries) section, the corresonding variables are not
//! captured. Note that macro expansion will still happen. This is relevant if you
//! define custom macros. For example:
//! ```
//! use serde::Deserialize;
//! use serde_bibtex::{de::Deserializer, MacroDictionary};
//! use std::collections::BTreeMap;
//!
//! #[derive(Debug, PartialEq, Deserialize)]
//! struct Contents {
//!     fields: BTreeMap<String, String>
//! }
//!
//! #[derive(Debug, PartialEq, Deserialize)]
//! enum Entry {
//!     Macro((String, String)),
//!     Comment(String),
//!     Preamble(String),
//!     Regular(Contents),
//! }
//!
//! type Bibliography = Vec<Entry>;
//!
//! let input = r#"
//!     @string{apr = {Nonsense}}
//!     @article{key,
//!       month = apr,
//!     }
//! "#;
//!
//! // Set 'month name' macros, e.g. @string{apr = {4}}
//! let mut macro_dict = MacroDictionary::<&str, &[u8]>::default();
//! macro_dict.set_month_macros();
//!
//! let mut de = Deserializer::from_str_with_macros(input, macro_dict);
//!
//! let mut expected_fields = BTreeMap::from([
//!     ("month".into(), "4".into()),
//! ]);
//!
//! assert_eq!(
//!     Bibliography::deserialize(&mut de),
//!     Ok(vec![
//!         // the 'apr' macro defined in input is captured here
//!         Entry::Macro(("apr".into(), "Nonsense".into())),
//!         // and the 'apr' macro defined by `set_month_macros` is
//!         // expanded here
//!         Entry::Regular(Contents {
//!             fields: expected_fields
//!         })
//!     ]),
//! );
//! ```
//! In contrast, if we do not capture the macros in the above example, the macro defined in the
//! file will *overwrite* the manually-defined macro.
//! ```
//! # use serde::Deserialize;
//! # use serde_bibtex::{MacroDictionary, de::Deserializer};
//! # use std::collections::BTreeMap;
//! #
//! # #[derive(Debug, PartialEq, Deserialize)]
//! # struct Contents {
//! #     fields: BTreeMap<String, String>
//! # }
//! #
//! #
//! # type Bibliography = Vec<Entry>;
//! #
//! # let input = r#"
//! #     @string{apr = {Nonsense}}
//! #     @article{key,
//! #       month = apr,
//! #     }
//! # "#;
//! // definition of Contents, Bibliography, input as in the previous example
//! #[derive(Debug, PartialEq, Deserialize)]
//! enum Entry {
//!     // do not capture macro
//!     Macro,
//!     Comment(String),
//!     Preamble(String),
//!     Regular(Contents),
//! }
//!
//! // Set 'month name' macros, e.g. @string{apr = {4}}
//! let mut macro_dict = MacroDictionary::default();
//! macro_dict.set_month_macros();
//!
//! let mut de = Deserializer::from_str_with_macros(input, macro_dict);
//!
//! let mut expected_fields = BTreeMap::from([
//!     ("month".into(), "Nonsense".into()),
//! ]);
//!
//! assert_eq!(
//!     Bibliography::deserialize(&mut de),
//!     Ok(vec![
//!         Entry::Macro,
//!         // the 'apr' macro defined in the input overwrites
//!         // the previously set macro
//!         Entry::Regular(Contents {
//!             fields: expected_fields
//!         })
//!     ]),
//! );
//! ```
//! If you wish to prevent automatic macro capturing, but do not care about the actual values of
//! the macro, use `serde::de::IgnoredAny`.
//! ```
//! use serde::{de::IgnoredAny, Deserialize};
//!
//! #[derive(Debug, PartialEq, Deserialize)]
//! enum Entry {
//!     Macro(IgnoredAny),
//!     Comment,
//!     Preamble,
//!     Regular,
//! }
//! ```
//! ## Deserializing values
//! *Values* can appear in three possible locations:
//! 1. Inside macro entries: `@string{key = <value>}`.
//! 2. Inside preamble entries: `@preamble{<value>}`.
//! 3. Inside field values: `@article{key, title = <value>}`.
//! A *value* is a sequence of *tokens* with length at least 1, delimited by `#`.
//! There are four possible tokens:
//! 1. Bracketed text: `{text}`.
//! 2. Quoted text: `"text"`.
//! 3. Numbers: `01234`.
//! 4. Variables: `var`.
//! For instance, the following is a value:
//! ```bib
//! {text} # "text" # 01234 # var
//! ```
//! One can think of `#` as a concatenation operator. For example, if `var` is defined as
//! ```bib
//! @string{var = {one} # 2}
//! ```
//! then the above defined value is equivalent to the single token
//! ```bib
//! {texttext01234one2}
//! ```
//! since `var` is expanded, and the resulting text is concatenated.
//! Since numbers need not be quoted, a variable cannot begin with a digit.
//!
//! In the earlier examples, we captured values directly as [`String`]s. However, if the string
//! contains undefined macros, then expansion will fail:
//! ```
//! use serde_bibtex::de::Deserializer;
//! use serde::Deserialize;
//!
//! #[derive(Debug, PartialEq, Deserialize)]
//! enum Entry {
//!     Macro,
//!     Comment,
//!     Preamble(String),
//!     Regular,
//! }
//!
//! // var is undefined
//! let input = r#"@preamble{{text} # var}"#;
//!
//! // need explicit type annotation since it cannot be inferred from the error
//! let mut de_iter = Deserializer::from_str(input).into_iter::<Entry>();
//! assert!(matches!(
//!     de_iter.next(),
//!     Some(Err(_)),
//! ));
//! ```
//! In order to tolerate errors, it is also possible to deserialize values as a sequence of enums.
//! The special variants `Variable` and `Text` are required; use `#[serde(rename = "...")]` to use
//! custom names.
//! ```
//! use serde_bibtex::de::Deserializer;
//! use serde::Deserialize;
//!
//! #[derive(Debug, PartialEq, Deserialize)]
//! enum Token {
//!     Variable(String),
//!     Text(String),
//! }
//!
//! #[derive(Debug, PartialEq, Deserialize)]
//! enum Entry {
//!     Macro,
//!     Comment,
//!     Preamble(Vec<Token>),
//!     Regular,
//! }
//!
//! // `var0` is defined using indefined macros
//! let input = r#"
//!     @string{var0 = var1 # 01234 # var2}
//!     @preamble{{text} # var0}
//! "#;
//!
//! let mut de_iter = Deserializer::from_str(input).into_iter::<Entry>();
//! assert_eq!(
//!     de_iter.next(),
//!     Some(Ok(Entry::Macro)),
//! );
//! assert_eq!(
//!     de_iter.next(),
//!     Some(Ok(Entry::Preamble(vec![
//!         Token::Text("text".into()),
//!         // var0 was defined, and therefore expanded,
//!         // but var1 and var2 are still undefined
//!         Token::Variable("var1".into()),
//!         Token::Text("01234".into()),
//!         Token::Variable("var2".into()),
//!     ]))),
//! );
//! ```
//! Internally, the [`token::Token`](token/enum.Token.html) enum is used to hold `@string` macro definitions. This helps to
//! automatically tolerate undefined macros when the value of that macro is not required.
//!
//! ## Borrowing and byte deserialization
//! Many fields can be safely borrowed since the `.bib` syntax ensures that the text will lie
//! contiguously in the underlying input stream. However, when deserializing directly from a file,
//! it is never possible to borrow. Therefore it is recommended to use something like [`std::borrow::Cow`].
//!
//! Moreover, if you are deserializing from raw bytes and you are unsure that the encoding is in
//! UTF-8, certain fields can be flexibly deserialized directly as bytes.
//!
//! The below example highlights a maximally flexible deserialization struct which will accept any
//! input file that satisfies the `.bib` syntax used by this crate.
//! ```
//! use serde::Deserialize;
//! use std::borrow::Cow;
//! use std::collections::HashMap;
//!
//! #[derive(Debug, Deserialize, PartialEq)]
//! enum Token<'a> {
//!     #[serde(borrow)] // explicit annotation is required to borrow
//!     Variable(Cow<'a, [u8]>), // defer byte conversion
//!     #[serde(borrow)]
//!     Text(Cow<'a, [u8]>),
//! }
//!
//! #[derive(Deserialize, Debug, PartialEq)]
//! struct Contents<'a> {
//!     entry_type: Cow<'a, str>, // the syntax requires these are valid utf_8
//!     entry_key: Cow<'a, str>,
//!     #[serde(borrow)]
//!     fields: HashMap<Cow<'a, str>, Vec<Token<'a>>>,
//! }
//!
//! #[derive(Deserialize, Debug, PartialEq)]
//! enum TestEntry<'a> {
//!     #[serde(borrow)]
//!     Regular(Contents<'a>),
//!     #[serde(borrow)]
//!     Macro(Option<(Cow<'a, str>, Vec<Token<'a>>)>),
//!     #[serde(borrow)]
//!     Comment(Cow<'a, [u8]>),
//!     #[serde(borrow)]
//!     Preamble(Vec<Token<'a>>),
//! }
//! ```
mod bibliography;
mod entry;
mod value;

pub use bibliography::{DeserializeEntriesIter, DeserializeIter, Deserializer};

use crate::error::Result;
use crate::parse::{SliceReader, StrReader};

use serde::Deserialize;

pub fn from_str<'r, D>(s: &'r str) -> Result<D>
where
    D: Deserialize<'r>,
{
    let reader = StrReader::new(s);
    let mut deserializer = Deserializer::new(reader);
    D::deserialize(&mut deserializer)
}

pub fn from_bytes<'r, D>(s: &'r [u8]) -> Result<D>
where
    D: Deserialize<'r>,
{
    let reader = SliceReader::new(s);
    let mut deserializer = Deserializer::new(reader);
    D::deserialize(&mut deserializer)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::borrow::Cow;
    use std::iter::zip;

    #[derive(Deserialize, Debug, PartialEq)]
    enum TestEntry<'a> {
        #[serde(borrow)]
        Regular(TestRegularEntry<'a>),
        Macro,
        Comment,
        Preamble,
    }

    #[derive(Deserialize, Debug, PartialEq)]
    struct TestRegularEntry<'a> {
        entry_type: &'a str,
        entry_key: &'a str,
        #[serde(borrow)]
        fields: TestFields<'a>,
    }

    #[derive(Deserialize, Debug, PartialEq)]
    struct TestFields<'a> {
        #[serde(borrow)]
        author: Cow<'a, str>,
        #[serde(borrow)]
        title: Cow<'a, str>,
    }

    #[test]
    fn test_deserialize_iter() {
        let input = r#"
        @article{key,
           author = {One, Author} # { and } # {Two, Author},
           title = {Title},
           year = 2024,
           nonsense = 12,
        }

        @string(k={val})

        @preamble{"a value"}

        @comment{ignored}

        @book{key2,
           author = {Auth},
           title = k # 1 # {Year},
        }
        "#;

        let expected = vec![
            TestEntry::Regular(TestRegularEntry {
                entry_type: "article",
                entry_key: "key",
                fields: TestFields {
                    author: "One, Author and Two, Author".into(),
                    title: "Title".into(),
                },
            }),
            TestEntry::Macro,
            TestEntry::Preamble,
            TestEntry::Comment,
            TestEntry::Regular(TestRegularEntry {
                entry_type: "book",
                entry_key: "key2",
                fields: TestFields {
                    author: "Auth".into(),
                    title: "val1Year".into(),
                },
            }),
        ];

        let reader = StrReader::new(input);
        for (expected, received) in zip(expected.into_iter(), Deserializer::new(reader).into_iter())
        {
            assert_eq!(Ok(expected), received);
        }
    }

    #[test]
    fn test_deserialize_iter_entries() {
        let input = r#"
        @string{k = {12}}

        @article{key,
           author = {Author},
           title = k
        }

        @string{k = k # k}

        @article{k2,
           author = {Author 2},
           title = k # k,
        }
        "#;

        let expected = vec![
            TestRegularEntry {
                entry_type: "article",
                entry_key: "key",
                fields: TestFields {
                    author: "Author".into(),
                    title: "12".into(),
                },
            },
            TestRegularEntry {
                entry_type: "article",
                entry_key: "k2",
                fields: TestFields {
                    author: "Author 2".into(),
                    title: "12121212".into(),
                },
            },
        ];

        let reader = StrReader::new(input);
        for (expected, received) in zip(
            expected.into_iter(),
            Deserializer::new(reader).into_iter_entry(),
        ) {
            assert_eq!(Ok(expected), received);
        }
    }
}
