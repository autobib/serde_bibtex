use std::io;

use crate::validate::{
    is_balanced, is_entry_key, is_field_key, is_regular_entry_type, is_variable,
};

/// A formatter which outputs with normal whitespace and guarantees valid BibTeX.
pub struct PrettyFormatter {}

impl Formatter for PrettyFormatter {}

/// A formatter which outputs with no excess whitespace and and does not check for valid BibTeX.
pub struct CompactFormatter {}

impl Formatter for CompactFormatter {
    #[inline]
    fn write_entry_separator<W>(&mut self, _writer: &mut W) -> io::Result<()>
    where
        W: ?Sized + io::Write,
    {
        Ok(())
    }

    #[inline]
    fn write_entry_key_end<W>(&mut self, _writer: &mut W) -> io::Result<()>
    where
        W: ?Sized + io::Write,
    {
        Ok(())
    }

    #[inline]
    fn write_field_start<W>(&mut self, writer: &mut W) -> io::Result<()>
    where
        W: ?Sized + io::Write,
    {
        writer.write_all(b",")
    }

    #[inline]
    fn write_field_separator<W>(&mut self, writer: &mut W) -> io::Result<()>
    where
        W: ?Sized + io::Write,
    {
        writer.write_all(b"=")
    }

    #[inline]
    fn write_token_separator<W>(&mut self, writer: &mut W) -> io::Result<()>
    where
        W: ?Sized + io::Write,
    {
        writer.write_all(b"#")
    }

    #[inline]
    fn write_field_end<W>(&mut self, _writer: &mut W) -> io::Result<()>
    where
        W: ?Sized + io::Write,
    {
        Ok(())
    }

    #[inline]
    fn write_bibliography_end<W>(&mut self, _writer: &mut W) -> io::Result<()>
    where
        W: ?Sized + io::Write,
    {
        Ok(())
    }
}

/// A wrapper struct to convert an arbitrary formatter into one which also performs validation.
pub struct ValidatingFormatter<F>(F);

impl<F> ValidatingFormatter<F> {
    /// Create a new ValidatingFormatter
    pub fn new(formatter: F) -> Self {
        Self(formatter)
    }
}

impl<F: Formatter> Formatter for ValidatingFormatter<F> {
    #[inline]
    fn write_entry_separator<W>(&mut self, writer: &mut W) -> io::Result<()>
    where
        W: ?Sized + io::Write,
    {
        self.0.write_entry_separator(writer)
    }

    #[inline]
    fn write_regular_entry_type<W>(&mut self, writer: &mut W, entry_type: &str) -> io::Result<()>
    where
        W: ?Sized + io::Write,
    {
        if !is_regular_entry_type(entry_type) {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("invalid entry type: '{entry_type}'"),
            ));
        }
        self.0.write_regular_entry_type(writer, entry_type)
    }

    #[inline]
    fn write_body_start<W>(&mut self, writer: &mut W) -> io::Result<()>
    where
        W: ?Sized + io::Write,
    {
        self.0.write_body_start(writer)
    }

    #[inline]
    fn write_entry_key<W>(&mut self, writer: &mut W, key: &str) -> io::Result<()>
    where
        W: ?Sized + io::Write,
    {
        if !is_entry_key(key) {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("invalid entry key: '{key}'"),
            ));
        }
        self.0.write_entry_key(writer, key)
    }

    #[inline]
    fn write_entry_key_end<W>(&mut self, writer: &mut W) -> io::Result<()>
    where
        W: ?Sized + io::Write,
    {
        self.0.write_entry_key_end(writer)
    }

    #[inline]
    fn write_field_start<W>(&mut self, writer: &mut W) -> io::Result<()>
    where
        W: ?Sized + io::Write,
    {
        self.0.write_field_start(writer)
    }

    #[inline]
    fn write_field_key<W>(&mut self, writer: &mut W, key: &str) -> io::Result<()>
    where
        W: ?Sized + io::Write,
    {
        if !is_field_key(key) {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("invalid field key: '{key}'"),
            ));
        }
        self.0.write_field_key(writer, key)
    }

    #[inline]
    fn write_field_separator<W>(&mut self, writer: &mut W) -> io::Result<()>
    where
        W: ?Sized + io::Write,
    {
        self.0.write_field_separator(writer)
    }

    #[inline]
    fn write_token_separator<W>(&mut self, writer: &mut W) -> io::Result<()>
    where
        W: ?Sized + io::Write,
    {
        self.0.write_token_separator(writer)
    }

    #[inline]
    fn write_bracketed_token<W>(&mut self, writer: &mut W, text: &str) -> io::Result<()>
    where
        W: ?Sized + io::Write,
    {
        if !is_balanced(text.as_bytes()) {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("unbalanced text token: '{text}'"),
            ));
        }
        self.0.write_bracketed_token(writer, text)
    }

    #[inline]
    fn write_variable_token<W>(&mut self, writer: &mut W, variable: &str) -> io::Result<()>
    where
        W: ?Sized + io::Write,
    {
        if !is_variable(variable) {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("invalid variable: '{variable}'"),
            ));
        }
        self.0.write_variable_token(writer, variable)
    }

    #[inline]
    fn write_field_end<W>(&mut self, writer: &mut W) -> io::Result<()>
    where
        W: ?Sized + io::Write,
    {
        self.0.write_field_end(writer)
    }

    #[inline]
    fn write_body_end<W>(&mut self, writer: &mut W) -> io::Result<()>
    where
        W: ?Sized + io::Write,
    {
        self.0.write_body_end(writer)
    }

    #[inline]
    fn write_bibliography_end<W>(&mut self, writer: &mut W) -> io::Result<()>
    where
        W: ?Sized + io::Write,
    {
        self.0.write_bibliography_end(writer)
    }
}

fn write_entry_type<W: ?Sized + io::Write>(writer: &mut W, entry_type: &str) -> io::Result<()> {
    writer.write_all(b"@")?;
    writer.write_all(entry_type.as_bytes())
}

/// A generic formatter used to write the components of a BibTeX bibliography.
pub trait Formatter {
    /// The separator between consecutive entries.
    #[inline]
    fn write_entry_separator<W>(&mut self, writer: &mut W) -> io::Result<()>
    where
        W: ?Sized + io::Write,
    {
        writer.write_all(b"\n\n")
    }

    /// Write the entry type, including the `@` symbol.
    #[inline]
    fn write_regular_entry_type<W>(&mut self, writer: &mut W, entry_type: &str) -> io::Result<()>
    where
        W: ?Sized + io::Write,
    {
        write_entry_type(writer, entry_type)
    }

    /// Write the macro entry type, including the `@` symbol.
    #[inline]
    fn write_macro_entry_type<W>(&mut self, writer: &mut W) -> io::Result<()>
    where
        W: ?Sized + io::Write,
    {
        write_entry_type(writer, "string")
    }

    /// Write the comment entry type, including the `@` symbol.
    #[inline]
    fn write_comment_entry_type<W>(&mut self, writer: &mut W) -> io::Result<()>
    where
        W: ?Sized + io::Write,
    {
        write_entry_type(writer, "comment")
    }

    /// Write the preamble entry type, including the `@` symbol.
    #[inline]
    fn write_preamble_entry_type<W>(&mut self, writer: &mut W) -> io::Result<()>
    where
        W: ?Sized + io::Write,
    {
        write_entry_type(writer, "preamble")
    }

    /// Write the body start character, typically `{`.
    #[inline]
    fn write_body_start<W>(&mut self, writer: &mut W) -> io::Result<()>
    where
        W: ?Sized + io::Write,
    {
        writer.write_all(b"{")
    }

    /// Write an entry key.
    #[inline]
    fn write_entry_key<W>(&mut self, writer: &mut W, key: &str) -> io::Result<()>
    where
        W: ?Sized + io::Write,
    {
        writer.write_all(key.as_bytes())
    }

    /// Write the terminator for an entry key, often `,\n`.
    #[inline]
    fn write_entry_key_end<W>(&mut self, writer: &mut W) -> io::Result<()>
    where
        W: ?Sized + io::Write,
    {
        writer.write_all(b",\n")
    }

    /// Write the start of a field, such as indentation `  `.
    #[inline]
    fn write_field_start<W>(&mut self, writer: &mut W) -> io::Result<()>
    where
        W: ?Sized + io::Write,
    {
        writer.write_all(b"  ")
    }

    /// Write a field key.
    #[inline]
    fn write_field_key<W>(&mut self, writer: &mut W, key: &str) -> io::Result<()>
    where
        W: ?Sized + io::Write,
    {
        writer.write_all(key.as_bytes())
    }

    /// Write a field separator, such as ` = `.
    #[inline]
    fn write_field_separator<W>(&mut self, writer: &mut W) -> io::Result<()>
    where
        W: ?Sized + io::Write,
    {
        writer.write_all(b" = ")
    }

    /// Write a token separator, such as ` # `.
    #[inline]
    fn write_token_separator<W>(&mut self, writer: &mut W) -> io::Result<()>
    where
        W: ?Sized + io::Write,
    {
        writer.write_all(b" # ")
    }

    /// Write a bracketed token `{text}`.
    #[inline]
    fn write_bracketed_token<W>(&mut self, writer: &mut W, token: &str) -> io::Result<()>
    where
        W: ?Sized + io::Write,
    {
        writer.write_all(b"{")?;
        writer.write_all(token.as_bytes())?;
        writer.write_all(b"}")
    }

    /// Write a variable token `text`.
    #[inline]
    fn write_variable_token<W>(&mut self, writer: &mut W, variable: &str) -> io::Result<()>
    where
        W: ?Sized + io::Write,
    {
        writer.write_all(variable.as_bytes())
    }

    /// Write the terminator for a field, often `,\n`.
    #[inline]
    fn write_field_end<W>(&mut self, writer: &mut W) -> io::Result<()>
    where
        W: ?Sized + io::Write,
    {
        writer.write_all(b",\n")
    }

    /// Write the terminator for the body, often `}`.
    #[inline]
    fn write_body_end<W>(&mut self, writer: &mut W) -> io::Result<()>
    where
        W: ?Sized + io::Write,
    {
        writer.write_all(b"}")
    }

    /// Write the terminator for the bibliography, such as a newline.
    #[inline]
    fn write_bibliography_end<W>(&mut self, writer: &mut W) -> io::Result<()>
    where
        W: ?Sized + io::Write,
    {
        writer.write_all(b"\n")
    }
}
