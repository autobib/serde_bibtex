use std::io;

use serde::ser;

use super::macros::{ser_wrapper, serialize_as_bytes, serialize_err, serialize_trait_impl};
use super::{Formatter, Serializer};
use crate::{
    error::{Error, Result},
    naming::{MACRO_TOKEN_VARIANT_NAME as MTVN, TEXT_TOKEN_VARIANT_NAME as TTVN},
};

ser_wrapper!(ValueSerializer);

impl<'a, W, F> ser::Serializer for ValueSerializer<'a, W, F>
where
    W: io::Write,
    F: Formatter,
{
    type Ok = ();

    serialize_err!(
        "value",
        i8,
        i16,
        i32,
        i64,
        u8,
        u16,
        u32,
        u64,
        f32,
        f64,
        option,
        bool,
        map,
        struct,
        struct_variant,
        tuple_variant,
        unit,
        unit_struct,
        unit_variant,
        newtype_variant
    );

    type SerializeSeq = TokenListSerializer<'a, W, F>;
    type SerializeTuple = TokenListSerializer<'a, W, F>;
    type SerializeTupleStruct = TokenListSerializer<'a, W, F>;

    fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq> {
        Ok(TokenListSerializer::new(&mut *self.ser))
    }

    fn serialize_tuple(self, _len: usize) -> Result<Self::SerializeTuple> {
        Ok(TokenListSerializer::new(&mut *self.ser))
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleStruct> {
        Ok(TokenListSerializer::new(&mut *self.ser))
    }

    fn serialize_char(self, v: char) -> Result<Self::Ok> {
        TextTokenSerializer::new(&mut *self.ser).serialize_char(v)
    }

    fn serialize_str(self, v: &str) -> Result<Self::Ok> {
        TextTokenSerializer::new(&mut *self.ser).serialize_str(v)
    }

    fn serialize_bytes(self, v: &[u8]) -> Result<Self::Ok> {
        TextTokenSerializer::new(&mut *self.ser).serialize_bytes(v)
    }
}

pub(crate) struct TokenListSerializer<'a, W, F> {
    ser: &'a mut Serializer<W, F>,
    first: bool,
}

impl<'a, W, F> TokenListSerializer<'a, W, F> {
    pub(crate) fn new(ser: &'a mut Serializer<W, F>) -> Self {
        Self { ser, first: true }
    }
}

macro_rules! token_list_serializer_impl {
    ($fn:ident, $trait:ident) => {
        serialize_trait_impl!(TokenListSerializer, $trait, {
            type Ok = ();

            fn $fn<T>(&mut self, value: &T) -> std::result::Result<Self::Ok, Self::Error>
            where
                T: ?Sized + ser::Serialize,
            {
                if self.first {
                    self.first = false;
                } else {
                    self.ser.buffer.write_token_separator()?;
                }
                value.serialize(TokenSerializer::new(&mut *self.ser))
            }
        });
    };
}

token_list_serializer_impl!(serialize_element, SerializeSeq);
token_list_serializer_impl!(serialize_element, SerializeTuple);
token_list_serializer_impl!(serialize_field, SerializeTupleStruct);

ser_wrapper!(TokenSerializer);

impl<'a, W, F> ser::Serializer for TokenSerializer<'a, W, F>
where
    W: io::Write,
    F: Formatter,
{
    type Ok = ();

    serialize_err!(
        "value token",
        i8,
        i16,
        i32,
        i64,
        u8,
        u16,
        u32,
        u64,
        f32,
        f64,
        option,
        char,
        str,
        bytes,
        bool,
        map,
        seq,
        tuple,
        tuple_struct,
        struct,
        struct_variant,
        tuple_variant,
        unit_variant,
        unit,
        unit_struct
    );

    fn serialize_newtype_variant<T>(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        value: &T,
    ) -> Result<Self::Ok>
    where
        T: ?Sized + ser::Serialize,
    {
        match variant {
            MTVN => value.serialize(VariableTokenSerializer::new(&mut *self.ser)),
            TTVN => value.serialize(TextTokenSerializer::new(&mut *self.ser)),
            var => Err(Error::ser(format!("invalid token variant '{var}'"))),
        }
    }
}

serialize_as_bytes!("text token", TextTokenSerializer, {
    fn serialize_str(self, value: &str) -> Result<Self::Ok> {
        self.ser.buffer.write_bracketed_token(value)?;
        Ok(())
    }
});

serialize_as_bytes!("field key", FieldKeySerializer, {
    fn serialize_str(self, value: &str) -> Result<Self::Ok> {
        self.ser.buffer.write_field_key(value)?;
        Ok(())
    }
});

serialize_as_bytes!("variable token", VariableTokenSerializer, {
    fn serialize_str(self, value: &str) -> Result<Self::Ok> {
        self.ser.buffer.write_variable_token(value)?;
        Ok(())
    }
});

serialize_as_bytes!("entry type", EntryTypeSerializer, {
    /// Serialize the entry type, and also write the body start
    fn serialize_str(self, value: &str) -> Result<Self::Ok> {
        self.ser.buffer.write_regular_entry_type(value)?;
        self.ser.buffer.write_body_start()?;
        Ok(())
    }
});

serialize_as_bytes!("entry key", EntryKeySerializer, {
    /// Serialize the entry type, and also the trailing comma
    fn serialize_str(self, value: &str) -> Result<Self::Ok> {
        self.ser.buffer.write_entry_key(value)?;
        self.ser.buffer.write_entry_key_end()?;
        Ok(())
    }
});
