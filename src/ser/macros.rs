macro_rules! serialize_err_helper {
    ($err:expr, bool) => {
        #[inline]
        fn serialize_bool(self, _v: bool) -> std::result::Result<Self::Ok, Self::Error> {
            Err(Self::Error::ser(concat!($err, " as bool").to_string()))
        }
    };

    ($err:tt, i8) => {
        #[inline]
        fn serialize_i8(self, _v: i8) -> std::result::Result<Self::Ok, Self::Error> {
            Err(Self::Error::ser(concat!($err, " as i8").to_string()))
        }
    };

    ($err:tt, i16) => {
        #[inline]
        fn serialize_i16(self, _v: i16) -> std::result::Result<Self::Ok, Self::Error> {
            Err(Self::Error::ser(concat!($err, " as i16").to_string()))
        }
    };

    ($err:tt, i32) => {
        #[inline]
        fn serialize_i32(self, _v: i32) -> std::result::Result<Self::Ok, Self::Error> {
            Err(Self::Error::ser(concat!($err, " as i32").to_string()))
        }
    };

    ($err:tt, i64) => {
        #[inline]
        fn serialize_i64(self, _v: i64) -> std::result::Result<Self::Ok, Self::Error> {
            Err(Self::Error::ser(concat!($err, " as i64").to_string()))
        }
    };

    ($err:tt, u8) => {
        #[inline]
        fn serialize_u8(self, _v: u8) -> std::result::Result<Self::Ok, Self::Error> {
            Err(Self::Error::ser(concat!($err, " as u8").to_string()))
        }
    };

    ($err:tt, u16) => {
        #[inline]
        fn serialize_u16(self, _v: u16) -> std::result::Result<Self::Ok, Self::Error> {
            Err(Self::Error::ser(concat!($err, " as u16").to_string()))
        }
    };

    ($err:tt, u32) => {
        #[inline]
        fn serialize_u32(self, _v: u32) -> std::result::Result<Self::Ok, Self::Error> {
            Err(Self::Error::ser(concat!($err, " as 32").to_string()))
        }
    };

    ($err:tt, u64) => {
        #[inline]
        fn serialize_u64(self, _v: u64) -> std::result::Result<Self::Ok, Self::Error> {
            Err(Self::Error::ser(concat!($err, " as u64").to_string()))
        }
    };

    ($err:tt, f32) => {
        #[inline]
        fn serialize_f32(self, _v: f32) -> std::result::Result<Self::Ok, Self::Error> {
            Err(Self::Error::ser(concat!($err, " as f32").to_string()))
        }
    };

    ($err:tt, f64) => {
        #[inline]
        fn serialize_f64(self, _v: f64) -> std::result::Result<Self::Ok, Self::Error> {
            Err(Self::Error::ser(concat!($err, " as f64").to_string()))
        }
    };

    ($err:tt, char) => {
        #[inline]
        fn serialize_char(self, _v: char) -> std::result::Result<Self::Ok, Self::Error> {
            Err(Self::Error::ser(concat!($err, " as char").to_string()))
        }
    };

    ($err:tt, str) => {
        #[inline]
        fn serialize_str(self, _v: &str) -> std::result::Result<Self::Ok, Self::Error> {
            Err(Self::Error::ser(concat!($err, " as str").to_string()))
        }
    };

    ($err:tt, bytes) => {
        #[inline]
        fn serialize_bytes(self, _v: &[u8]) -> std::result::Result<Self::Ok, Self::Error> {
            Err(Self::Error::ser(concat!($err, " as bytes").to_string()))
        }
    };

    ($err:tt, option) => {
        #[inline]
        fn serialize_none(self) -> std::result::Result<Self::Ok, Self::Error> {
            Err(Self::Error::ser(concat!($err, " as option").to_string()))
        }

        #[inline]
        fn serialize_some<T>(self, _v: &T) -> std::result::Result<Self::Ok, Self::Error>
        where
            T: ?Sized + ser::Serialize,
        {
            Err(Self::Error::ser(concat!($err, " as option").to_string()))
        }
    };

    ($err:tt, unit) => {
        #[inline]
        fn serialize_unit(self) -> std::result::Result<Self::Ok, Self::Error> {
            Err(Self::Error::ser(concat!($err, " as unit").to_string()))
        }
    };

    ($err:tt, unit_struct) => {
        #[inline]
        fn serialize_unit_struct(
            self,
            _name: &'static str,
        ) -> std::result::Result<Self::Ok, Self::Error> {
            Err(Self::Error::ser(
                concat!($err, " as unit struct").to_string(),
            ))
        }
    };

    ($err:tt, unit_variant) => {
        #[inline]
        fn serialize_unit_variant(
            self,
            _name: &'static str,
            _variant_index: u32,
            _variant: &'static str,
        ) -> std::result::Result<Self::Ok, Self::Error> {
            Err(Self::Error::ser(
                concat!($err, " as unit variant").to_string(),
            ))
        }
    };

    ($err:tt, newtype_variant) => {
        #[inline]
        fn serialize_newtype_variant<T>(
            self,
            _name: &'static str,
            _variant_index: u32,
            _variant: &'static str,
            _value: &T,
        ) -> std::result::Result<Self::Ok, Self::Error>
        where
            T: ?Sized + ser::Serialize,
        {
            Err(Self::Error::ser(
                concat!($err, " as newtype variant").to_string(),
            ))
        }
    };

    ($err:tt, seq) => {
        type SerializeSeq = serde::ser::Impossible<Self::Ok, Self::Error>;

        #[inline]
        fn serialize_seq(
            self,
            _len: Option<usize>,
        ) -> std::result::Result<Self::SerializeSeq, Self::Error> {
            Err(Self::Error::ser(concat!($err, " as sequence").to_string()))
        }
    };

    ($err:tt, tuple) => {
        type SerializeTuple = serde::ser::Impossible<Self::Ok, Self::Error>;

        #[inline]
        fn serialize_tuple(
            self,
            _len: usize,
        ) -> std::result::Result<Self::SerializeTuple, Self::Error> {
            Err(Self::Error::ser(concat!($err, " as tuple").to_string()))
        }
    };

    ($err:tt, tuple_struct) => {
        type SerializeTupleStruct = serde::ser::Impossible<Self::Ok, Self::Error>;

        #[inline]
        fn serialize_tuple_struct(
            self,
            _name: &'static str,
            _len: usize,
        ) -> std::result::Result<Self::SerializeTupleStruct, Self::Error> {
            Err(Self::Error::ser(
                concat!($err, " as tuple struct").to_string(),
            ))
        }
    };

    ($err:tt, tuple_variant) => {
        type SerializeTupleVariant = serde::ser::Impossible<Self::Ok, Self::Error>;

        #[inline]
        fn serialize_tuple_variant(
            self,
            _name: &'static str,
            _variant_index: u32,
            _variant: &'static str,
            _len: usize,
        ) -> std::result::Result<Self::SerializeTupleVariant, Self::Error> {
            Err(Self::Error::ser(
                concat!($err, " as tuple variant").to_string(),
            ))
        }
    };

    ($err:tt, map) => {
        type SerializeMap = serde::ser::Impossible<Self::Ok, Self::Error>;

        #[inline]
        fn serialize_map(
            self,
            _len: Option<usize>,
        ) -> std::result::Result<Self::SerializeMap, Self::Error> {
            Err(Self::Error::ser(concat!($err, " as map").to_string()))
        }
    };

    ($err:tt, struct) => {
        type SerializeStruct = serde::ser::Impossible<Self::Ok, Self::Error>;

        #[inline]
        fn serialize_struct(
            self,
            _name: &'static str,
            _len: usize,
        ) -> std::result::Result<Self::SerializeStruct, Self::Error> {
            Err(Self::Error::ser(concat!($err, " as struct").to_string()))
        }
    };

    ($err:tt, struct_variant) => {
        type SerializeStructVariant = serde::ser::Impossible<Self::Ok, Self::Error>;

        #[inline]
        fn serialize_struct_variant(
            self,
            _name: &'static str,
            _variant_index: u32,
            _variant: &'static str,
            _len: usize,
        ) -> std::result::Result<Self::SerializeStructVariant, Self::Error> {
            Err(Self::Error::ser(
                concat!($err, " as struct variant").to_string(),
            ))
        }
    };
}

macro_rules! serialize_err {
    ($err:expr) => {};
    ($err:expr, $e:tt) => {
        type Error = Error;

        #[inline]
        fn serialize_newtype_struct<T>(
            self,
            _name: &'static str,
            value: &T,
        ) -> std::result::Result<Self::Ok, Self::Error>
        where
            T: ?Sized + ser::Serialize,
        {
            value.serialize(self)
        }

        crate::ser::macros::serialize_err_helper!($err, $e);
    };
    ($err:expr, $e:tt, $($es:tt),+) => {
        crate::ser::macros::serialize_err_helper!($err, $e);
        serialize_err!($err, $($es),*);
    };
}

/// A macro to defer serialization to an implementation for bytes
macro_rules! serialize_as_bytes {
    ($err:expr, $name:ident, {$($str_impl:tt)*}) => {
        pub(crate) struct $name<'a, W, F> {
            ser: &'a mut Serializer<W, F>,
        }

        impl<'a, W, F> $name<'a, W, F> {
            pub(crate) fn new(ser: &'a mut Serializer<W, F>) -> Self {
                Self { ser }
            }
        }

        impl<'a, W, F> ser::Serializer for $name<'a, W, F>
        where
            W: std::io::Write,
            F: Formatter,
        {
            type Ok = ();

            serialize_err!(
                $err,
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
                bool,
                seq,
                bytes,
                option,
                tuple,
                tuple_struct,
                tuple_variant,
                map,
                struct,
                struct_variant,
                unit,
                unit_struct,
                newtype_variant
            );

            #[inline]
            $($str_impl)*

            #[inline]
            fn serialize_char(self, value: char) -> Result<Self::Ok> {
                // A char encoded as UTF-8 takes 4 bytes at most.
                let mut buf = [0; 4];
                self.serialize_bytes(value.encode_utf8(&mut buf).as_bytes())
            }

            /// A unit variant is serialized using the name of the variant.
            #[inline]
            fn serialize_unit_variant(
                self,
                _name: &'static str,
                _variant_index: u32,
                variant: &'static str,
            ) -> std::result::Result<Self::Ok, Self::Error> {
                self.serialize_bytes(variant.as_bytes())
            }
        }
    };
}

macro_rules! serialize_trait_impl {
    ($name:ident, $trait:ident, {$($byte_impl:tt)*}) => {
        impl<'a, W, F> ser::$trait for $name<'a, W, F>
        where
            W: io::Write,
            F: Formatter,
        {
            type Error = Error;

            $($byte_impl)*

            #[inline]
            fn end(self) -> std::result::Result<Self::Ok, Self::Error> {
                Ok(Self::Ok::default())
            }
        }
    };
}

macro_rules! ser_wrapper {
    ($name:ident) => {
        pub(crate) struct $name<'a, W, F> {
            ser: &'a mut Serializer<W, F>,
        }

        impl<'a, W, F> $name<'a, W, F> {
            #[inline]
            pub(crate) fn new(ser: &'a mut Serializer<W, F>) -> Self {
                Self { ser }
            }
        }
    };

    ($name:ident, index) => {
        pub(crate) struct $name<'a, W, F> {
            ser: &'a mut Serializer<W, F>,
            index: usize,
        }

        impl<'a, W, F> $name<'a, W, F> {
            #[inline]
            pub(crate) fn new(ser: &'a mut Serializer<W, F>) -> Self {
                Self { ser, index: 0 }
            }
        }
    };
}

pub(crate) use {
    ser_wrapper, serialize_as_bytes, serialize_err, serialize_err_helper, serialize_trait_impl,
};
