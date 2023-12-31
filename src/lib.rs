//! A BibTex parser and deserializer.
//!
//! # Description
//!
//! A fast zero-copy BibTex parser and deserializer built on [nom]. For basic parsing types, see
//! - [`Bibliography`]: convenient representation of a BibTex file
//! - [`Reader`]: parse a BibTex file directly
//! - TODO: [`Event`] and [`Entry`] Deserialization
//!
//! # Examples
//! ```
//! use serde_bibtex::Bibliography;
//!
//! let bibfile = r#"
//!     @string{A = "Author"}
//!
//!     @article{key,
//!         author = {One, } # Author,
//!         title = "Example Title",
//!         year = 2023,
//!     }
//! "#;
//!
//! let bibliography = Bibliography::from_str(bibfile);
//! ```

// mod bibliography;
// pub mod reader;
// pub use bibliography::Bibliography;
// pub use reader::Reader;

/// Representations of BibTex components.
pub mod bib;

/// Error types for parsing and conversion.
pub mod error;

/// Reader for lower-level document parsing.
pub mod abbrev;

/// Fundamental parsers.
pub mod parse;

pub mod de;

// re-exports
pub use bib::{Entry, Event};
