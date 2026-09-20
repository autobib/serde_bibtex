//! Byte spans through public readers, Serde adapters, and both iterator interfaces.
#![allow(dead_code)]

use core::{fmt::Debug, ops::Range};
use std::collections::BTreeMap;

use serde::{
    Deserialize,
    de::{DeserializeOwned, IgnoredAny},
};
use serde_bibtex::{
    Error, MacroDictionary, StrReader,
    de::Deserializer,
    from_bytes, from_str,
    token::{Token, Variable},
};

type Record<V = String, K = String> = (String, String, BTreeMap<K, V>);

fn span(error: Error, expected: Range<usize>) {
    assert_eq!(error.span(), Some(expected), "{error}");
    assert!(!error.to_string().contains(" at line "));
}

fn check<D: DeserializeOwned + Debug>(input: &str, expected: Range<usize>) {
    for error in [
        from_str::<Vec<D>>(input).unwrap_err(),
        from_bytes::<Vec<D>>(input.as_bytes()).unwrap_err(),
        Deserializer::from_str(input)
            .into_iter::<D>()
            .next()
            .unwrap()
            .unwrap_err(),
        Deserializer::from_slice(input.as_bytes())
            .into_iter::<D>()
            .next()
            .unwrap()
            .unwrap_err(),
    ] {
        assert!(input.get(error.span().unwrap()).is_some());
        span(error, expected.clone());
    }
}

fn regular<D: DeserializeOwned + Debug>(input: &str, expected: Range<usize>) {
    check::<D>(input, expected.clone());
    for error in [
        Deserializer::from_str(input)
            .into_iter_regular_entry::<D>()
            .next()
            .unwrap()
            .unwrap_err(),
        Deserializer::from_slice(input.as_bytes())
            .into_iter_regular_entry::<D>()
            .next()
            .unwrap()
            .unwrap_err(),
    ] {
        assert!(input.get(error.span().unwrap()).is_some());
        span(error, expected.clone());
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
struct Reject;
impl<'de> Deserialize<'de> for Reject {
    fn deserialize<D: serde::Deserializer<'de>>(_: D) -> Result<Self, D::Error> {
        Err(serde::de::Error::custom("rejected"))
    }
}

#[test]
fn identifiers_tokens_and_values() {
    regular::<(Reject, String, IgnoredAny)>("@article{k}", 1..8);
    regular::<(String, Reject, IgnoredAny)>("@article{k}", 9..10);
    regular::<Record<String, Reject>>("@article{k,field={x}}", 11..16);
    regular::<(String, String, Vec<(Reject, String)>)>("@article{k,field={x}}", 11..16);
    regular::<Record<Reject>>(
        "@article{k,f= %before\n {x} # %inside\n \"y\" %after\n}",
        23..41,
    );
    regular::<Record<bool>>("@article{k,f={x}#{y}}", 13..20);
    regular::<Record<Vec<Reject>>>("@article{k,f={x}#{y}}", 13..20);
    regular::<Record<Vec<Reject>>>("@article{k,f=undefined}", 13..22);
    regular::<Record<Vec<Reject>>>("@article{k,f=1234}", 13..17);
    regular::<Record<Vec<Reject>>>("@article{k,f=\"text\"}", 13..19);
    regular::<Record<Vec<Reject>>>("@article{k,f={}}", 13..15);
    regular::<Record>("@article{k,f={x}#undefined}", 13..26);

    #[derive(Debug, Deserialize)]
    enum Preamble {
        Preamble(Vec<Reject>),
    }
    check::<Preamble>("@preamble{123}", 10..13);
    check::<Preamble>("@preamble(123)", 10..13);
    check::<Preamble>("@preamble{{123}}", 10..15);
    check::<Preamble>("@preamble(\"123\")", 10..15);
    let input = "🍄 % @ignored\n @a@b{k}";
    let entry = input.find("@a@b").unwrap();
    regular::<(Reject, String, IgnoredAny)>(input, entry + 1..entry + 4);
    #[derive(Debug, Deserialize)]
    enum BadToken {
        Text(bool),
        Variable(bool),
    }
    regular::<Record<Vec<BadToken>>>("@article{k,f={x}}", 13..16);
    regular::<Record<Vec<BadToken>>>("@article{k,f=macro}", 13..18);
    #[derive(Debug, Deserialize)]
    enum BadValue {
        Newtype(String),
        Tuple(String, String),
        Struct { field: String },
    }
    for name in ["Newtype", "Tuple", "Struct"] {
        regular::<Record<BadValue>>(&format!("@article{{k,f={{{name}}}}}"), 13..15 + name.len());
    }
    #[derive(Debug, Deserialize)]
    enum WrongToken {
        Text(String, String),
        Variable { name: String },
    }
    regular::<Record<Vec<WrongToken>>>("@article{k,f={x}}", 13..16);
    regular::<Record<Vec<WrongToken>>>("@article{k,f=macro}", 13..18);
}

#[test]
fn containers_and_bibliographies() {
    regular::<Reject>(" junk @article{k}", 6..14);
    #[derive(Debug, Deserialize)]
    struct Missing {
        required: String,
    }
    regular::<Missing>("@article{k}", 0..11);
    regular::<Missing>("@article{k}}", 0..11);
    regular::<Record<Missing>>("@article{k,f={x}}", 13..16);
    regular::<(String, String, Missing)>("@article{k}", 0..11);
    regular::<(String, String)>("@article{k}", 0..8);
    #[derive(Debug, Deserialize)]
    enum OnlyComment {
        Comment,
    }
    check::<OnlyComment>("@article{k}", 0..8);
    #[derive(Debug, Deserialize)]
    enum Entries {
        Preamble(Reject),
        Macro(Option<(Reject, String)>),
        Comment(Reject),
    }
    check::<Entries>("@preamble{{x}}", 10..13);
    check::<Entries>("@string{x={y}}", 8..9);
    check::<Entries>("@comment(text)", 8..14);
    check::<Entries>("@comment{text}", 8..14);
    for input in ["", "unconsumed junk"] {
        span(from_str::<Reject>(input).unwrap_err(), 0..0);
        span(from_bytes::<Reject>(input.as_bytes()).unwrap_err(), 0..0);
        span(
            bool::deserialize(&mut Deserializer::from_str(input)).unwrap_err(),
            0..0,
        );
    }
    #[derive(Debug)]
    struct RejectBibliography;
    impl<'de> Deserialize<'de> for RejectBibliography {
        fn deserialize<D: serde::Deserializer<'de>>(de: D) -> Result<Self, D::Error> {
            Vec::<IgnoredAny>::deserialize(de)?;
            Err(serde::de::Error::custom("after bibliography"))
        }
    }
    span(
        from_str::<RejectBibliography>("@a{k} trailing").unwrap_err(),
        0..14,
    );
    let mut iter =
        Deserializer::from_str("@a{k} junk @article{key}").into_iter_regular_entry::<Missing>();
    span(iter.next().unwrap().unwrap_err(), 0..5);
    span(iter.next().unwrap().unwrap_err(), 11..24);
}

#[test]
fn unicode_syntax_spans() {
    for ch in ['é', '中', '🍄'] {
        for input in [
            format!("@article {ch}"),
            format!("@article{{key {ch}}}"),
            format!("@article{{key,field {ch}}}"),
            format!("@article{{key,field={{text}}{ch}}}"),
            format!("@article{{key,field=123{ch}}}"),
            format!("@string{{name {ch}}}"),
            format!("@preamble{{{{text}}{ch}}}"),
            format!("@comment {ch}"),
        ] {
            let start = input.find(ch).unwrap();
            let expected = start..start + ch.len_utf8();
            for error in [
                from_str::<IgnoredAny>(&input).unwrap_err(),
                from_str::<Vec<IgnoredAny>>(&input).unwrap_err(),
                Deserializer::from_str(&input)
                    .into_iter::<IgnoredAny>()
                    .next()
                    .unwrap()
                    .unwrap_err(),
                Deserializer::from_str(&input)
                    .into_iter_regular_entry::<IgnoredAny>()
                    .next()
                    .unwrap()
                    .unwrap_err(),
            ] {
                assert_eq!(
                    input.get(error.span().unwrap()),
                    Some(ch.to_string().as_str())
                );
                span(error, expected.clone());
            }
            for error in [
                from_bytes::<IgnoredAny>(input.as_bytes()).unwrap_err(),
                from_bytes::<Vec<IgnoredAny>>(input.as_bytes()).unwrap_err(),
                Deserializer::from_slice(input.as_bytes())
                    .into_iter::<IgnoredAny>()
                    .next()
                    .unwrap()
                    .unwrap_err(),
                Deserializer::from_slice(input.as_bytes())
                    .into_iter_regular_entry::<IgnoredAny>()
                    .next()
                    .unwrap()
                    .unwrap_err(),
            ] {
                span(error, start..start + 1);
            }
        }
        regular::<(Reject, String, IgnoredAny)>(&format!("@{ch}{{key}}"), 1..1 + ch.len_utf8());
        regular::<Record<Vec<Reject>>>(
            &format!("@article{{key,field={{{ch}}}}}"),
            19..21 + ch.len_utf8(),
        );
    }
}

#[test]
fn macro_byte_slices_point_to_the_value() {
    let input = "% 🍄\n@article{k,f={prefix}#x}";
    for bytes in [&input.as_bytes()[2..3], &input.as_bytes()[3..6]] {
        let mut macros = MacroDictionary::default();
        macros.insert(
            Variable::new("x").unwrap(),
            vec![Token::bytes(bytes).unwrap()],
        );
        for error in [
            Deserializer::from_str_with_macros(input, macros.clone())
                .into_iter_regular_entry::<Record>()
                .next()
                .unwrap()
                .unwrap_err(),
            Deserializer::from_str_with_macros(input, macros)
                .into_iter_regular_entry::<Record<Vec<Reject>>>()
                .next()
                .unwrap()
                .unwrap_err(),
        ] {
            assert_eq!(input.get(error.span().unwrap()), Some("{prefix}#x"));
        }
    }
}

proptest::proptest! {
    #[test]
    fn string_error_spans_are_sliceable(s in ".*") {
        for input in [s.clone(), format!("@{s}"), format!("@article{{key,field={s}")] {
            for error in [
                from_str::<IgnoredAny>(&input).err(),
                from_str::<Vec<Record>>(&input).err(),
            ].into_iter().flatten() {
                let span = error.span().expect("string input has an error span");
                proptest::prop_assert!(input.get(span.clone()).is_some(), "{span:?} in {input:?}: {error}");
            }
            if let Err(error) = StrReader::new(&input).read_field_key() {
                proptest::prop_assert!(input.get(error.span().unwrap()).is_some());
            }
        }
    }
}

#[test]
fn utf8_spans() {
    for bytes in [b"a\xffz".as_slice(), b"a\xe2\x82", b"a\xe2\x82!"] {
        let mut input = b"@article{k,f={".to_vec();
        input.extend_from_slice(bytes);
        input.extend_from_slice(b"}}");
        let expected = 13..input.len() - 1;
        for error in [
            from_bytes::<Vec<Record>>(&input).unwrap_err(),
            Deserializer::from_slice(&input)
                .into_iter::<Record>()
                .next()
                .unwrap()
                .unwrap_err(),
            Deserializer::from_slice(&input)
                .into_iter_regular_entry::<Record>()
                .next()
                .unwrap()
                .unwrap_err(),
        ] {
            span(error, expected.clone());
        }
        assert!(from_bytes::<IgnoredAny>(&input).is_ok());
    }
    #[derive(Debug, Deserialize)]
    enum Entry {
        Comment(String),
        Preamble(String),
        Regular(Record<Vec<Text>>),
    }
    #[derive(Debug, Deserialize)]
    enum Text {
        Text(String),
    }
    for (input, expected) in [
        (b"@comment{a\xff}".as_slice(), 10..11),
        (b"@preamble{{a\xe2\x82}}", 10..15),
        (b"@article{k,f={a\xff}}", 13..17),
        (b"@\xff{}", 1..2),
        (b"@a{\xff}", 3..4),
        (b"@a{k,\xff={x}}", 5..6),
    ] {
        span(from_bytes::<Vec<Entry>>(input).unwrap_err(), expected);
    }
}

#[test]
fn macro_errors_keep_the_message_and_cover_the_value() {
    #[derive(Debug, Deserialize)]
    enum Entry {
        Macro,
        Regular(Record),
    }
    #[derive(Debug, Deserialize)]
    enum TextOnly {
        Text(String),
    }

    for definition in [
        "@string{x=missing}",
        "@string{inner={ok}#missing}@string{x=inner}",
    ] {
        let input =
            format!("{definition}@article{{k,title={{prefix}} # % before\n x # {{suffix}}}}");
        let expected = input.find("{prefix}").unwrap()..input.len() - 1;
        for error in [
            from_str::<Vec<Entry>>(&input).unwrap_err(),
            from_bytes::<Vec<Entry>>(input.as_bytes()).unwrap_err(),
            Deserializer::from_str(&input)
                .into_iter::<Entry>()
                .last()
                .unwrap()
                .unwrap_err(),
            Deserializer::from_str(&input)
                .into_iter_regular_entry::<Record>()
                .next()
                .unwrap()
                .unwrap_err(),
            Deserializer::from_str(&input)
                .into_iter_regular_entry::<(String, String, Vec<(String, String)>)>()
                .next()
                .unwrap()
                .unwrap_err(),
        ] {
            assert_eq!(
                error.to_string(),
                "expected text, got unresolved macro missing"
            );
            span(error, expected.clone());
        }
        // A Serde error on a replacement token also points to the complete value.
        let error = Deserializer::from_str(&input)
            .into_iter_regular_entry::<Record<Vec<TextOnly>>>()
            .next()
            .unwrap()
            .unwrap_err();
        span(error, expected.clone());

        // A visitor rejecting the combined value still gets the entire value's span.
        let error = Deserializer::from_str(&input)
            .into_iter_regular_entry::<Record<Reject>>()
            .next()
            .unwrap()
            .unwrap_err();
        span(error, input.find("{prefix}").unwrap()..input.len() - 1);
    }

    // Empty and multi-token expansions preserve the original value span.
    let mut macros = MacroDictionary::default();
    macros.insert(Variable::new("empty").unwrap(), vec![]);
    macros.insert(
        Variable::new("x").unwrap(),
        vec![Token::str("a").unwrap(), Token::str("b").unwrap()],
    );
    let input = "@article{k,f=empty#x#missing}";
    let error = Deserializer::from_str_with_macros(input, macros)
        .into_iter_regular_entry::<Record>()
        .next()
        .unwrap()
        .unwrap_err();
    assert_eq!(input.get(error.span().unwrap()), Some("empty#x#missing"));
    assert_eq!(
        error.to_string(),
        "expected text, got unresolved macro missing"
    );
}

#[test]
fn macro_sources_and_external_dictionaries() {
    let input = "@string{x=missing}@article{k,f=x}";
    span(
        Deserializer::from_str(input)
            .into_iter_regular_entry::<Record>()
            .next()
            .unwrap()
            .unwrap_err(),
        31..32,
    );
    let input = b"@string{x={a\xff}}@article{k,f=x}";
    span(
        Deserializer::from_slice(input)
            .into_iter_regular_entry::<Record>()
            .next()
            .unwrap()
            .unwrap_err(),
        28..29,
    );
    let input = "@string{x={text}}@article{k,f=x}";
    span(
        Deserializer::from_str(input)
            .into_iter_regular_entry::<Record<Vec<Reject>>>()
            .next()
            .unwrap()
            .unwrap_err(),
        30..31,
    );
    span(
        Deserializer::from_str(input)
            .into_iter_regular_entry::<Record<Reject>>()
            .next()
            .unwrap()
            .unwrap_err(),
        30..31,
    );
    // External dictionary tokens point to the whole value, regardless of their contents.
    let external = String::from("missing");
    let input = String::from("@article{missing,f={prefix}#x}");
    let mut macros = MacroDictionary::default();
    macros.insert(
        Variable::new("x").unwrap(),
        vec![Token::variable(external.as_str()).unwrap()],
    );
    span(
        Deserializer::from_str_with_macros(&input, macros)
            .into_iter_regular_entry::<Record>()
            .next()
            .unwrap()
            .unwrap_err(),
        19..29,
    );
    let mut macros = MacroDictionary::default();
    macros.insert(
        Variable::new("x").unwrap(),
        vec![Token::bytes(b"a\xff".as_slice()).unwrap()],
    );
    span(
        Deserializer::from_str_with_macros("@article{k,f={ok}#x}", macros)
            .into_iter_regular_entry::<Record>()
            .next()
            .unwrap()
            .unwrap_err(),
        13..19,
    );
    let old = String::from("@string{x=missing}");
    let mut de = Deserializer::from_str(&old);
    #[derive(Deserialize)]
    enum Macro {
        Macro,
    }
    Vec::<Macro>::deserialize(&mut de).unwrap();
    span(
        Deserializer::from_str_with_macros("@article{k,f=x}", de.finish())
            .into_iter_regular_entry::<Record>()
            .next()
            .unwrap()
            .unwrap_err(),
        13..14,
    );
}

#[test]
fn source_free_and_compact_errors() {
    let input = String::from("@comment(unclosed");
    let error = from_str::<IgnoredAny>(&input).unwrap_err();
    drop(input);
    span(error, 8..9);

    assert_eq!(core::mem::size_of::<Error>(), core::mem::size_of::<usize>());
    for error in [
        <Error as serde::de::Error>::custom("standalone"),
        <Error as serde::ser::Error>::custom("standalone"),
        Error::from(std::io::Error::other("io")),
        Error::from(serde_bibtex::token::ConversionError::UnexpandedMacro(
            "x".into(),
        )),
        serde_bibtex::to_string(&true).unwrap_err(),
    ] {
        assert_eq!(error.span(), None);
    }
    let invalid = vec![0xff];
    assert_eq!(
        Error::from(core::str::from_utf8(&invalid).unwrap_err()).span(),
        None
    );
}
