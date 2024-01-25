mod biblatex;
mod borrow;
mod owned;

pub use borrow::{BorrowEntry, BorrowRegularEntry, BorrowToken};
pub use owned::{Entry, RegularEntry};

pub type OwnedBibliography = Vec<Entry>;
pub type RawBibliography<'r> = Vec<BorrowEntry<'r>>;
