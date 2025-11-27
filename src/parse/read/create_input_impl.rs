macro_rules! read_impl {
    (
        $(#[$outer:meta])*
        $vis:vis struct $name:ident<'r>(&'r $target:ty);

        $var:ident;

        $convert:expr;
    ) => {
        $(#[$outer])*
        $vis struct $name<'r> {
            pub(crate) input: &'r $target,
            pub(crate) pos: usize,
        }

        impl<'r> $name<'r> {
            /// Create a new reader from the provided input buffer.
            pub fn new(input: &'r $target) -> Self {
                Self { input, pos: 0 }
            }

            /// Apply `parser` to `self.input` and `self.pos`, updating `self.pos` and returning `O`.
            #[inline]
            fn apply<O>(
                &mut self,
                mut parser: impl FnMut(&'r $target, usize) -> Result<(usize, O), Error>,
            ) -> Result<O, Error> {
                let (new, ret) = parser(self.input, self.pos)?;
                self.pos = new;
                Ok(ret)
            }
        }

        impl<'r> Read<'r> for $name<'r> {
            #[inline]
            fn peek(&self) -> Option<u8> {
                if self.pos < self.input.len() {
                    Some($convert(self.input)[self.pos])
                } else {
                    None
                }
            }

            #[inline]
            fn discard(&mut self) {
                self.pos += 1
            }

            #[inline]
            fn next_entry_or_eof(&mut self) -> bool {
                let (new, res) = next_entry_or_eof(self.input, self.pos);
                self.pos = new;
                res
            }

            #[inline]
            fn comment(&mut self) {
                self.pos = comment(self.input, self.pos)
            }

            #[inline]
            fn identifier(&mut self) -> Result<Identifier<&'r str>, Error> {
                self.apply(identifier)
            }

            #[inline]
            fn balanced(&mut self) -> Result<Text<&'r str, &'r [u8]>, Error> {
                Ok(Text::$var(self.apply(balanced)?))
            }

            #[inline]
            fn protected(&mut self, until: u8) -> Result<Text<&'r str, &'r [u8]>, Error> {
                Ok(Text::$var(self.apply(protected(until))?))
            }

            #[inline]
            fn number(&mut self) -> Result<&'r str, Error> {
                self.apply(number)
            }
        }
        impl<'r> BibtexParse<'r> for $name<'r> {}
    };
}

pub(crate) use read_impl;
