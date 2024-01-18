macro_rules! input_read_impl {
    ($target:ty, $name:ident, $var:ident, $convert:expr) => {
        use crate::de::Deserializer;

        #[derive(Debug)]
        pub struct $name<'r> {
            pub(crate) input: &'r $target,
        }

        impl<'r> $name<'r> {
            pub fn new(input: &'r $target) -> Self {
                Self { input }
            }

            /// Apply `parser` to `self.input`, updating `input` and returning `T`.
            fn apply<O>(
                &mut self,
                mut parser: impl FnMut(&'r $target) -> Result<(&'r $target, O), ReadError>,
            ) -> Result<O, ReadError> {
                let (input, ret) = parser(self.input)?;
                self.input = input;
                Ok(ret)
            }

            pub fn deserialize(self) -> Deserializer<'r, Self> {
                Deserializer::new(self)
            }
        }

        impl<'r> Read<'r> for $name<'r> {
            #[inline]
            fn peek(&self) -> Option<u8> {
                let bytes = $convert(self.input);
                bytes.first().copied()
            }

            #[inline]
            fn discard(&mut self) {
                self.input = &self.input[1..];
            }

            #[inline]
            fn next_entry_or_eof(&mut self) -> bool {
                let (tail, res) = next_entry_or_eof(self.input);
                self.input = tail;
                res
            }

            #[inline]
            fn comment(&mut self) {
                self.input = comment(self.input)
            }

            #[inline]
            fn identifier_unicode(&mut self) -> Result<UnicodeIdentifier<'r>, ReadError> {
                self.apply(identifier_unicode)
            }

            #[inline]
            fn identifier_ascii(&mut self) -> Result<AsciiIdentifier<'r>, ReadError> {
                self.apply(identifier_ascii)
            }

            #[inline]
            fn balanced(&mut self) -> Result<Text<'r>, ReadError> {
                Ok(Text::$var(Cow::Borrowed(self.apply(balanced)?)))
            }

            #[inline]
            fn protected(&mut self, until: u8) -> Result<Text<'r>, ReadError> {
                Ok(Text::$var(Cow::Borrowed(self.apply(protected(until))?)))
            }

            #[inline]
            fn number(&mut self) -> Result<Text<'r>, ReadError> {
                self.apply(number)
            }
        }
        impl<'r> BibtexParse<'r> for $name<'r> {}
    };
}

pub(crate) use input_read_impl;
