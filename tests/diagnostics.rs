//! Exact-message regression tests; see diagnostics.md for the error-site matrix.
#![allow(dead_code)]

use std::{collections::BTreeMap, io};

use serde::{Deserialize, Serialize, de::IgnoredAny};
use serde_bibtex::{
    BibtexRead, Error, SliceReader, StrReader, de::Deserializer, error::Category, from_bytes,
    from_str, to_string, to_writer,
};

#[derive(Debug, Deserialize)]
enum Token {
    Text(String),
    Variable(String),
}

type Record = (String, String, BTreeMap<String, Vec<Token>>);

#[derive(Debug, Deserialize)]
struct MapRecord {
    entry_type: String,
    entry_key: String,
    fields: BTreeMap<String, Vec<Token>>,
}

type SeqRecord = (String, String, Vec<(String, Vec<Token>)>);

#[derive(Debug, Deserialize)]
enum Entry<R = Record> {
    Regular(R),
    Macro(Option<(String, Vec<Token>)>),
    Comment(String),
    Preamble(Vec<Token>),
}

#[derive(Debug, Deserialize)]
enum IgnoredEntry {
    Regular,
    Macro,
    Comment,
    Preamble,
}

fn check(error: Error, message: &str, category: Category) {
    assert_eq!(error.to_string(), message);
    assert_eq!(error.classify(), category, "{message}");
}

fn syntax_case(input: &str, message: &str, eof: bool) {
    let errors = [
        from_str::<Vec<Entry>>(input).unwrap_err(),
        from_bytes::<Vec<Entry>>(input.as_bytes()).unwrap_err(),
        from_str::<Vec<Entry<MapRecord>>>(input).unwrap_err(),
        from_str::<Vec<Entry<SeqRecord>>>(input).unwrap_err(),
        from_str::<Vec<Entry<(String, String, IgnoredAny)>>>(input).unwrap_err(),
        from_str::<IgnoredAny>(input).unwrap_err(),
        from_bytes::<IgnoredAny>(input.as_bytes()).unwrap_err(),
        from_str::<Vec<IgnoredEntry>>(input).unwrap_err(),
        Deserializer::from_str(input)
            .into_iter::<Entry>()
            .next()
            .unwrap()
            .unwrap_err(),
        Deserializer::from_slice(input.as_bytes())
            .into_iter::<Entry>()
            .next()
            .unwrap()
            .unwrap_err(),
        Deserializer::from_str(input)
            .into_iter_regular_entry::<Record>()
            .next()
            .unwrap()
            .unwrap_err(),
        Deserializer::from_slice(input.as_bytes())
            .into_iter_regular_entry::<Record>()
            .next()
            .unwrap()
            .unwrap_err(),
        Deserializer::from_str(input)
            .into_iter_regular_entry::<()>()
            .next()
            .unwrap()
            .unwrap_err(),
    ];
    for error in errors {
        assert_eq!(error.to_string(), message, "input: {input:?}");
        assert_eq!(
            error.classify(),
            if eof { Category::Eof } else { Category::Syntax },
            "input: {input:?}"
        );
    }
}

#[test]
fn entry_error_matrix() {
    for (input, message, eof) in [
        ("@", "unexpected end of input; expected identifier", true),
        (
            "@ %comment",
            "unexpected end of input; expected identifier",
            true,
        ),
        ("@{", "expected identifier", false),
        (
            "@article",
            "unexpected end of input; expected start of entry '{' or '('",
            true,
        ),
        ("@article=", "expected start of entry '{' or '('", false),
        ("@article{}", "expected identifier", false),
        ("@article{k,,}", "expected identifier", false),
        ("@article{k,f {x}}", "expected field separator '='", false),
        ("@article{k,f=}", "expected value", false),
        ("@article{k,f=#}", "expected value", false),
        ("@article{k,f={x} # ,}", "expected value", false),
        (
            "@article{k,f={x} {y}}",
            "expected token separator '#' or end of value",
            false,
        ),
        ("@article{k)", "expected end of entry '}', found ')'", false),
        ("@article(k}", "expected end of entry ')', found '}'", false),
        (
            "@article{k,)",
            "expected end of entry '}', found ')'",
            false,
        ),
        (
            "@article{k,f={x})",
            "expected end of entry '}', found ')'",
            false,
        ),
        (
            "@article{k=}",
            "expected end of entry '}', found '='",
            false,
        ),
        ("@article{k,f=\"}\"}", "unmatched closing '}'", false),
        (
            "@string{1x={x}}",
            "identifier starts with ASCII digit",
            false,
        ),
        ("@string{=}", "expected identifier", false),
        ("@string{x {x}}", "expected field separator '='", false),
        ("@string{x=}", "expected value", false),
        ("@string{)", "expected end of entry '}', found ')'", false),
        (
            "@string(x={x},}",
            "expected end of entry ')', found '}'",
            false,
        ),
        ("@preamble{}", "expected value", false),
        (
            "@preamble({x}}",
            "expected end of entry ')', found '}'",
            false,
        ),
        ("@comment(})", "unmatched closing '}'", false),
        ("@comment(text", "unclosed '('", true),
        ("@comment({text", "unclosed '{'", true),
        ("@comment({text}", "unclosed '('", true),
        ("@comment{text", "unclosed '{'", true),
        ("@article(k,f=\"text", "unclosed '\"'", true),
        ("@article(k,f=\"{text", "unclosed '{'", true),
        ("@article(k,f=\"{text}", "unclosed '\"'", true),
        ("@article(k,f={text", "unclosed '{'", true),
    ] {
        syntax_case(input, message, eof);
    }

    // Every grammar position where EOF can occur inherits the entry opener.
    for opener in ['{', '('] {
        for body in [
            "", "k", "k,", "k,f", "k,f=", "k,f={x}", "k,f={x}#", "k,f={x},",
        ] {
            syntax_case(
                &format!("@article{opener}{body}"),
                &format!("unclosed '{opener}'"),
                true,
            );
        }
        for body in ["", "x", "x=", "x={x}", "x={x}#", "x={x},"] {
            syntax_case(
                &format!("@string{opener}{body}"),
                &format!("unclosed '{opener}'"),
                true,
            );
        }
        for body in ["", "{x}", "{x}#"] {
            syntax_case(
                &format!("@preamble{opener}{body}"),
                &format!("unclosed '{opener}'"),
                true,
            );
        }
    }
}

#[test]
fn reader_error_matrix() {
    for (input, message, category) in [
        (
            "",
            "unexpected end of input; expected identifier",
            Category::Eof,
        ),
        (",", "expected identifier", Category::Syntax),
    ] {
        check(
            StrReader::new(input).identifier().unwrap_err(),
            message,
            category,
        );
        check(
            SliceReader::new(input.as_bytes()).identifier().unwrap_err(),
            message,
            if input.is_empty() {
                Category::Eof
            } else {
                Category::Syntax
            },
        );
    }
    for input in ["", "x"] {
        let message = if input.is_empty() {
            "unexpected end of input; expected number"
        } else {
            "expected number"
        };
        check(
            StrReader::new(input).number().unwrap_err(),
            message,
            if input.is_empty() {
                Category::Eof
            } else {
                Category::Syntax
            },
        );
        check(
            SliceReader::new(input.as_bytes()).number().unwrap_err(),
            message,
            if input.is_empty() {
                Category::Eof
            } else {
                Category::Syntax
            },
        );
    }
    for input in ["", "text", "{{text}"] {
        check(
            StrReader::new(input).balanced().unwrap_err(),
            "unclosed '{'",
            Category::Eof,
        );
        check(
            SliceReader::new(input.as_bytes()).balanced().unwrap_err(),
            "unclosed '{'",
            Category::Eof,
        );
    }
    for (until, input, message, eof) in [
        (b'"', "", "unclosed '\"'", true),
        (b'"', "{", "unclosed '{'", true),
        (b'"', "}", "unmatched closing '}'", false),
        (b')', "", "unclosed '('", true),
        (b')', "{", "unclosed '{'", true),
        (b')', "}", "unmatched closing '}'", false),
    ] {
        check(
            StrReader::new(input).protected(until).unwrap_err(),
            message,
            if eof { Category::Eof } else { Category::Syntax },
        );
        check(
            SliceReader::new(input.as_bytes())
                .protected(until)
                .unwrap_err(),
            message,
            if eof { Category::Eof } else { Category::Syntax },
        );
    }
    check(
        StrReader::new(" %end").read_field_key().unwrap_err(),
        "unexpected end of input; expected identifier",
        Category::Eof,
    );
    check(
        StrReader::new(",").read_field_key().unwrap_err(),
        "expected identifier",
        Category::Syntax,
    );
    check(
        StrReader::new("").skip_field_sep().unwrap_err(),
        "unexpected end of input; expected field separator '='",
        Category::Eof,
    );
    check(
        StrReader::new("x").skip_field_sep().unwrap_err(),
        "expected field separator '='",
        Category::Syntax,
    );
    check(
        StrReader::new("").read_text_token().unwrap_err(),
        "unexpected end of input; expected value",
        Category::Eof,
    );
    check(
        StrReader::new("#").read_text_token().unwrap_err(),
        "expected value",
        Category::Syntax,
    );
    check(
        StrReader::new("macro").read_text_token().unwrap_err(),
        "expected text token, found variable",
        Category::Syntax,
    );
}

#[derive(Debug)]
struct Reject;

impl<'de> Deserialize<'de> for Reject {
    fn deserialize<D: serde::Deserializer<'de>>(_: D) -> Result<Self, D::Error> {
        Err(serde::de::Error::custom("visitor rejected input"))
    }
}

#[test]
fn visitor_error_precedence() {
    #[derive(Debug, Deserialize)]
    enum RejectPreamble {
        Preamble(Reject),
    }
    #[derive(Debug, Deserialize)]
    enum RejectMacro {
        Macro(Reject),
    }
    #[derive(Debug, Deserialize)]
    enum RejectOptionalMacro {
        Macro(Option<Reject>),
    }
    for input in ["@preamble{{x}}", "@preamble{{x}", "@preamble({x}}"] {
        check(
            from_str::<Vec<RejectPreamble>>(input).unwrap_err(),
            "visitor rejected input",
            Category::Data,
        );
    }
    // Reject from inside visit_seq/visit_some, after parsing the macro value.
    #[derive(Debug, Deserialize)]
    enum RejectMacroKey {
        Macro((Reject, String)),
    }
    for input in ["@string{x={x}}", "@string{x={x}", "@string(x={x}}"] {
        check(
            from_str::<Vec<RejectMacroKey>>(input).unwrap_err(),
            "visitor rejected input",
            Category::Data,
        );
        check(
            from_str::<Vec<RejectOptionalMacro>>(input).unwrap_err(),
            "visitor rejected input",
            Category::Data,
        );
        check(
            from_str::<Vec<RejectMacro>>(input).unwrap_err(),
            "visitor rejected input",
            Category::Data,
        );
    }
    check(
        from_str::<Reject>("").unwrap_err(),
        "visitor rejected input",
        Category::Data,
    );
    check(
        from_str::<Vec<Reject>>("@article{k}").unwrap_err(),
        "visitor rejected input",
        Category::Data,
    );
    check(
        Deserializer::from_str("@article{k}")
            .into_iter::<Reject>()
            .next()
            .unwrap()
            .unwrap_err(),
        "visitor rejected input",
        Category::Data,
    );
    check(
        Deserializer::from_str("@article{k}")
            .into_iter_regular_entry::<Reject>()
            .next()
            .unwrap()
            .unwrap_err(),
        "visitor rejected input",
        Category::Data,
    );
}

#[test]
fn serde_representation_errors_are_data() {
    #[derive(Debug, Deserialize)]
    enum OnlyComment {
        Comment,
    }
    check(
        from_str::<Vec<OnlyComment>>("@article{k}").unwrap_err(),
        "unknown variant `Regular`, expected `Comment`",
        Category::Data,
    );
    #[derive(Debug, Deserialize)]
    struct Required {
        required: String,
    }
    check(
        from_str::<Vec<Required>>("@article{k}").unwrap_err(),
        "missing field `required`",
        Category::Data,
    );
    check(
        from_str::<bool>("").unwrap_err(),
        "invalid type: sequence, expected a boolean",
        Category::Data,
    );
    check(
        from_str::<Vec<Record>>("@comment{x}").unwrap_err(),
        "invalid type: tuple variant, expected non-regular entry as tuple variant",
        Category::Data,
    );
}

#[test]
fn adapter_representation_matrix() {
    macro_rules! rejected {
        ($target:ty, $input:expr, $message:expr) => {
            check(
                from_str::<Vec<$target>>($input).unwrap_err(),
                $message,
                Category::Data,
            );
        };
    }
    rejected!(
        (String, String),
        "@article{k}",
        "invalid type: tuple variant, expected regular entry as tuple of length not 3"
    );
    rejected!(
        MapRecord,
        "@comment{x}",
        "invalid type: struct variant, expected non-regular entry as struct variant"
    );
    #[derive(Debug, Deserialize)]
    enum WrongTuple {
        Regular(String, String),
        Macro(String, String, String),
        Preamble(String, String),
        Comment(String, String),
    }
    for (input, expected) in [
        ("@article{k}", "regular entry as tuple of length not 3"),
        ("@string{x={x}}", "macro as tuple of length not 2"),
        ("@preamble{{x}}", "preamble as tuple variant"),
        ("@comment{x}", "comment as tuple variant"),
    ] {
        rejected!(
            WrongTuple,
            input,
            &format!("invalid type: tuple variant, expected {expected}")
        );
    }
    rejected!(
        Entry<(String, String)>,
        "@article{k}",
        "invalid type: sequence, expected entry can only be deserialized as a tuple of length 3"
    );
    rejected!(
        Entry<Vec<String>>,
        "@article{k}",
        "invalid type: sequence, expected entry can only be deserialized as a tuple of length 3"
    );
    rejected!(
        (bool, String, IgnoredAny),
        "@article{k}",
        "invalid type: string \"article\", expected a boolean"
    );
    rejected!(
        (String, bool, IgnoredAny),
        "@article{k}",
        "invalid type: string \"k\", expected a boolean"
    );
    rejected!(
        (String, String, BTreeMap<bool, String>),
        "@article{k,f={x}}",
        "invalid type: string \"f\", expected a boolean"
    );
    rejected!(
        (String, String, BTreeMap<String, bool>),
        "@article{k,f={x}}",
        "invalid type: string \"x\", expected a boolean"
    );
    rejected!(
        (String, String, BTreeMap<String, &str>),
        "@article{k,f={x}#{y}}",
        "invalid type: string \"xy\", expected a borrowed string"
    );

    #[derive(Debug, Deserialize)]
    enum ValueVariant {
        Newtype(String),
        Tuple(String, String),
        Struct { field: String },
    }
    type ValueRecord = (String, String, BTreeMap<String, ValueVariant>);
    for (variant, shape) in [
        ("Newtype", "newtype"),
        ("Tuple", "tuple"),
        ("Struct", "struct"),
    ] {
        rejected!(
            ValueRecord,
            &format!("@article{{k,f={{{variant}}}}}"),
            &format!("invalid type: {shape} variant, expected value as {shape} variant")
        );
    }
    #[derive(Debug, Deserialize)]
    enum TokenVariant {
        Text(String, String),
        Variable { name: String },
    }
    type TokenRecord = (String, String, BTreeMap<String, Vec<TokenVariant>>);
    rejected!(
        TokenRecord,
        "@article{k,f={x}}",
        "invalid type: tuple variant, expected token as tuple variant"
    );
    rejected!(
        TokenRecord,
        "@article{k,f=x}",
        "invalid type: struct variant, expected token as struct variant"
    );

    // The mandatory macro adapter has a separate path from optional macros.
    #[derive(Debug, Deserialize)]
    enum MacroPair {
        Macro(String, Vec<Token>),
    }
    for body in ["", "x", "x=", "x={x}", "x={x}#"] {
        check(
            from_str::<Vec<MacroPair>>(&format!("@string({body}")).unwrap_err(),
            "unclosed '('",
            Category::Eof,
        );
    }
    check(
        from_str::<Vec<MacroPair>>("@string{=}").unwrap_err(),
        "expected identifier",
        Category::Syntax,
    );
}

#[test]
fn rejected_empty_macro_keeps_visitor_error() {
    #[derive(Debug)]
    struct RejectNone;
    impl<'de> Deserialize<'de> for RejectNone {
        fn deserialize<D: serde::Deserializer<'de>>(de: D) -> Result<Self, D::Error> {
            struct Visitor;
            impl<'de> serde::de::Visitor<'de> for Visitor {
                type Value = RejectNone;
                fn expecting(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                    f.write_str("a nonempty macro")
                }
                fn visit_none<E: serde::de::Error>(self) -> Result<Self::Value, E> {
                    Err(E::custom("empty macro rejected"))
                }
            }
            de.deserialize_option(Visitor)
        }
    }
    #[derive(Debug, Deserialize)]
    enum Macro {
        Macro(RejectNone),
    }
    for input in ["@string{}", "@string{)"] {
        check(
            from_str::<Vec<Macro>>(input).unwrap_err(),
            "empty macro rejected",
            Category::Data,
        );
    }
}

#[test]
fn conversion_errors_survive_propagation() {
    type TextRecord = (String, String, BTreeMap<String, String>);
    for value in ["undefined", "{prefix} # undefined"] {
        let input = format!("@article{{k,f={value}}}");
        check(
            from_str::<Vec<TextRecord>>(&input).unwrap_err(),
            "expected text, got unresolved macro undefined",
            Category::Data,
        );
        check(
            Deserializer::from_str(&input)
                .into_iter::<TextRecord>()
                .next()
                .unwrap()
                .unwrap_err(),
            "expected text, got unresolved macro undefined",
            Category::Data,
        );
        check(
            Deserializer::from_str(&input)
                .into_iter_regular_entry::<TextRecord>()
                .next()
                .unwrap()
                .unwrap_err(),
            "expected text, got unresolved macro undefined",
            Category::Data,
        );
    }
    let utf8 = "invalid utf-8 sequence of 1 bytes from index 0";
    check(
        SliceReader::new(b"\xff").identifier().unwrap_err(),
        utf8,
        Category::Data,
    );
    for input in [
        b"@\xff{}".as_slice(),
        b"@article{\xff}",
        b"@article{k,\xff={x}}",
    ] {
        check(
            from_bytes::<IgnoredAny>(input).unwrap_err(),
            utf8,
            Category::Data,
        );
        check(
            from_bytes::<Vec<Entry>>(input).unwrap_err(),
            utf8,
            Category::Data,
        );
    }
    for input in [
        b"@article{k,f={\xff}}".as_slice(),
        b"@article{k,f={x}#{\xff}}",
    ] {
        check(
            from_bytes::<Vec<TextRecord>>(input).unwrap_err(),
            utf8,
            Category::Data,
        );
        // Byte text remains valid syntax when no UTF-8 conversion is requested.
        assert!(from_bytes::<IgnoredAny>(input).is_ok());
    }
    check(
        from_bytes::<Vec<Entry>>(b"@comment{\xff}").unwrap_err(),
        utf8,
        Category::Data,
    );
    check(
        from_bytes::<Vec<Entry>>(b"@preamble{{\xff}}").unwrap_err(),
        utf8,
        Category::Data,
    );
}

#[test]
fn serialization_diagnostics() {
    check(
        to_string(&true).unwrap_err(),
        "invalid serialization format: bibliography as bool",
        Category::Data,
    );
    check(
        to_string(&[true]).unwrap_err(),
        "invalid serialization format: entry as bool",
        Category::Data,
    );
    check(
        to_string(&[("article", "k", [("f", true)])]).unwrap_err(),
        "invalid serialization format: value as bool",
        Category::Data,
    );
    check(
        to_string(&[("article", "k")]).unwrap_err(),
        "regular entry from tuple not of length 3",
        Category::Data,
    );
    check(
        <Error as serde::ser::Error>::custom("serialization rejected input"),
        "serialization rejected input",
        Category::Data,
    );
    #[derive(Serialize)]
    enum BadToken {
        Bad(String),
    }
    check(
        to_string(&[("article", "k", [("f", vec![BadToken::Bad("x".into())])])]).unwrap_err(),
        "invalid serialization format: invalid token variant 'Bad'",
        Category::Data,
    );
    for (entry_type, key, field, value, message) in [
        (
            "comment",
            "k",
            "f",
            "x",
            "IO error: invalid entry type: 'comment'",
        ),
        ("article", ",", "f", "x", "IO error: invalid entry key: ','"),
        ("article", "k", ",", "x", "IO error: invalid field key: ','"),
        (
            "article",
            "k",
            "f",
            "{",
            "IO error: unbalanced text token: '{'",
        ),
    ] {
        check(
            to_string(&[(entry_type, key, [(field, value)])]).unwrap_err(),
            message,
            Category::Io,
        );
    }
    #[derive(Serialize)]
    enum Variable {
        Variable(&'static str),
    }
    check(
        to_string(&[("article", "k", [("f", [Variable::Variable("1x")])])]).unwrap_err(),
        "IO error: invalid variable: '1x'",
        Category::Io,
    );
    check(
        to_string(&[("article", "k", [("f", serde_bytes::Bytes::new(b"\xff"))])]).unwrap_err(),
        "invalid serialization format: text token as bytes",
        Category::Data,
    );

    struct BrokenWriter;
    impl io::Write for BrokenWriter {
        fn write(&mut self, _: &[u8]) -> io::Result<usize> {
            Err(io::Error::other("writer failed"))
        }
        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }
    check(
        to_writer(BrokenWriter, &[("article", "k", [("f", "x")])]).unwrap_err(),
        "IO error: writer failed",
        Category::Io,
    );
}
