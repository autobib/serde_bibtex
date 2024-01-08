//! Some lenient parsers

fn key_chars(input: &str) -> IResult<&str, &str> {
    is_not("{}(), \t\n")(input)
}

fn ignored(input: &str) -> IResult<&str, ()> {
    // TODO: incorporate bibtex_comment
    //     nom_value(
    //         (), // Output is thrown away.
    //         pair(char('%'), is_not("\n\r")),
    //     )(i)

    // TODO: incorporate bibtex skips, e.g. \% is discarded, or '%
    nom_value((), multispace0)(input)
}
