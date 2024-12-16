use std::io;

use crate::token::{is_balanced, is_entry_key, is_field_key, is_regular_entry_type, is_variable};

pub(crate) struct FormatBuffer<F> {
    formatter: F,
    entry_key: Vec<u8>,
    entry_type: Vec<u8>,
    fields: Vec<u8>,
}

/// A wrapper struct for a [`Formatter`] which writes to an internal buffer. This struct is needed
/// in order to support out-of-order serialization of struct fields.
impl<F> FormatBuffer<F> {
    pub fn new(formatter: F) -> Self {
        Self {
            formatter,
            entry_key: Vec::with_capacity(16),
            entry_type: Vec::with_capacity(16),
            fields: Vec::with_capacity(128),
        }
    }

    /// Write the contents of the buffers in order
    pub fn write<W>(&mut self, writer: &mut W) -> io::Result<()>
    where
        W: ?Sized + io::Write,
    {
        writer.write_all(&self.entry_type)?;
        self.entry_type.clear();
        writer.write_all(&self.entry_key)?;
        self.entry_key.clear();
        writer.write_all(&self.fields)?;
        self.fields.clear();
        Ok(())
    }
}

impl<F: Formatter> FormatBuffer<F> {
    /// The separator between consecutive entries.
    #[inline]
    pub fn write_entry_separator<W>(&mut self, writer: &mut W) -> io::Result<()>
    where
        W: ?Sized + io::Write,
    {
        self.formatter.write_entry_separator(writer)
    }

    /// Write the entry type, including the `@` symbol.
    #[inline]
    pub fn write_regular_entry_type(&mut self, entry_type: &str) -> io::Result<()> {
        self.formatter
            .write_regular_entry_type(&mut self.entry_type, entry_type)
    }

    /// Write the macro entry type, including the `@` symbol.
    #[inline]
    pub fn write_macro_entry_type(&mut self) -> io::Result<()> {
        self.formatter.write_macro_entry_type(&mut self.entry_type)
    }

    /// Write the comment entry type, including the `@` symbol.
    #[inline]
    pub fn write_comment_entry_type(&mut self) -> io::Result<()> {
        self.formatter
            .write_comment_entry_type(&mut self.entry_type)
    }

    /// Write the preamble entry type, including the `@` symbol.
    #[inline]
    pub fn write_preamble_entry_type(&mut self) -> io::Result<()> {
        self.formatter
            .write_preamble_entry_type(&mut self.entry_type)
    }

    /// Write the body start character, typically `{`.
    #[inline]
    pub fn write_body_start(&mut self) -> io::Result<()> {
        self.formatter.write_body_start(&mut self.entry_type)
    }

    /// Write an entry key.
    #[inline]
    pub fn write_entry_key(&mut self, key: &str) -> io::Result<()> {
        self.formatter.write_entry_key(&mut self.entry_key, key)
    }

    /// Write the terminator for an entry key, often `,\n`.
    #[inline]
    pub fn write_entry_key_end(&mut self) -> io::Result<()> {
        self.formatter.write_entry_key_end(&mut self.entry_key)
    }

    /// Write the start of a field, such as indentation `  `.
    #[inline]
    pub fn write_field_start(&mut self) -> io::Result<()> {
        self.formatter.write_field_start(&mut self.fields)
    }

    /// Write a field key.
    #[inline]
    pub fn write_field_key(&mut self, key: &str) -> io::Result<()> {
        self.formatter.write_field_key(&mut self.fields, key)
    }

    /// Write a field separator, such as ` = `.
    #[inline]
    pub fn write_field_separator(&mut self) -> io::Result<()> {
        self.formatter.write_field_separator(&mut self.fields)
    }

    /// Write a token separator, such as ` # `.
    #[inline]
    pub fn write_token_separator(&mut self) -> io::Result<()> {
        self.formatter.write_token_separator(&mut self.fields)
    }

    /// Write a bracketed token `{text}`.
    #[inline]
    pub fn write_bracketed_token(&mut self, token: &str) -> io::Result<()> {
        self.formatter
            .write_bracketed_token(&mut self.fields, token)
    }

    /// Write a variable token `text`.
    #[inline]
    pub fn write_variable_token(&mut self, variable: &str) -> io::Result<()> {
        self.formatter
            .write_variable_token(&mut self.fields, variable)
    }

    /// Write the terminator for a field, often `,\n`.
    #[inline]
    pub fn write_field_end(&mut self) -> io::Result<()> {
        self.formatter.write_field_end(&mut self.fields)
    }

    /// Write the terminator for the body, often `}`.
    #[inline]
    pub fn write_body_end(&mut self) -> io::Result<()> {
        self.formatter.write_body_end(&mut self.fields)
    }

    /// Write the terminator for the bibliography, such as a newline.
    #[inline]
    pub fn write_bibliography_end<W>(&mut self, writer: &mut W) -> io::Result<()>
    where
        W: ?Sized + io::Write,
    {
        self.formatter.write_bibliography_end(writer)
    }
}

/// A formatter which outputs with normal whitespace and does not check for valid BibTeX.
pub struct PrettyFormatter {}

impl Formatter for PrettyFormatter {}

impl PrettyFormatter {
    /// Return a formatter with the same output, except that also validates the generated BibTeX.
    pub fn validate(self) -> ValidatingFormatter<PrettyFormatter> {
        ValidatingFormatter::new(self)
    }
}

/// A formatter which outputs with no excess whitespace and does not check for valid BibTeX.
pub struct CompactFormatter {}

impl CompactFormatter {
    /// Return a formatter with the same output, except that also validates the generated BibTeX.
    pub fn validate(self) -> ValidatingFormatter<CompactFormatter> {
        ValidatingFormatter::new(self)
    }
}

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

/// A wrapper to convert an arbitrary formatter into one which also performs validation.
pub struct ValidatingFormatter<F>(F);

impl<F> ValidatingFormatter<F> {
    /// Create a `ValidatingFormatter` by wrapping another formatter.
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
