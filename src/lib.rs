pub mod de;
pub mod reader;

pub(crate) mod naming;

/// Representations of BibTex components.
pub mod value;

/// Error types for parsing and conversion.
pub mod error;

/// Reader for lower-level document parsing.
pub mod abbrev;

/// Fundamental parsers.
pub mod parse;

// re-exports
pub use value::{Entry, Event};
