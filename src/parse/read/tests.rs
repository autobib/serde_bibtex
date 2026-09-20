//! Reader regression tests for the private primitives.
use core::ops::Range;

use serde::de::IgnoredAny;

use super::{BibtexRead, BibtexReadInner, SliceReader, StrReader, TextDelimiter};
use crate::{Error, error::Category, from_bytes, from_str};

fn check(error: Error, message: &str, category: Category) {
    assert_eq!(error.to_string(), message);
    assert_eq!(error.classify(), category, "{message}");
}

fn span(error: Error, expected: Range<usize>) {
    assert_eq!(error.span(), Some(expected), "{error}");
    assert!(!error.to_string().contains(" at line "));
}

#[test]
fn text_delimiters_respect_braces_and_unicode() {
    for (until, input) in [
        (TextDelimiter::Brace, "é{{}}中}"),
        (TextDelimiter::Quote, "é{\"}中\""),
        (TextDelimiter::Parenthesis, "é{)}中)"),
    ] {
        let expected = &input[..input.len() - 1];
        let mut reader = StrReader::new(input);
        assert_eq!(
            reader.text_until(until).unwrap().into_str().unwrap(),
            expected
        );
        assert_eq!(reader.byte_offset(), Some(expected.len()));
        let mut reader = SliceReader::new(input.as_bytes());
        assert_eq!(
            reader.text_until(until).unwrap().into_bytes(),
            expected.as_bytes()
        );
        assert_eq!(reader.byte_offset(), Some(expected.len()));
    }
}

#[test]
fn ascii_consumption_checks_matches_and_eof() {
    fn check<'r>(mut reader: impl BibtexRead<'r>) {
        assert_eq!(reader.peek(), Some(b','));
        // SAFETY: all expected bytes in this test are ASCII.
        unsafe {
            assert!(!reader.consume_ascii(b'='));
            assert_eq!(reader.byte_offset(), Some(0));
            assert!(reader.consume_ascii(b','));
            assert_eq!(reader.byte_offset(), Some(1));
            assert!(!reader.consume_ascii(b','));
            assert!(!reader.consume_ascii(b'e'));
        }
        assert_eq!(reader.identifier().unwrap().into_inner(), "é");
        // SAFETY: the expected bytes are ASCII.
        unsafe {
            assert!(reader.consume_ascii(b'}'));
            assert!(!reader.consume_ascii(b'}'));
        }
        assert_eq!(reader.peek(), None);
        assert_eq!(reader.byte_offset(), Some(4));
    }

    check(StrReader::new(",é}"));
    check(SliceReader::new(",é}".as_bytes()));
}

#[test]
fn failed_text_scans_preserve_the_cursor() {
    for delimiter in [
        TextDelimiter::Brace,
        TextDelimiter::Quote,
        TextDelimiter::Parenthesis,
    ] {
        for input in ["é{unfinished", "é{{}unfinished"] {
            fn check<'r>(mut reader: impl BibtexRead<'r>, delimiter: TextDelimiter) {
                let error = reader.text_until(delimiter).unwrap_err();
                assert_eq!(error.span(), Some(2..3));
                assert_eq!(reader.byte_offset(), Some(0));
                assert_eq!(reader.peek(), Some(0xc3));
            }
            check(StrReader::new(input), delimiter);
            check(SliceReader::new(input.as_bytes()), delimiter);
        }
    }
}

proptest::proptest! {
    #[test]
    fn reader_primitives_preserve_character_boundaries(
        input in ".*",
        operations in proptest::collection::vec((0u8..9, 0u8..=127), 0..40),
    ) {
        let mut reader = StrReader::new(&input);
        for (operation, expected) in operations {
            let result = match operation {
                0 => { reader.peek(); Ok(()) }
                1 => {
                    // SAFETY: the generated expected byte is in the ASCII range.
                    unsafe { reader.consume_ascii(expected); }
                    Ok(())
                }
                2 => { reader.comment(); Ok(()) }
                3 => { reader.next_entry_or_eof(); Ok(()) }
                4 => reader.identifier().map(drop),
                5 => reader.number().map(drop),
                6 => reader.text_until(TextDelimiter::Brace).map(drop),
                7 => reader.text_until(TextDelimiter::Quote).map(drop),
                _ => reader.text_until(TextDelimiter::Parenthesis).map(drop),
            };
            let offset = reader.byte_offset().unwrap();
            proptest::prop_assert!(input.is_char_boundary(offset));
            if let Err(error) = result {
                proptest::prop_assert!(input.get(error.span().unwrap()).is_some());
            }
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
            StrReader::new(input)
                .text_until(TextDelimiter::Brace)
                .unwrap_err(),
            "unclosed '{'",
            Category::Eof,
        );
        check(
            SliceReader::new(input.as_bytes())
                .text_until(TextDelimiter::Brace)
                .unwrap_err(),
            "unclosed '{'",
            Category::Eof,
        );
    }
    for (until, input, message, eof) in [
        (TextDelimiter::Quote, "", "unclosed '\"'", true),
        (TextDelimiter::Quote, "{", "unclosed '{'", true),
        (TextDelimiter::Quote, "}", "unmatched closing '}'", false),
        (TextDelimiter::Parenthesis, "", "unclosed '('", true),
        (TextDelimiter::Parenthesis, "{", "unclosed '{'", true),
        (
            TextDelimiter::Parenthesis,
            "}",
            "unmatched closing '}'",
            false,
        ),
    ] {
        check(
            StrReader::new(input).text_until(until).unwrap_err(),
            message,
            if eof { Category::Eof } else { Category::Syntax },
        );
        check(
            SliceReader::new(input.as_bytes())
                .text_until(until)
                .unwrap_err(),
            message,
            if eof { Category::Eof } else { Category::Syntax },
        );
    }
}

#[test]
fn identifier_utf8_diagnostic() {
    let utf8 = "invalid utf-8 sequence of 1 bytes from index 0";
    check(
        SliceReader::new(b"\xff").identifier().unwrap_err(),
        utf8,
        Category::Data,
    );
}

#[test]
fn nested_openers_and_reader_offsets() {
    for (input, expected) in [
        ("", 0..0),
        ("text", 0..0),
        ("{a{b", 2..3),
        ("{a{b}", 0..1),
        ("{a}x{b{c}d", 4..5),
        ("{a{b}}", 0..0),
    ] {
        span(
            StrReader::new(input)
                .text_until(TextDelimiter::Brace)
                .unwrap_err(),
            expected.clone(),
        );
        span(
            SliceReader::new(input.as_bytes())
                .text_until(TextDelimiter::Brace)
                .unwrap_err(),
            expected.clone(),
        );
        for until in [TextDelimiter::Quote, TextDelimiter::Parenthesis] {
            span(
                StrReader::new(input).text_until(until).unwrap_err(),
                expected.clone(),
            );
            span(
                SliceReader::new(input.as_bytes())
                    .text_until(until)
                    .unwrap_err(),
                expected.clone(),
            );
        }
    }
    for input in [
        "@comment({a{b",
        "@comment{{a{b",
        "@preamble{\"{a{b",
        "@a{k,f={{a{b",
    ] {
        let start = input.rfind('{').unwrap();
        span(from_str::<IgnoredAny>(input).unwrap_err(), start..start + 1);
        span(
            from_bytes::<IgnoredAny>(input.as_bytes()).unwrap_err(),
            start..start + 1,
        );
    }
    for input in ["", ","] {
        let expected = 0..input.len();
        span(
            StrReader::new(input).identifier().unwrap_err(),
            expected.clone(),
        );
        span(
            SliceReader::new(input.as_bytes()).identifier().unwrap_err(),
            expected.clone(),
        );
        span(
            StrReader::new(input).number().unwrap_err(),
            expected.clone(),
        );
        span(
            SliceReader::new(input.as_bytes()).number().unwrap_err(),
            expected,
        );
    }
    span(StrReader::new(" %end").read_field_key().unwrap_err(), 5..5);
    span(StrReader::new(" x").skip_field_sep().unwrap_err(), 1..2);
    span(StrReader::new(" %end").skip_field_sep().unwrap_err(), 5..5);
    span(StrReader::new(" %end").read_text_token().unwrap_err(), 5..5);
    span(StrReader::new(" #").read_text_token().unwrap_err(), 1..2);
    span(
        StrReader::new("  macro ").read_text_token().unwrap_err(),
        2..7,
    );
    let mut reader = StrReader::new("{ok} {bad{inner");
    reader.read_text_token().unwrap();
    span(reader.read_text_token().unwrap_err(), 9..10);
    assert_eq!(reader.byte_offset(), Some(6)); // A scanner failure does not advance the cursor.
    span(
        StrReader::new("text}")
            .text_until(TextDelimiter::Quote)
            .unwrap_err(),
        4..5,
    );
    span(
        SliceReader::new(b"text}")
            .text_until(TextDelimiter::Parenthesis)
            .unwrap_err(),
        4..5,
    );
}

#[test]
fn reader_error_spans_preserve_the_start() {
    let mut reader = StrReader::new("123🍄");
    assert_eq!(reader.number().unwrap(), "123");
    span(reader.number().unwrap_err(), 3..7);
    assert_eq!(reader.byte_offset(), Some(3));

    // Byte spans stay byte spans, even within valid UTF-8 or a run of continuation bytes.
    for input in ["🍄".as_bytes(), b"\x80\x81\x82"] {
        let mut reader = SliceReader::new(input);
        span(reader.number().unwrap_err(), 0..1);
    }
}

#[test]
fn reader_error_span_defaults_and_override() {
    assert_eq!(StrReader::new("🍄").error_span(), Some(0..4));
    assert_eq!(SliceReader::new("🍄".as_bytes()).error_span(), Some(0..1));
    assert_eq!(StrReader::new("").error_span(), Some(0..0));
    assert_eq!(SliceReader::new(b"").error_span(), Some(0..0));
}

#[test]
fn unicode_reader_spans() {
    for ch in ['é', '中', '🍄'] {
        let input = format!("{ch}rest");
        for error in [
            StrReader::new(&input).number().unwrap_err(),
            StrReader::new(&input).skip_field_sep().unwrap_err(),
        ] {
            assert_eq!(
                input.get(error.span().unwrap()),
                Some(&input[..ch.len_utf8()])
            );
            span(error, 0..ch.len_utf8());
        }
        span(
            SliceReader::new(input.as_bytes()).number().unwrap_err(),
            0..1,
        );
    }
}

#[test]
fn identifier_utf8_spans() {
    for (bytes, expected) in [
        (b"a\xffz".as_slice(), 1..2),
        (b"a\xe2\x82", 1..3),
        (b"a\xe2\x82!", 1..3),
    ] {
        span(
            SliceReader::new(bytes).identifier().unwrap_err(),
            expected.clone(),
        );
    }
}
