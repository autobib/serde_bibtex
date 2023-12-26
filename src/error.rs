pub type ParseError<'de> = nom::Err<nom::error::Error<&'de str>>;

#[derive(Debug, PartialEq)]
pub enum ConversionError {
    UnresolvedAbbreviations,
    NotText,
}
