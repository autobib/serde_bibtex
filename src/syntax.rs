//! # Description of the bibliography syntax
//! The goal of this module is to give an explicit description of the grammar accepted by this
//! crate. For other grammars, see for example the [btparse
//! documentation](https://metacpan.org/dist/Text-BibTeX/view/btparse/doc/btparse.pod).
//! For an informal description of the `.bib` grammar, visit the documentation for the [de
//! module](de).
//!
//! Generally speaking, we attempt to align with the grammar accepted by
//! [biber](https://en.wikipedia.org/wiki/Biber_(LaTeX)), extended to handle ASCII-compatible
//! non-UTF-8 input where sensible. However, biber has certain idiosyncracies that we intentionally
//! do not support. Jump to the [comparisons](#grammar-comparisons) section for an informal discussion
//! of the differences with other bibtex-compatible programs.
//!
//! ## Structure of a bibliography
//! ### Whitespace, comments, and junk characters.
//! 1. Whitespace is defined as any ASCII char accepted by the
//!    [`is_ascii_whitespace`](https://doc.rust-lang.org/std/primitive.u8.html#method.is_ascii_whitespace)
//!    method.
//!    ```ignore
//!    ws = _{ (" " |  "\t" | "\n" | "\r" | "\x0C" )+ }
//!    ```
//! 2. A TeX Comment is started by a `%` symbol and terminated by a newline `\n`.
//!    ```ignore
//!    tex_comment = _{ "%" ~ (!"\n" ~ ANY)* ~ "\n" }
//!    ```
//! 3. Whitespace and TeX comments can be combined to match ignored characters, which is any
//!    sequence of whitespace and TeX comments.
//!    ```ignore
//!    ign = _{ (tex_comment | ws)* }
//!    ```
//! 4. Junk characters are any characters which are either commented or are not `@`.
//!    ```ignore
//!    junk = _{ (tex_comment | !("@" | "%") ~ ANY)* }
//!    ```
//!
//! ### Identifiers
//! 1. An identifier is any UTF-8 character which is not ASCII, or a printable ASCII character
//!    which is not one of the literal characters `{}(),=\#%"`.
//!    ```ignore
//!    identifier = _{ (!('\x00'..'\x20' | "{" | "}" | "(" | ")" | "," | "=" | "\\" | "#" | "%" | "\"" | "\x7f") ~ ANY)+}
//!    ```
//! 2. A variable is an identifier which can be used in macro expansions. The syntax is the same as
//!    an identifier, except additionally it cannot begin with an ASCII digit.
//!    ```ignore
//!    variable = @{ !ASCII_DIGIT ~ identifier }
//!    ```
//! 3. An entry type, entry key, and field key are all parsed as identifiers.
//!    ```ignore
//!    entry_type = @{ identifier }
//!    entry_key = @{ identifier }
//!    field_key = @{ identifier }
//!    ```
//!
//! ### Field tokens and values.
//! 1. A numeric token is a sequence of digits
//!    ```ignore
//!    token_number = @{ ASCII_DIGIT+ }
//!    ```
//! 2. A balanced token is a sequence of characters such that the brackets `{}` are balanced.
//!    ```ignore
//!    balanced = _{ "{" ~ balanced* ~ "}" | (!("{" | "}") ~ ANY) }
//!    token_curly = @{ balanced* }
//!    ```
//! 3. A quoted token is a sequence of characters delimited by `"` such that the brackets `{}`
//!    are balanced. The closing `"` must not be captured within any brackets `{}`.
//!    ```ignore
//!    quoted = _{ "{" ~ balanced* ~ "}" | (!("{" | "}" | "\"") ~ ANY) }
//!    token_quoted = @{ quoted* }
//!    ```
//! 4. A token can be any of the above, or also a variable.
//!    ```ignore
//!    token = _{ token_number | "{" ~ token_curly ~ "}" | "\"" ~ token_quoted ~ "\"" | variable }
//!    ```
//! 5. A value is a sequence of tokens delimited by `#` and separated possibly by ignored
//!    characters.
//!    ```ignore
//!    value = { token ~ (ign ~ "#" ~ ign ~ token)* }
//!    ```
//!
//! ### Comment entry
//! 1. A comment entry is essentially parsed as a text token, except in place of quotes we allow
//!    delimitation by round brackets. Similarly to the quoted text token, it is terminated by a closing `)`
//!    which is not enclosed by curly brackets. It is identified by a case-insensitive `comment` entry type.
//!    ```ignore
//!    round = _{ "{" ~ balanced* ~ "}" | (!("{" | "}" | ")") ~ ANY) }
//!    token_round = @{ round* }
//!    comment_entry_type = _{ ^"comment" ~ ign }
//!    entry_comment = { comment_entry_type ~ ( "{" ~ token_curly ~ "}" | "(" ~ token_round ~ ")" ) }
//!    ```
//!
//! ### Preamble entry
//! 1. A preamble entry contains only a value and is identified by a case-insensitive `preamble`
//!    entry type.
//!    ```ignore
//!    preamble_contents = _{ ign ~ value ~ ign }
//!    preamble_entry_type = _{ ^"preamble" ~ ign }
//!    entry_preamble = { preamble_entry_type ~ ( "{" ~ preamble_contents ~ "}" | "(" ~ preamble_contents ~ ")" ) }
//!    ```
//!
//! ### Macro entry
//! 1. A macro entry consists of a variable and a value, separated by a `=` character.
//!    Note that a macro can optionally have empty contents, and if it is not empty, it can optionally
//!    have a trailing comma.
//!    ```ignore
//!    macro_contents = _{ (ign ~ variable ~ ign ~ "=" ~ ign ~ value ~ ign ~ ","?)? ~ ign }
//!    macro_entry_type = _{ ^"string" ~ ign }
//!    entry_macro = { macro_entry_type ~ ("{" ~ macro_contents ~ "}" | "(" ~ macro_contents ~ ")") }
//!    ```
//!
//! ### Regular entry
//! 1. The basic component of a regular entry is the field. A field consists of a field key and a
//!    value, separated by an "=". Note the similarity to the macro entry: however, the field key
//!    is permitted to start with an ASCII digit.
//!    ```ignore
//!    field = _{ ign ~ "," ~ ign ~ field_key ~ ign ~ "=" ~ ign ~ value }
//!    ```
//! 2. The bracketed component of a regular entry consists of an entry key, followed by a list of
//!    fields (possibly none), followed by an optional comma.
//!    ```ignore
//!    regular_entry_contents = _{ ign ~ entry_key ~ field* ~ ign ~ ","? ~ ign }
//!    ```
//! 3. A regular entry then consists of the entry type along with the contents of the entry,
//!    delimieted by brackets.
//!    ```ignore
//!    entry_regular = { entry_type ~ ign ~ ("{" ~ regular_entry_contents ~ "}" | "(" ~ regular_entry_contents ~ ")") }
//!    ```
//!
//! ### Bibliography
//! 1. An entry is any one of the above cases (comment, preamble, macro, or regular) preceded by an
//!    `@` symbol.
//!    ```ignore
//!    entry = { "@" ~ ign ~ (entry_comment | entry_preamble | entry_macro | entry_regular) }
//!    ```
//! 2. A bibliography is a possibly empty list of entries, separated by junk characters.
//!    ```ignore
//!    bib = _{ SOI ~ junk ~ (entry ~ junk)* ~ EOI }
//!    ```
//!
//!
//! ## Grammar comparisons
//!
//! ### Differences from biber
//! 1. A field key is permitted to start with an ASCII digit.
//! 2. We do not skip chars following `\` and `'`. When biber encounters one of these characters,
//!    it consumes the following character and counts it as whitespace. For instance, biber
//!    considers `@ '%article` to be equivalent to `@article`, since the `%` character is ignored since
//!    it follows `'` and does not begin a comment.
//! 4. We treat `comment` entries delimited by `()` in the same way as quoted text fields. This is
//!    more flexible than biber, which considers a closing `)` to terminate the comment field,
//!    regardless of the current depth of `{}` brackets.
//! 5. A field key allowed to start with digit. The only place we do not permit digits is at the
//!    beginning of a variable, so that a variable can be unambiguously distinguished from an
//!    unquoted number.
//!
//! ### Differences from bibtex
//! 1. Bibtex does not support `%`-style comments.
//! 2. Bibtex does not capture `@comment` strings: instead, upon reading an `@comment` entry, it
//!    immediately resets and applies 'junk' parsing. For example
//!    ```bib
//!    @comment{@article}
//!    ```
//!    will result in a parse error, since the `@comment` is discarded, then `{` is discarded as a
//!    junk character, then `@article` is parsed to begin a new entry, and `}` then results in an error.
//! 3. Bibtex does not support unicode.
//! 4. The only disallowed printable ASCII character in an entry key is `,`
//!
//! ## More flexible syntax?
//! The syntax could intentionally be made more flexible while still accepting all files satisfying
//! the current grammar. However, we do not want to promote proliferation of `.bib` files that are
//! incompatible with other more well-established tools.
use pest_derive::Parser;

/// A simple automatically derived pest parser.
#[derive(Parser)]
#[grammar = "syntax/bibtex.pest"] // relative to src
pub struct BibtexParser;

#[cfg(test)]
mod tests {
    use super::*;
    use pest::Parser;

    #[test]
    fn test_parse() {
        let input = r#"
            @article (2:k,
               @et= { Text} #
                1234,
            )
            @article {??,1={T} # var,}
            @article {??,1={T} # var,title = "{"}"}
            @a{k}
            @string{k=1234}
            @string{k=1 # {Text} # var,}
            @comment{{bal}{anced@@@}}
            @preamble{ {Text} # expand # {"}}
        "#;

        let parsed = BibtexParser::parse(Rule::bib, input);

        assert!(parsed.is_ok());
    }
}
