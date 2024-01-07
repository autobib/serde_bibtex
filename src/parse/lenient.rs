//! Some lenient parsers

fn key_chars(input: &str) -> IResult<&str, &str> {
    is_not("{}(), \t\n")(input)
}
