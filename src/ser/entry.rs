use std::io;

use serde::ser::{self, Error as _, Serialize as _};

use super::{
    macros::{ser_wrapper, serialize_err, serialize_trait_impl},
    value::{
        EntryKeySerializer, EntryTypeSerializer, FieldKeySerializer, TextTokenSerializer,
        ValueSerializer, VariableTokenSerializer,
    },
    Formatter, Serializer,
};
use crate::error::{Error, Result};
use crate::naming::{
    COMMENT_ENTRY_VARIANT_NAME as CVN, ENTRY_KEY_NAME, ENTRY_TYPE_NAME, FIELDS_NAME,
    MACRO_ENTRY_VARIANT_NAME as MVN, PREAMBLE_ENTRY_VARIANT_NAME as PVN,
    REGULAR_ENTRY_VARIANT_NAME as RVN,
};

ser_wrapper!(EntrySerializer);

impl<'a, W, F> serde::Serializer for EntrySerializer<'a, W, F>
where
    W: std::io::Write,
    F: Formatter,
{
    type Ok = bool;

    type SerializeTuple = RegularEntryTupleSerializer<'a, W, F>;
    type SerializeTupleVariant = RegularOrMacroEntrySerializer<'a, W, F>;
    type SerializeTupleStruct = RegularEntryTupleSerializer<'a, W, F>;
    type SerializeStruct = RegularEntryStructSerializer<'a, W, F>;
    type SerializeStructVariant = RegularEntryStructSerializer<'a, W, F>;

    serialize_err!(
        only_enum,
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
        char,
        seq,
        str,
        bytes,
        bool,
        map,
        option,
        unit,
        unit_struct
    );

    /// A unit variant is simply skipped. However, the variant name must be valid.
    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
    ) -> Result<Self::Ok> {
        match variant {
            RVN | MVN | CVN | PVN => Ok(true),
            var => Err(Error::custom(format!("Unexpected enum variant {var}"))),
        }
    }

    fn serialize_tuple(self, len: usize) -> std::result::Result<Self::SerializeTuple, Self::Error> {
        if len == 3 {
            Ok(RegularEntryTupleSerializer::new(&mut *self.ser))
        } else {
            Err(Self::Error::custom(
                "regular entry from tuple not of length 3",
            ))
        }
    }

    /// A tuple struct is treated as a regular entry.
    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleStruct> {
        if len == 3 {
            Ok(RegularEntryTupleSerializer::new(&mut *self.ser))
        } else {
            Err(Self::Error::custom(
                "regular entry from tuple not of length 3",
            ))
        }
    }

    /// A tuple variant is either a regular entry or a macro entry
    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleVariant> {
        match (variant, len) {
            (RVN, 3) => Ok(RegularOrMacroEntrySerializer::new(
                &mut *self.ser,
                TupleEntryVariant::Regular,
            )),
            (MVN, 2) => Ok(RegularOrMacroEntrySerializer::new(
                &mut *self.ser,
                TupleEntryVariant::Macro,
            )),
            (RVN, _) => Err(Self::Error::custom(
                "regular entry from tuple not of length 3",
            )),
            (MVN, _) => Err(Self::Error::custom(
                "macro entry from tuple not of length 2",
            )),
            (CVN, _) => Err(Self::Error::custom(
                "tuple serialization not supported for comment",
            )),
            (PVN, _) => Err(Self::Error::custom(
                "tuple serialization not supported for preamble",
            )),
            _ => Err(Self::Error::custom("unrecognized entry variant")),
        }
    }

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
            RVN => value.serialize(RegularEntrySerializer::new(&mut *self.ser)),
            MVN => value.serialize(MacroRuleSerializer::new(&mut *self.ser)),
            CVN => {
                self.ser
                    .buffer
                    .write_comment_entry_type()
                    .map_err(Error::io)?;
                value.serialize(TextTokenSerializer::new(&mut *self.ser))?;
                Ok(false)
            }
            PVN => {
                self.ser.buffer.write_preamble_entry_type()?;
                self.ser.buffer.write_body_start()?;
                value.serialize(ValueSerializer::new(&mut *self.ser))?;
                self.ser.buffer.write_body_end()?;
                Ok(false)
            }
            _ => Err(Error::custom(format!("Invalid variant name `{variant}`"))),
        }
    }

    fn serialize_struct(self, _name: &'static str, _len: usize) -> Result<Self::SerializeStruct> {
        Ok(RegularEntryStructSerializer::new(&mut *self.ser))
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant> {
        match variant {
            RVN => Ok(RegularEntryStructSerializer::new(&mut *self.ser)),
            _ => Err(Error::custom(
                "struct serialization only supported for regular entry".to_string(),
            )),
        }
    }
}

ser_wrapper!(RegularEntrySerializer);

impl<'a, W, F> ser::Serializer for RegularEntrySerializer<'a, W, F>
where
    W: std::io::Write,
    F: Formatter,
{
    type Ok = bool;

    serialize_err!(
        only_seq,
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
        char,
        str,
        seq,
        bytes,
        bool,
        tuple_variant,
        map,
        option,
        struct_variant,
        unit,
        unit_struct,
        unit_variant,
        newtype_variant
    );

    type SerializeTuple = RegularEntryTupleSerializer<'a, W, F>;
    type SerializeTupleStruct = RegularEntryTupleSerializer<'a, W, F>;
    type SerializeStruct = RegularEntryStructSerializer<'a, W, F>;

    #[inline]
    fn serialize_tuple(self, len: usize) -> std::result::Result<Self::SerializeTuple, Self::Error> {
        if len == 3 {
            Ok(RegularEntryTupleSerializer::new(&mut *self.ser))
        } else {
            Err(Self::Error::custom(
                "regular entry from tuple not of length 3",
            ))
        }
    }

    #[inline]
    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        len: usize,
    ) -> std::result::Result<Self::SerializeTupleStruct, Self::Error> {
        if len == 3 {
            Ok(RegularEntryTupleSerializer::new(&mut *self.ser))
        } else {
            Err(Self::Error::custom(
                "regular entry from tuple not of length 3",
            ))
        }
    }

    #[inline]
    fn serialize_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> std::result::Result<Self::SerializeStruct, Self::Error> {
        Ok(RegularEntryStructSerializer::new(&mut *self.ser))
    }
}

ser_wrapper!(RegularEntryTupleSerializer, index);

macro_rules! regular_entry_tuple_serializer_impl {
    ($fn:ident, $trait:ident) => {
        serialize_trait_impl!(RegularEntryTupleSerializer, $trait, {
            type Ok = bool;

            fn $fn<T>(&mut self, value: &T) -> std::result::Result<(), Self::Error>
            where
                T: ?Sized + ser::Serialize,
            {
                self.index += 1;
                match &self.index {
                    1 => value.serialize(EntryTypeSerializer::new(&mut *self.ser)),
                    2 => value.serialize(EntryKeySerializer::new(&mut *self.ser)),
                    3 => value.serialize(EntryFieldsSerializer::new(&mut *self.ser)),
                    _ => unreachable!(),
                }
            }
        });
    };
}

regular_entry_tuple_serializer_impl!(serialize_field, SerializeTupleStruct);
regular_entry_tuple_serializer_impl!(serialize_element, SerializeTuple);

pub(crate) struct RegularEntryStructSerializer<'a, W, F> {
    ser: &'a mut Serializer<W, F>,
}
impl<'a, W, F> RegularEntryStructSerializer<'a, W, F> {
    #[inline]
    pub(crate) fn new(ser: &'a mut Serializer<W, F>) -> Self {
        Self { ser }
    }
}

macro_rules! regular_entry_serializer_impl {
    ($trait:ident) => {
        serialize_trait_impl!(RegularEntryStructSerializer, $trait, {
            type Ok = bool;

            fn serialize_field<T>(
                &mut self,
                key: &'static str,
                value: &T,
            ) -> std::result::Result<(), Self::Error>
            where
                T: ?Sized + ser::Serialize,
            {
                match key {
                    ENTRY_TYPE_NAME => value.serialize(EntryTypeSerializer::new(&mut *self.ser)),
                    ENTRY_KEY_NAME => value.serialize(EntryKeySerializer::new(&mut *self.ser)),
                    FIELDS_NAME => value.serialize(EntryFieldsSerializer::new(&mut *self.ser)),
                    var => Err(Error::custom(format!("Unexpected struct field {var}"))),
                }
            }
        });
    };
}

regular_entry_serializer_impl!(SerializeStruct);
regular_entry_serializer_impl!(SerializeStructVariant);

pub(crate) enum TupleEntryVariant {
    Regular,
    Macro,
}

pub(crate) struct RegularOrMacroEntrySerializer<'a, W, F> {
    ser: &'a mut Serializer<W, F>,
    variant: TupleEntryVariant,
    index: usize,
}

impl<'a, W, F> RegularOrMacroEntrySerializer<'a, W, F> {
    #[inline]
    pub(crate) fn new(ser: &'a mut Serializer<W, F>, variant: TupleEntryVariant) -> Self {
        Self {
            ser,
            variant,
            index: 0,
        }
    }
}

impl<'a, W, F> ser::SerializeTupleVariant for RegularOrMacroEntrySerializer<'a, W, F>
where
    W: io::Write,
    F: Formatter,
{
    type Ok = bool;

    type Error = Error;

    fn serialize_field<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + ser::Serialize,
    {
        self.index += 1;
        match (&self.variant, &self.index) {
            (TupleEntryVariant::Regular, 1) => {
                value.serialize(EntryTypeSerializer::new(&mut *self.ser))
            }
            (TupleEntryVariant::Regular, 2) => {
                value.serialize(EntryKeySerializer::new(&mut *self.ser))
            }
            (TupleEntryVariant::Regular, 3) => {
                value.serialize(EntryFieldsSerializer::new(&mut *self.ser))
            }
            (TupleEntryVariant::Regular, _) => unreachable!(),
            (TupleEntryVariant::Macro, 1) => {
                self.ser
                    .buffer
                    .write_macro_entry_type()
                    .map_err(Error::io)?;
                self.ser.buffer.write_body_start().map_err(Error::io)?;
                value.serialize(VariableTokenSerializer::new(&mut *self.ser))
            }
            (TupleEntryVariant::Macro, 2) => {
                self.ser.buffer.write_field_separator().map_err(Error::io)?;
                value.serialize(ValueSerializer::new(&mut *self.ser))?;
                self.ser.buffer.write_body_end().map_err(Error::io)
            }
            (TupleEntryVariant::Macro, _) => unreachable!(),
        }
    }

    fn end(self) -> Result<Self::Ok> {
        Ok(Self::Ok::default())
    }
}

ser_wrapper!(MacroRuleSerializer);

impl<'a, W, F> ser::Serializer for MacroRuleSerializer<'a, W, F>
where
    W: std::io::Write,
    F: Formatter,
{
    type Ok = bool;

    serialize_err!(
        only_seq,
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
        char,
        str,
        bytes,
        bool,
        seq,
        tuple_variant,
        map,
        struct,
        struct_variant,
        unit,
        unit_struct,
        unit_variant,
        newtype_variant
    );

    type SerializeTuple = MacroTupleSerializer<'a, W, F>;
    type SerializeTupleStruct = MacroTupleSerializer<'a, W, F>;

    #[inline]
    fn serialize_none(self) -> std::result::Result<Self::Ok, Self::Error> {
        Ok(true)
    }

    #[inline]
    fn serialize_some<T>(self, value: &T) -> std::result::Result<Self::Ok, Self::Error>
    where
        T: ?Sized + ser::Serialize,
    {
        value.serialize(self)
    }

    fn serialize_tuple(self, len: usize) -> std::result::Result<Self::SerializeTuple, Self::Error> {
        if len == 2 {
            Ok(Self::SerializeTuple::new(&mut *self.ser))
        } else {
            Err(Self::Error::custom(
                "macro entry from tuple not of length 2",
            ))
        }
    }

    #[inline]
    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        len: usize,
    ) -> std::result::Result<Self::SerializeTupleStruct, Self::Error> {
        if len == 2 {
            Ok(Self::SerializeTupleStruct::new(&mut *self.ser))
        } else {
            Err(Self::Error::custom(
                "macro entry from tuple not of length 2",
            ))
        }
    }
}

ser_wrapper!(MacroTupleSerializer, index);
macro_rules! macro_tuple_serializer_impl {
    ($fn:ident, $trait:ident) => {
        serialize_trait_impl!(MacroTupleSerializer, $trait, {
            type Ok = bool;

            fn $fn<T>(&mut self, value: &T) -> Result<()>
            where
                T: ?Sized + ser::Serialize,
            {
                self.index += 1;
                match self.index {
                    1 => {
                        self.ser
                            .buffer
                            .write_macro_entry_type()
                            .map_err(Error::io)?;
                        self.ser.buffer.write_body_start().map_err(Error::io)?;
                        value.serialize(VariableTokenSerializer::new(&mut *self.ser))
                    }
                    2 => {
                        self.ser.buffer.write_field_separator().map_err(Error::io)?;
                        value.serialize(ValueSerializer::new(&mut *self.ser))?;
                        self.ser.buffer.write_body_end().map_err(Error::io)
                    }
                    _ => unreachable!(),
                }
            }
        });
    };
}

macro_tuple_serializer_impl!(serialize_element, SerializeTuple);
macro_tuple_serializer_impl!(serialize_field, SerializeTupleStruct);

ser_wrapper!(EntryFieldsSerializer);

impl<'a, W, F> ser::Serializer for EntryFieldsSerializer<'a, W, F>
where
    W: std::io::Write,
    F: Formatter,
{
    type Ok = ();

    serialize_err!(
        only_seq,
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
        char,
        str,
        option,
        bytes,
        bool,
        tuple_variant,
        struct_variant,
        unit,
        unit_struct,
        unit_variant,
        newtype_variant
    );

    type SerializeSeq = Self;
    type SerializeTuple = Self;
    type SerializeTupleStruct = Self;
    type SerializeStruct = Self;
    type SerializeMap = Self;

    #[inline]
    fn serialize_tuple(
        self,
        _len: usize,
    ) -> std::result::Result<Self::SerializeTuple, Self::Error> {
        Ok(Self::SerializeTuple::new(&mut *self.ser))
    }

    #[inline]
    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> std::result::Result<Self::SerializeTupleStruct, Self::Error> {
        Ok(Self::SerializeTupleStruct::new(&mut *self.ser))
    }

    #[inline]
    fn serialize_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> std::result::Result<Self::SerializeStruct, Self::Error> {
        Ok(Self::SerializeStruct::new(&mut *self.ser))
    }

    #[inline]
    fn serialize_map(
        self,
        _len: Option<usize>,
    ) -> std::result::Result<Self::SerializeMap, Self::Error> {
        Ok(Self::SerializeMap::new(&mut *self.ser))
    }

    #[inline]
    fn serialize_seq(
        self,
        _len: Option<usize>,
    ) -> std::result::Result<Self::SerializeSeq, Self::Error> {
        Ok(Self::SerializeSeq::new(&mut *self.ser))
    }
}

impl<'a, W, F> ser::SerializeStruct for EntryFieldsSerializer<'a, W, F>
where
    W: io::Write,
    F: Formatter,
{
    type Ok = ();
    type Error = Error;
    fn serialize_field<T>(
        &mut self,
        key: &'static str,
        value: &T,
    ) -> std::result::Result<Self::Ok, Self::Error>
    where
        T: ?Sized + ser::Serialize,
    {
        self.ser.buffer.write_field_start()?;
        key.serialize(FieldKeySerializer::new(&mut *self.ser))?;
        self.ser.buffer.write_field_separator()?;
        value.serialize(ValueSerializer::new(&mut *self.ser))?;
        self.ser.buffer.write_field_end()?;

        Self::Ok::default();
        Ok(())
    }
    #[inline]
    fn end(self) -> std::result::Result<Self::Ok, Self::Error> {
        self.ser.buffer.write_body_end()?;
        Self::Ok::default();
        Ok(())
    }
}

impl<'a, W, F> ser::SerializeMap for EntryFieldsSerializer<'a, W, F>
where
    W: io::Write,
    F: Formatter,
{
    type Ok = ();
    type Error = Error;

    fn serialize_key<T>(&mut self, key: &T) -> std::result::Result<Self::Ok, Self::Error>
    where
        T: ?Sized + ser::Serialize,
    {
        self.ser.buffer.write_field_start()?;
        key.serialize(FieldKeySerializer::new(&mut *self.ser))
    }

    fn serialize_value<T>(&mut self, value: &T) -> std::result::Result<Self::Ok, Self::Error>
    where
        T: ?Sized + ser::Serialize,
    {
        self.ser.buffer.write_field_separator()?;
        value.serialize(ValueSerializer::new(&mut *self.ser))?;
        self.ser.buffer.write_field_end()?;
        Self::Ok::default();
        Ok(())
    }

    #[inline]
    fn end(self) -> std::result::Result<Self::Ok, Self::Error> {
        self.ser.buffer.write_body_end()?;
        Self::Ok::default();
        Ok(())
    }
}

macro_rules! entry_fields_serializer_impl {
    ($fn:ident, $trait:ident) => {
        impl<'a, W, F> ser::$trait for EntryFieldsSerializer<'a, W, F>
        where
            W: io::Write,
            F: Formatter,
        {
            type Ok = ();

            type Error = Error;

            fn $fn<T>(&mut self, value: &T) -> std::result::Result<Self::Ok, Self::Error>
            where
                T: ?Sized + ser::Serialize,
            {
                value.serialize(KeyValueSerializer::new(&mut *self.ser))
            }

            #[inline]
            fn end(self) -> std::result::Result<Self::Ok, Self::Error> {
                self.ser.buffer.write_body_end()?;
                Ok(Self::Ok::default())
            }
        }
    };
}

entry_fields_serializer_impl!(serialize_element, SerializeSeq);
entry_fields_serializer_impl!(serialize_element, SerializeTuple);
entry_fields_serializer_impl!(serialize_field, SerializeTupleStruct);

ser_wrapper!(KeyValueSerializer);

impl<'a, W, F> ser::Serializer for KeyValueSerializer<'a, W, F>
where
    W: std::io::Write,
    F: Formatter,
{
    type Ok = ();

    serialize_err!(
        only_seq,
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
        char,
        str,
        bytes,
        option,
        bool,
        seq,
        tuple_variant,
        map,
        struct,
        struct_variant,
        unit,
        unit_struct,
        unit_variant,
        newtype_variant
    );

    type SerializeTuple = KeyValueTupleSerializer<'a, W, F>;
    type SerializeTupleStruct = KeyValueTupleSerializer<'a, W, F>;

    fn serialize_tuple(self, len: usize) -> std::result::Result<Self::SerializeTuple, Self::Error> {
        if len == 2 {
            Ok(Self::SerializeTuple::new(&mut *self.ser))
        } else {
            Err(Self::Error::custom("key value from tuple not of length 2"))
        }
    }

    #[inline]
    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        len: usize,
    ) -> std::result::Result<Self::SerializeTupleStruct, Self::Error> {
        if len == 2 {
            Ok(Self::SerializeTupleStruct::new(&mut *self.ser))
        } else {
            Err(Self::Error::custom("key value from tuple not of length 2"))
        }
    }
}

ser_wrapper!(KeyValueTupleSerializer, index);

macro_rules! key_value_tuple_serializer_impl {
    ($fn:ident, $trait:ident) => {
        serialize_trait_impl!(KeyValueTupleSerializer, $trait, {
            type Ok = ();

            fn $fn<T>(&mut self, value: &T) -> Result<()>
            where
                T: ?Sized + ser::Serialize,
            {
                self.index += 1;
                match self.index {
                    1 => {
                        self.ser.buffer.write_field_start()?;
                        value.serialize(FieldKeySerializer::new(&mut *self.ser))
                    }
                    2 => {
                        self.ser.buffer.write_field_separator()?;
                        value.serialize(ValueSerializer::new(&mut *self.ser))?;
                        self.ser.buffer.write_field_end()?;
                        Ok(Self::Ok::default())
                    }
                    _ => unreachable!(),
                }
            }
        });
    };
}

key_value_tuple_serializer_impl!(serialize_element, SerializeTuple);
key_value_tuple_serializer_impl!(serialize_field, SerializeTupleStruct);
