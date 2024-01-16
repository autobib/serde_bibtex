macro_rules! input_read_impl {
    ($target:ty, $name:ident, $var:ident, $convert:expr) => {
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
        }

        impl<'r> InputRead<'r> for $name<'r> {
            fn peek(&self) -> Option<u8> {
                let bytes = $convert(self.input);
                bytes.first().copied()
            }

            fn discard(&mut self) {
                self.input = &self.input[1..];
            }

            fn next_entry_or_eof(&mut self) -> bool {
                let (tail, res) = next_entry_or_eof(self.input);
                self.input = tail;
                res
            }

            fn comment(&mut self) {
                self.input = comment(self.input)
            }

            fn identifier_unicode(&mut self) -> Result<UnicodeIdentifier<'r>, ReadError> {
                self.apply(identifier_unicode)
            }

            fn identifier_ascii(&mut self) -> Result<AsciiIdentifier<'r>, ReadError> {
                self.apply(identifier_ascii)
            }

            fn balanced(&mut self) -> Result<Text<'r>, ReadError> {
                Ok(Text::$var(Cow::Borrowed(self.apply(balanced)?)))
            }

            fn protected(&mut self, until: u8) -> Result<Text<'r>, ReadError> {
                Ok(Text::$var(Cow::Borrowed(self.apply(protected(until))?)))
            }

            fn number(&mut self) -> Result<Text<'r>, ReadError> {
                self.apply(number)
            }
        }
        impl<'r> BibtexParser<'r> for $name<'r> {}
    };
}

pub(crate) use input_read_impl;
