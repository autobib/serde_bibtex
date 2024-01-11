use serde::Deserialize;
use std::fmt;

/// Characters not permitted in an EntryKey
pub const ENTRY_KEY_DISALLOWED_CHARS: &'static str = "{}(),= \t\n\\#%'\"";

/// Entry key, such as `article` in `@article{...`.
///
/// Case-insensitive, can use any unicode character except characters in
/// [`ENTRY_KEY_DISALLOWED_CHARS`].
pub struct EntryKey;
