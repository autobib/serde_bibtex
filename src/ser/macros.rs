macro_rules! serialize_err_helper {
    ($err:ident, bool) => {
        fn serialize_bool(self, _v: bool) -> std::result::Result<Self::Ok, Self::Error> {
            Err(Self::Error::$err())
        }
    };

    ($err:tt, i8) => {
        fn serialize_i8(self, _v: i8) -> std::result::Result<Self::Ok, Self::Error> {
            Err(Self::Error::$err())
        }
    };

    ($err:tt, i16) => {
        fn serialize_i16(self, _v: i16) -> std::result::Result<Self::Ok, Self::Error> {
            Err(Self::Error::$err())
        }
    };

    ($err:tt, i32) => {
        fn serialize_i32(self, _v: i32) -> std::result::Result<Self::Ok, Self::Error> {
            Err(Self::Error::$err())
        }
    };

    ($err:tt, i64) => {
        fn serialize_i64(self, _v: i64) -> std::result::Result<Self::Ok, Self::Error> {
            Err(Self::Error::$err())
        }
    };

    ($err:tt, u8) => {
        fn serialize_u8(self, _v: u8) -> std::result::Result<Self::Ok, Self::Error> {
            Err(Self::Error::$err())
        }
    };

    ($err:tt, u16) => {
        fn serialize_u16(self, _v: u16) -> std::result::Result<Self::Ok, Self::Error> {
            Err(Self::Error::$err())
        }
    };

    ($err:tt, u32) => {
        fn serialize_u32(self, _v: u32) -> std::result::Result<Self::Ok, Self::Error> {
            Err(Self::Error::$err())
        }
    };

    ($err:tt, u64) => {
        fn serialize_u64(self, _v: u64) -> std::result::Result<Self::Ok, Self::Error> {
            Err(Self::Error::$err())
        }
    };

    ($err:tt, f32) => {
        fn serialize_f32(self, _v: f32) -> std::result::Result<Self::Ok, Self::Error> {
            Err(Self::Error::$err())
        }
    };

    ($err:tt, f64) => {
        fn serialize_f64(self, _v: f64) -> std::result::Result<Self::Ok, Self::Error> {
            Err(Self::Error::$err())
        }
    };

    ($err:tt, char) => {
        fn serialize_char(self, _v: char) -> std::result::Result<Self::Ok, Self::Error> {
            Err(Self::Error::$err())
        }
    };

    ($err:tt, str) => {
        fn serialize_str(self, _v: &str) -> std::result::Result<Self::Ok, Self::Error> {
            Err(Self::Error::$err())
        }
    };

    ($err:tt, bytes) => {
        fn serialize_bytes(self, _v: &[u8]) -> std::result::Result<Self::Ok, Self::Error> {
            Err(Self::Error::$err())
        }
    };

    ($err:tt, option) => {
        fn serialize_none(self) -> std::result::Result<Self::Ok, Self::Error> {
            Err(Self::Error::$err())
        }

        fn serialize_some<T>(self, _v: &T) -> std::result::Result<Self::Ok, Self::Error>
        where
            T: ?Sized + ser::Serialize,
        {
            Err(Self::Error::$err())
        }
    };

    ($err:tt, unit) => {
        fn serialize_unit(self) -> std::result::Result<Self::Ok, Self::Error> {
            Err(Self::Error::$err())
        }
    };

    ($err:tt, unit_struct) => {
        fn serialize_unit_struct(
            self,
            _name: &'static str,
        ) -> std::result::Result<Self::Ok, Self::Error> {
            Err(Self::Error::$err())
        }
    };

    ($err:tt, unit_variant) => {
        fn serialize_unit_variant(
            self,
            _name: &'static str,
            _variant_index: u32,
            _variant: &'static str,
        ) -> std::result::Result<Self::Ok, Self::Error> {
            Err(Self::Error::$err())
        }
    };

    ($err:tt, newtype_variant) => {
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
            Err(Self::Error::$err())
        }
    };

    ($err:tt, seq) => {
        type SerializeSeq = serde::ser::Impossible<Self::Ok, Self::Error>;

        fn serialize_seq(
            self,
            _len: Option<usize>,
        ) -> std::result::Result<Self::SerializeSeq, Self::Error> {
            Err(Self::Error::$err())
        }
    };

    ($err:tt, tuple) => {
        type SerializeTuple = serde::ser::Impossible<Self::Ok, Self::Error>;

        fn serialize_tuple(
            self,
            _len: usize,
        ) -> std::result::Result<Self::SerializeTuple, Self::Error> {
            Err(Self::Error::$err())
        }
    };

    ($err:tt, tuple_struct) => {
        type SerializeTupleStruct = serde::ser::Impossible<Self::Ok, Self::Error>;

        fn serialize_tuple_struct(
            self,
            _name: &'static str,
            _len: usize,
        ) -> std::result::Result<Self::SerializeTupleStruct, Self::Error> {
            Err(Self::Error::$err())
        }
    };

    ($err:tt, tuple_variant) => {
        type SerializeTupleVariant = serde::ser::Impossible<Self::Ok, Self::Error>;

        fn serialize_tuple_variant(
            self,
            _name: &'static str,
            _variant_index: u32,
            _variant: &'static str,
            _len: usize,
        ) -> std::result::Result<Self::SerializeTupleVariant, Self::Error> {
            Err(Self::Error::$err())
        }
    };

    ($err:tt, map) => {
        type SerializeMap = serde::ser::Impossible<Self::Ok, Self::Error>;

        fn serialize_map(
            self,
            _len: Option<usize>,
        ) -> std::result::Result<Self::SerializeMap, Self::Error> {
            Err(Self::Error::$err())
        }
    };

    ($err:tt, struct) => {
        type SerializeStruct = serde::ser::Impossible<Self::Ok, Self::Error>;

        fn serialize_struct(
            self,
            _name: &'static str,
            _len: usize,
        ) -> std::result::Result<Self::SerializeStruct, Self::Error> {
            Err(Self::Error::$err())
        }
    };

    ($err:tt, struct_variant) => {
        type SerializeStructVariant = serde::ser::Impossible<Self::Ok, Self::Error>;

        fn serialize_struct_variant(
            self,
            _name: &'static str,
            _variant_index: u32,
            _variant: &'static str,
            _len: usize,
        ) -> std::result::Result<Self::SerializeStructVariant, Self::Error> {
            Err(Self::Error::$err())
        }
    };
}

macro_rules! serialize_err {
    ($err:ident) => {};
    ($err:ident, $e:tt) => {
        type Error = Error;

        fn serialize_newtype_struct<T>(
            self,
            _name: &'static str,
            _value: &T,
        ) -> std::result::Result<Self::Ok, Self::Error>
        where
            T: ?Sized + ser::Serialize,
        {
            Err(Self::Error::$err())
        }

        crate::ser::macros::serialize_err_helper!($err, $e);
    };
    ($err:ident, $e:tt, $($es:tt),+) => {
        crate::ser::macros::serialize_err_helper!($err, $e);
        serialize_err!($err, $($es),*);
    };
}

/// A macro to defer serialization to an implementation for bytes
macro_rules! serialize_as_bytes {
    ($name:ident, {$($str_impl:tt)*}) => {
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
