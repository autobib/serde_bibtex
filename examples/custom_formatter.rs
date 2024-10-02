//! # A custom formatter implementation
//!
//! This example demonstrates a custom formatter implementation which normalizes entry keys and entry values.
use std::collections::BTreeMap;
use std::io;

use serde::{Deserialize, Serialize};
use serde_bibtex::{
    from_str,
    ser::{Formatter, Serializer, ValidatingFormatter},
    Error,
};

/// An implementation of [`Formatter`] which converts the entry type to lowercase when it is
/// written.
#[derive(Default)]
struct NormalizingFormatter {
    buffer: String,
}

impl Formatter for NormalizingFormatter {
    #[inline]
    fn write_regular_entry_type<W>(&mut self, writer: &mut W, entry_type: &str) -> io::Result<()>
    where
        W: ?Sized + io::Write,
    {
        self.buffer = entry_type.to_lowercase();
        writer.write_all(b"@")?;
        writer.write_all(self.buffer.as_bytes())
    }
}

// A basic record type
#[derive(Deserialize, Serialize)]
struct Record {
    entry_type: String,
    entry_key: String,
    fields: BTreeMap<String, String>,
}

fn main() -> Result<(), Error> {
    let input = "
        @ARTICLE{key,
          author = {Author},
          year = 2023
        }
    ";
    let bibliography: Vec<Record> = from_str(input)?;
    let mut stdout = io::stdout();

    let formatter = ValidatingFormatter::new(NormalizingFormatter::default());
    let mut ser = Serializer::new_with_formatter(&mut stdout, formatter);

    bibliography.serialize(&mut ser)?;

    Ok(())
}
