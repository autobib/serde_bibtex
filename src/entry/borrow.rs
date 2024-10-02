use serde::{Deserialize, Serialize};

/// A raw token.
#[derive(Debug, PartialEq, Deserialize, Serialize)]
pub enum Token<'a> {
    /// A `variable` token
    Variable(&'a str),
    /// A `{text}` token
    Text(&'a str),
}

/// An entry which borrows as much as possible from the underlying record.
#[derive(Debug, PartialEq, Deserialize, Serialize)]
pub enum BorrowEntry<'a> {
    /// A regular entry
    Regular {
        /// The entry type
        entry_type: &'a str,
        /// The entry key
        entry_key: &'a str,
        /// The unordered list of fields
        fields: Vec<(&'a str, Vec<Token<'a>>)>,
    },
    /// A macro entry
    Macro(Option<(&'a str, Vec<Token<'a>>)>),
    /// A comment entry
    Comment(&'a str),
    /// A preamble
    Preamble(Vec<Token<'a>>),
}
