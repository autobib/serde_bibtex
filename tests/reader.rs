//! Exercise the sealed reader API from outside the crate.
use std::collections::BTreeMap;

use serde::Deserialize;
use serde_bibtex::{
    BibtexRead, MacroDictionary, Result, SliceReader, StrReader,
    de::Deserializer,
    token::{Token, Variable},
};

type Record = (String, String, BTreeMap<String, String>);

#[derive(Debug, Deserialize, PartialEq)]
enum Entry {
    Macro,
    Comment(String),
    Preamble(String),
    Regular(Record),
}

fn deserialize<'r, R: BibtexRead<'r>, D: Deserialize<'r>>(reader: R) -> Result<D> {
    D::deserialize(&mut Deserializer::new(reader))
}

#[test]
fn public_trait_alone_drives_deserialization() {
    let input = "@string{prefix={hé}} @comment(note {nested} text) \
                 @preamble(prefix # \"!\") @article(k,title=prefix # {llo},year=2026,)";
    let expected = serde_bibtex::from_str::<Vec<Entry>>(input).unwrap();
    let actual: Vec<Entry> = deserialize(StrReader::new(input)).unwrap();
    assert_eq!(actual, expected);
    assert_eq!(actual[2], Entry::Preamble("hé!".into()));

    let entries = Deserializer::new(StrReader::new(input))
        .into_iter::<Entry>()
        .collect::<Result<Vec<_>>>()
        .unwrap();
    assert_eq!(entries, expected);

    let regular = Deserializer::new(StrReader::new(input))
        .into_iter_regular_entry::<Record>()
        .collect::<Result<Vec<_>>>()
        .unwrap();
    assert_eq!(regular.len(), 1);
    assert_eq!(regular[0].2["title"], "héllo");

    let reader = StrReader::new(input);
    serde::de::IgnoredAny::deserialize(&mut Deserializer::new(reader)).unwrap();

    let actual: Vec<Entry> = deserialize(SliceReader::new(input.as_bytes())).unwrap();
    assert_eq!(actual, expected);
}

#[test]
fn reader_accepts_external_macros_and_borrowed_output() {
    let mut macros = MacroDictionary::default();
    macros.insert(
        Variable::new("x").unwrap(),
        vec![Token::str("external").unwrap()],
    );
    let mut de = Deserializer::new_with_macros(StrReader::new("@a{k,f=x}"), macros);
    let records = Vec::<Record>::deserialize(&mut de).unwrap();
    assert_eq!(records[0].2["f"], "external");

    type BorrowedRecord<'r> = (&'r str, &'r str, BTreeMap<&'r str, &'r str>);
    let input = String::from("@article{κ,title={é中}}");
    let records: Vec<BorrowedRecord<'_>> = deserialize(StrReader::new(&input)).unwrap();
    assert_eq!(records[0].0, "article");
    assert_eq!(records[0].1, "κ");
    assert_eq!(records[0].2["title"], "é中");
}

#[test]
fn fixed_length_fields_do_not_need_to_request_the_end() {
    type OneField = (String, String, [(String, String); 1]);
    for input in ["@a{k,f={é}}", "@a(k,f={é},)"] {
        let entries: Vec<OneField> = deserialize(StrReader::new(input)).unwrap();
        assert_eq!(entries[0].2, [("f".into(), "é".into())]);
    }
}

#[test]
fn mutable_readers_support_borrowed_output_and_preserve_error_spans() {
    type BorrowedRecord<'r> = (&'r str, &'r str, BTreeMap<&'r str, &'r str>);
    let input = String::from("@article{κ,title={é中}}");
    let mut string_reader = StrReader::new(&input);
    let mut slice_reader = SliceReader::new(input.as_bytes());
    let strings: Vec<BorrowedRecord<'_>> = deserialize(&mut string_reader).unwrap();
    let bytes: Vec<BorrowedRecord<'_>> = deserialize(&mut &mut slice_reader).unwrap();
    assert_eq!(strings, bytes);
    assert_eq!(strings[0].2["title"], "é中");

    let input = "@article 🍄";
    let error = deserialize::<_, Vec<Record>>(&mut StrReader::new(input)).unwrap_err();
    assert_eq!(error.span(), Some(9..13));
    let error = deserialize::<_, Vec<Record>>(&mut SliceReader::new(input.as_bytes())).unwrap_err();
    assert_eq!(error.span(), Some(9..10));
}

#[test]
fn string_helpers_share_the_cursor_with_deserialization() {
    let mut reader =
        StrReader::new("@a{k,f={é}} % comment\n title = {中{文}} year=2026 note=\"a{b}\"");
    let entry = Deserializer::new(&mut reader)
        .into_iter_regular_entry::<Record>()
        .next()
        .unwrap()
        .unwrap();
    assert_eq!(entry.2["f"], "é");

    for (key, value) in [("title", "中{文}"), ("year", "2026"), ("note", "a{b}")] {
        assert_eq!(reader.read_field_key().unwrap().into_inner(), key);
        reader.skip_field_sep().unwrap();
        assert_eq!(reader.read_text_token().unwrap(), value);
    }
}
