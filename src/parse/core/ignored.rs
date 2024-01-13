use memchr::{memchr, memchr2};
use std::str::from_utf8;

/// Ignore junk characters between entries.
pub fn ignore_junk_bytes(input: &[u8]) -> &[u8] {
    let mut tail = input;

    loop {
        // Search for `@` or `%`.
        match memchr2(b'@', b'%', tail) {
            Some(idx) => {
                // realign to the matched index
                tail = &tail[idx..];

                if tail[0] == b'@' {
                    // we found an @, so we return it
                    return tail;
                } else {
                    // we found a %, so we skip to the the end of the line
                    match memchr(b'\n', tail) {
                        Some(idx) => {
                            tail = &tail[idx + 1..];
                        }
                        None => return &[],
                    }
                }
            }
            None => return &[],
        }
    }
}

/// Ignore whitespace and comments within entries.
///
/// Note that this uses the built-in `.is_ascii_whitespace` and in particular
/// unlike biber does not consider U+000B VERTICAL TAB to be whitespace.
pub fn ignore_comment_bytes(input: &[u8]) -> &[u8] {
    let mut pos = 0;
    loop {
        if pos == input.len() {
            return &[];
        }

        if input[pos].is_ascii_whitespace() {
            pos += 1;
        } else if input[pos] == b'%' {
            match memchr(b'\n', &input[pos + 1..]) {
                Some(offset) => {
                    // alignment math: skip the '%' and the '\n'
                    pos += offset + 2;
                }
                None => return &[],
            }
        } else {
            return &input[pos..];
        }
    }
}

pub fn ignore_junk(input: &str) -> &str {
    // SAFETY: ignore_junk_bytes only truncates on ascii `@`, `%`, and `\n`.
    from_utf8(ignore_junk_bytes(input.as_bytes())).unwrap()
}

pub fn ignore_comment(input: &str) -> &str {
    // SAFETY: ignore_comment_bytes only truncates on ascii whitespace
    // and therefore always has valid boundaries
    from_utf8(ignore_comment_bytes(input.as_bytes())).unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ignore_junk() {
        assert_eq!(ignore_junk_bytes(b"junk"), &b""[..]);
        assert_eq!(ignore_junk_bytes(b""), &b""[..]);
        assert_eq!(ignore_junk_bytes(b"@art"), &b"@art"[..]);
        assert_eq!(ignore_junk_bytes(b"%@@\n@a"), &b"@a"[..]);
        assert_eq!(ignore_junk_bytes(b"\nignored @a"), &b"@a"[..]);
        assert_eq!(ignore_junk_bytes(b"%@a"), &b""[..]);

        assert_eq!(ignore_junk("🍄@a"), "@a");
        assert_eq!(ignore_junk("@🍄a"), "@🍄a");
        assert_eq!(ignore_junk("%🍄\n@a"), "@a");
    }

    #[test]
    fn test_ignore() {
        assert_eq!(ignore_comment_bytes(b"%   a\n ab"), &b"ab"[..]);
        assert_eq!(ignore_comment_bytes(b"  %\na"), &b"a"[..]);
        // all valid whitespace chars
        assert_eq!(ignore_comment_bytes(b"\x09\x0a\x0c\x0d\x20b"), &b"b"[..]);
        // comments ignore everything, including invalid utf-8
        assert_eq!(ignore_comment_bytes(b"%\xa8!\xfd!\x7f!\nc"), &b"c"[..]);
        // we follow whatwg convention and do not consider U+000B VERTICAL TAB
        // to be ascii whitespace, unlike biber
        assert_eq!(ignore_comment_bytes(b"\x0b"), &b"\x0b"[..]);
        assert_eq!(ignore_comment_bytes(b""), &b""[..]);
    }
}
