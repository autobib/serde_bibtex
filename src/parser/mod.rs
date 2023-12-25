pub mod balanced;
pub mod lenient;
pub mod strict;

use pest_derive::Parser;

#[derive(Parser)]
#[grammar = "parser/bibtex.pest"]
pub struct BibtexParser;

#[cfg(test)]
mod tests {
    use crate::parser::*;
    use pest::Parser;
    use std::fs;

    #[test]
    fn syntax_test() {
        let bibfile = "assets/parse_test.bib";
        let contents = fs::read_to_string(bibfile).unwrap();
        let parsed = BibtexParser::parse(Rule::bibtex, &contents);
        assert!(parsed.is_ok(), "{:?}", parsed);
    }
}
