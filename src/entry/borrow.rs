use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Deserialize, Serialize)]
pub enum Token<'a> {
    Variable(&'a [u8]),
    Text(&'a [u8]),
}

#[derive(Debug, PartialEq, Deserialize, Serialize)]
pub enum BorrowEntry<'a> {
    Regular {
        entry_type: &'a str,
        entry_key: &'a str,
        fields: Vec<(&'a str, Vec<Token<'a>>)>,
    },
    Macro(Option<(&'a str, Vec<Token<'a>>)>),
    Comment(&'a [u8]),
    Preamble(Vec<Token<'a>>),
}
