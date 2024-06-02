mod borrow;
mod owned;

pub use borrow::{BorrowEntry, Token};
pub use owned::Entry;

pub type OwnedBibliography = Vec<Entry>;
pub type RawBibliography<'r> = Vec<BorrowEntry<'r>>;
