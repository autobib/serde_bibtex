mod macros;
mod read;

pub use macros::MacroDictionary;
#[cfg(test)]
use read::BibtexReadInner;
pub use read::{BibtexRead, EntryDelimiter, SliceReader, StrReader, TextDelimiter};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::Category;

    #[test]
    fn standalone_grammar_eof_has_no_entry_context() {
        for error in [
            StrReader::new("").variable().unwrap_err(),
            StrReader::new("").macro_variable_opt().unwrap_err(),
            StrReader::new(",").field_or_terminal().unwrap_err(),
        ] {
            assert_eq!(
                error.to_string(),
                "unexpected end of input; expected identifier"
            );
            assert_eq!(error.classify(), Category::Eof);
        }
        let error = StrReader::new("{x}#").ignore_value().unwrap_err();
        assert_eq!(error.to_string(), "unexpected end of input; expected value");
        assert_eq!(error.classify(), Category::Eof);
    }
}
