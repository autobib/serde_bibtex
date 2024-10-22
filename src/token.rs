//! # Types to represent various components of a bibliography.
//!
//! This module implements various validated types to represent the corresponding components of a BibTeX bibliography.
//! The constructors are fallible, and resulting in the appropriate [`TokenError`] if invalid.
//!
//! In the below table, the BibTeX component refers to the relevant entry in the
//! [`syntax`](crate::syntax) module documentation.
//!
//! | Type           | BibTeX component                              |
//! |----------------|-----------------------------------------------|
//! | [`Identifier`] | `identifier`                                  |
//! | [`EntryKey`]   | `entry_key`                                   |
//! | [`EntryType`]  | `entry_type`                                  |
//! | [`FieldKey`]   | `field_key`                                   |
//! | [`Variable`]   | `variable`                                    |
//! | [`Token`]      | `token`                                       |
//! | [`Text`]       | `token_number`, `token_curly`, `token_quoted` |
mod error;
mod types;
mod validate;

pub use error::*;
pub use types::*;
pub use validate::*;
