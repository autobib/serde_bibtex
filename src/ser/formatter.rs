use std::io;

pub struct DefaultFormatter {}

impl Formatter for DefaultFormatter {
    const VALIDATE: bool = false;
}

pub struct ValidatingFormatter {}

impl Formatter for ValidatingFormatter {
    const VALIDATE: bool = true;
}

pub struct CompactFormatter {}

impl Formatter for CompactFormatter {
    const VALIDATE: bool = false;

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
}

pub trait Formatter {
    const VALIDATE: bool;

    #[inline]
    fn write_entry_separator<W>(&mut self, writer: &mut W) -> io::Result<()>
    where
        W: ?Sized + io::Write,
    {
        writer.write_all(b"\n\n")
    }

    #[inline]
    fn write_entry_type<W>(&mut self, writer: &mut W, entry_type: &str) -> io::Result<()>
    where
        W: ?Sized + io::Write,
    {
        writer.write_all(b"@")?;
        writer.write_all(entry_type.as_bytes())
    }

    #[inline]
    fn write_body_start<W>(&mut self, writer: &mut W) -> io::Result<()>
    where
        W: ?Sized + io::Write,
    {
        writer.write_all(b"{")
    }

    #[inline]
    fn write_entry_key<W>(&mut self, writer: &mut W, key: &str) -> io::Result<()>
    where
        W: ?Sized + io::Write,
    {
        writer.write_all(key.as_bytes())
    }

    #[inline]
    fn write_entry_key_end<W>(&mut self, writer: &mut W) -> io::Result<()>
    where
        W: ?Sized + io::Write,
    {
        writer.write_all(b",\n")
    }

    #[inline]
    fn write_field_start<W>(&mut self, writer: &mut W) -> io::Result<()>
    where
        W: ?Sized + io::Write,
    {
        writer.write_all(b"  ")
    }

    #[inline]
    fn write_field_key<W>(&mut self, writer: &mut W, key: &str) -> io::Result<()>
    where
        W: ?Sized + io::Write,
    {
        writer.write_all(key.as_bytes())
    }

    #[inline]
    fn write_field_separator<W>(&mut self, writer: &mut W) -> io::Result<()>
    where
        W: ?Sized + io::Write,
    {
        writer.write_all(b" = ")
    }

    #[inline]
    fn write_token_separator<W>(&mut self, writer: &mut W) -> io::Result<()>
    where
        W: ?Sized + io::Write,
    {
        writer.write_all(b" # ")
    }

    #[inline]
    fn write_bracketed_token<W>(&mut self, writer: &mut W, token: &str) -> io::Result<()>
    where
        W: ?Sized + io::Write,
    {
        writer.write_all(b"{")?;
        writer.write_all(token.as_bytes())?;
        writer.write_all(b"}")
    }

    #[inline]
    fn write_variable_token<W>(&mut self, writer: &mut W, variable: &str) -> io::Result<()>
    where
        W: ?Sized + io::Write,
    {
        writer.write_all(variable.as_bytes())
    }

    #[inline]
    fn write_field_end<W>(&mut self, writer: &mut W) -> io::Result<()>
    where
        W: ?Sized + io::Write,
    {
        writer.write_all(b",\n")
    }

    #[inline]
    fn write_body_end<W>(&mut self, writer: &mut W) -> io::Result<()>
    where
        W: ?Sized + io::Write,
    {
        writer.write_all(b"}")
    }
}
