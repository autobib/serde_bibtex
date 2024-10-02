//! # Built-in types
mod borrow;
mod owned;

pub use borrow::{BorrowEntry, Token};
pub use owned::Entry;

/// A bibliography of owned entries.
pub type OwnedBibliography = Vec<Entry>;

/// A bibliography of borrowed entries.
pub type RawBibliography<'r> = Vec<BorrowEntry<'r>>;
