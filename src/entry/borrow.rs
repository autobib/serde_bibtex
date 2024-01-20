use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Deserialize, Serialize)]
pub enum BorrowToken<'a> {
    Variable(&'a [u8]),
    Text(&'a [u8]),
}

#[derive(Debug, PartialEq, Deserialize, Serialize)]
pub struct BorrowRegularEntry<'a> {
    pub entry_type: &'a str,
    pub entry_key: &'a str,
    pub fields: Vec<(&'a str, Vec<BorrowToken<'a>>)>,
}

#[derive(Debug, PartialEq, Deserialize, Serialize)]
pub enum BorrowEntry<'a> {
    Regular(BorrowRegularEntry<'a>),
    Macro(Option<(&'a str, Vec<BorrowToken<'a>>)>),
    Comment(&'a [u8]),
    Preamble(Vec<BorrowToken<'a>>),
}
