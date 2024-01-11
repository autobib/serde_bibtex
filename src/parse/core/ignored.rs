use memchr::{memchr, memchr2};
use std::str::from_utf8;

/// Ignore junk characters betwee entries.
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

use trace::trace;
trace::init_depth_var!();
#[trace]
pub fn ignore_junk(input: &str) -> &str {
    // SAFETY: ignore_junk_bytes only truncates on ascii `@`, `%`, and `\n`.
    from_utf8(ignore_junk_bytes(input.as_bytes())).unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ignore_junk() {
        assert_eq!(ignore_junk_bytes(b"junk"), &b""[..]);
        assert_eq!(ignore_junk_bytes(b"@art"), &b"@art"[..]);
        assert_eq!(ignore_junk_bytes(b"%@@\n@a"), &b"@a"[..]);
        assert_eq!(ignore_junk_bytes(b"\nignored @a"), &b"@a"[..]);
        assert_eq!(ignore_junk_bytes(b"%@a"), &b""[..]);

        assert_eq!(ignore_junk("🍄@a"), "@a");
        assert_eq!(ignore_junk("@🍄a"), "@🍄a");
        assert_eq!(ignore_junk("%🍄\n@a"), "@a");
    }
}
