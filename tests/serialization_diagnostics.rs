//! Serialization error sites, including custom struct and enum representations.
use serde::{Serialize, ser::SerializeStruct};
use serde_bibtex::{Error, error::Category, to_string};

fn data<T: Serialize>(value: T, message: &str) {
    let error = to_string(&value).unwrap_err();
    assert_eq!(error.to_string(), message);
    assert_eq!(error.classify(), Category::Data);
}

#[test]
fn entry_shapes() {
    #[derive(Serialize)]
    enum Unit {
        Unknown,
    }
    #[derive(Serialize)]
    enum Newtype {
        Unknown(String),
    }
    #[derive(Serialize)]
    enum Tuples {
        Regular(String, String),
        Macro(String, String, String),
        Comment(String, String),
        Preamble(String, String),
        Unknown(String, String),
    }
    #[derive(Serialize)]
    enum Struct {
        Comment { text: String },
    }
    data([Unit::Unknown], "Unexpected enum variant Unknown");
    data(
        [Newtype::Unknown(String::new())],
        "Invalid variant name `Unknown`",
    );
    data(
        [Tuples::Regular(String::new(), String::new())],
        "regular entry from tuple not of length 3",
    );
    data(
        [Tuples::Macro(String::new(), String::new(), String::new())],
        "macro entry from tuple not of length 2",
    );
    data(
        [Tuples::Comment(String::new(), String::new())],
        "tuple serialization not supported for comment",
    );
    data(
        [Tuples::Preamble(String::new(), String::new())],
        "tuple serialization not supported for preamble",
    );
    data(
        [Tuples::Unknown(String::new(), String::new())],
        "unrecognized entry variant",
    );
    data(
        [Struct::Comment {
            text: String::new(),
        }],
        "struct serialization only supported for regular entry",
    );

    #[derive(Serialize)]
    struct Pair(&'static str, &'static str);
    #[derive(Serialize)]
    struct Triple(&'static str, &'static str, &'static str);
    #[derive(Serialize)]
    enum Regular<T> {
        Regular(T),
    }
    #[derive(Serialize)]
    enum Macro<T> {
        Macro(T),
    }
    data(
        [Pair("article", "key")],
        "regular entry from tuple not of length 3",
    );
    data(
        [Regular::Regular(Pair("article", "key"))],
        "regular entry from tuple not of length 3",
    );
    data(
        [Regular::Regular(("article", "key"))],
        "regular entry from tuple not of length 3",
    );
    data(
        [Macro::Macro(("x", "y", "z"))],
        "macro entry from tuple not of length 2",
    );
    data(
        [Macro::Macro(Triple("x", "y", "z"))],
        "macro entry from tuple not of length 2",
    );
    data(
        [("article", "key", [("x", "y", "z")])],
        "key value from tuple not of length 2",
    );
    data(
        [("article", "key", [Triple("x", "y", "z")])],
        "key value from tuple not of length 2",
    );
    data(
        [Regular::Regular(true)],
        "invalid serialization format: regular entry as bool",
    );
    data(
        [Macro::Macro(true)],
        "invalid serialization format: macro entry as bool",
    );
}

struct Fields(&'static [&'static str]);

impl Serialize for Fields {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut record = serializer.serialize_struct("Record", self.0.len())?;
        for field in self.0 {
            match *field {
                "entry_type" => record.serialize_field(field, "article")?,
                "entry_key" => record.serialize_field(field, "key")?,
                "fields" => record.serialize_field(field, &[("f", "v")])?,
                _ => record.serialize_field(field, "unexpected")?,
            }
        }
        record.end()
    }
}

#[test]
fn entry_struct_fields() {
    for (fields, message) in [
        (&[][..], "Missing entry type"),
        (&["entry_type"][..], "Missing entry key"),
        (&["entry_type", "entry_key"][..], "Missing fields"),
        (&["unexpected"][..], "Unexpected struct field unexpected"),
        (&["entry_type", "entry_type"][..], "Duplicate entry type"),
        (&["entry_key", "entry_key"][..], "Duplicate entry key"),
        (&["fields", "fields"][..], "Duplicate fields"),
    ] {
        data([Fields(fields)], message);
    }
}

#[test]
fn unsupported_representations_at_each_serializer() {
    data(
        [(true, "key", [("f", "v")])],
        "invalid serialization format: entry type as bool",
    );
    data(
        [("article", true, [("f", "v")])],
        "invalid serialization format: entry key as bool",
    );
    data(
        [("article", "key", true)],
        "invalid serialization format: regular entry fields as bool",
    );
    data(
        [("article", "key", [true])],
        "invalid serialization format: field key-value pair as bool",
    );
    data(
        [("article", "key", [(true, "v")])],
        "invalid serialization format: field key as bool",
    );
    data(
        [("article", "key", [("f", [true])])],
        "invalid serialization format: value token as bool",
    );
    #[derive(Serialize)]
    enum Token {
        Text(bool),
        Variable(bool),
    }
    data(
        [("article", "key", [("f", [Token::Text(true)])])],
        "invalid serialization format: text token as bool",
    );
    data(
        [("article", "key", [("f", [Token::Variable(true)])])],
        "invalid serialization format: variable token as bool",
    );
}

#[test]
fn primitive_rejection_messages() {
    macro_rules! primitive {
        ($value:expr, $name:literal) => {
            data(
                $value,
                concat!("invalid serialization format: bibliography as ", $name),
            );
        };
    }
    primitive!(0i8, "i8");
    primitive!(0i16, "i16");
    primitive!(0i32, "i32");
    primitive!(0i64, "i64");
    primitive!(0u8, "u8");
    primitive!(0u16, "u16");
    primitive!(0u32, "u32");
    primitive!(0u64, "u64");
    primitive!(0f32, "f32");
    primitive!(0f64, "f64");
    primitive!('x', "char");
    primitive!("x", "str");
    primitive!(serde_bytes::Bytes::new(b"x"), "bytes");
    primitive!(None::<String>, "option");
    primitive!(Some("x"), "option");
    primitive!((), "unit");

    #[derive(Serialize)]
    struct Unit;
    #[derive(Serialize)]
    struct Struct {
        value: bool,
    }
    #[derive(Serialize)]
    struct Tuple(bool, bool);
    #[derive(Serialize)]
    enum Enum {
        Unit,
        Newtype(bool),
        Tuple(bool, bool),
        Struct { value: bool },
    }
    primitive!(Unit, "unit struct");
    primitive!(Struct { value: true }, "struct");
    primitive!(Enum::Unit, "unit variant");
    primitive!(Enum::Newtype(true), "newtype variant");
    primitive!(Enum::Tuple(true, false), "tuple variant");
    primitive!(Enum::Struct { value: true }, "struct variant");
    primitive!(std::collections::BTreeMap::<String, String>::new(), "map");
    data(
        [vec![true]],
        "invalid serialization format: entry as sequence",
    );
    data(
        [("article", "key", [("f", [Tuple(true, false)])])],
        "invalid serialization format: value token as tuple struct",
    );
    data(
        [("article", "key", [("f", [(true, false)])])],
        "invalid serialization format: value token as tuple",
    );
}

#[test]
fn source_free_errors() {
    let bytes = vec![0xff];
    let utf8 = core::str::from_utf8(&bytes).unwrap_err();
    for error in [
        Error::from(utf8),
        Error::from(serde_bibtex::token::ConversionError::InvalidUtf8(utf8)),
    ] {
        assert_eq!(
            error.to_string(),
            "invalid utf-8 sequence of 1 bytes from index 0"
        );
        assert_eq!(error.classify(), Category::Data);
    }
    let error = Error::from(serde_bibtex::token::ConversionError::UnexpandedMacro(
        "unknown".into(),
    ));
    assert_eq!(
        error.to_string(),
        "expected text, got unresolved macro unknown"
    );
    assert_eq!(error.classify(), Category::Data);
    let error = Error::from(std::io::Error::other("writer failed"));
    assert_eq!(error.to_string(), "IO error: writer failed");
    assert_eq!(error.classify(), Category::Io);
}
