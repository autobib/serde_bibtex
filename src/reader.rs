use nom::IResult;

use crate::parse::BibtexReader;

use crate::error::Error;
use crate::parse::core as p;
use crate::parse::core::ChunkType;
use crate::value::{Identifier, Token};

// TODO: parsing variants
// resolving parser (for abbreviations)
// enforcing valid (to check parsing grammar)
// comment style (bibtex vs biber)
#[derive(Debug)]
pub(crate) struct ResolvingReader<'r> {
    pub(crate) input: &'r str,
}

impl<'r> BibtexReader<'r> for ResolvingReader<'r> {
    fn take_chunk_type(&mut self) -> Result<Option<ChunkType<'r>>, Error> {
        self.step(p::chunk_type)
    }

    fn take_initial(&mut self) -> Result<char, Error> {
        self.step(p::initial)
    }

    fn opt_comma(&mut self) -> Result<(), Error> {
        self.step(p::opt_comma)
    }

    fn take_terminal(&mut self, closing: char) -> Result<(), Error> {
        self.step(p::terminal(closing))
    }

    fn take_citation_key(&mut self) -> Result<&'r str, Error> {
        self.step(p::citation_key)
    }

    fn take_field_key(&mut self) -> Result<Option<Identifier<'r>>, Error> {
        self.step(p::field_key)
    }

    fn take_comma_and_field_key(&mut self) -> Result<Option<Identifier<'r>>, Error> {
        self.step(p::comma_and_field_key)
    }

    fn take_bracketed_text(&mut self) -> Result<&'r str, Error> {
        self.step(p::bracketed_text)
    }

    fn ignore_field_sep(&mut self) -> Result<(), Error> {
        self.step(p::field_sep)
    }

    fn take_token(&mut self, is_first_token: &mut bool) -> Result<Option<Token<'r>>, Error> {
        if *is_first_token {
            let (updated, token) = p::token(self.input)?;
            *is_first_token = false;
            self.input = updated;
            Ok(Some(token))
        } else {
            let (updated, opt_token) = p::subsequent_token(self.input)?;
            self.input = updated;
            match opt_token {
                Some(_) => Ok(opt_token),
                None => {
                    *is_first_token = true;
                    Ok(None)
                }
            }
        }
    }
}

// use trace::trace;
// trace::init_depth_var!();
// #[trace]
impl<'r> ResolvingReader<'r> {
    /// Construct a new Reader
    pub fn new(input: &'r str) -> Self {
        Self { input }
    }

    /// Apply `parser` to `self.input`, updating `input` and returning `T`.
    fn step<O>(
        &mut self,
        mut parser: impl FnMut(&'r str) -> IResult<&'r str, O>,
    ) -> Result<O, Error> {
        let (input, ret) = parser(self.input)?;
        self.input = input;
        Ok(ret)
    }
}
