use std::collections::HashMap;

use crate::value::{Identifier, Token, Value};

#[derive(Debug, Default, Clone)]
pub struct Abbreviations<'r> {
    abbrevs: HashMap<Identifier<'r>, Value<'r>>,
    buffer: Vec<Token<'r>>,
}

impl<'r> Abbreviations<'r> {
    pub fn insert(&mut self, identifier: Identifier<'r>, mut value: Value<'r>) {
        self.resolve(&mut value);
        self.abbrevs.insert(identifier, value);
    }

    pub fn get(&self, identifier: &Identifier<'r>) -> Option<&[Token<'r>]> {
        self.abbrevs.get(identifier).map(|v| v.0.as_slice())
    }

    pub fn resolve(&mut self, value: &mut Value<'r>) {
        self.buffer.clear();
        for token in value.0.drain(..) {
            if let Token::Abbrev(ref identifier) = token {
                match self.abbrevs.get(identifier) {
                    Some(sub) => self.buffer.extend(sub.0.iter().cloned()),
                    None => self.buffer.push(token),
                };
            } else {
                self.buffer.push(token);
            }
        }
        value.0.append(&mut self.buffer);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::value::{Identifier, Token, Value};

    #[test]
    fn test_abbreviations() {
        let mut abbrevs = Abbreviations::default();
        abbrevs.insert(
            Identifier::from_str_unchecked("a"),
            Value(vec![Token::text_from("1"), Token::abbrev_from("b")]),
        );
        abbrevs.insert(
            Identifier::from_str_unchecked("b"),
            Value(vec![Token::text_from("2")]),
        );
        assert_eq!(
            abbrevs.get(&Identifier::from_str_unchecked("a")),
            Some(&[Token::text_from("1"), Token::abbrev_from("b")][..])
        );

        abbrevs.insert(
            Identifier::from_str_unchecked("c"),
            Value(vec![Token::abbrev_from("a"), Token::abbrev_from("b")]),
        );
        assert_eq!(
            abbrevs.get(&Identifier::from_str_unchecked("c")),
            Some(
                &[
                    Token::text_from("1"),
                    Token::abbrev_from("b"),
                    Token::text_from("2")
                ][..]
            )
        );

        let mut value = Value(vec![
            Token::abbrev_from("c"),
            Token::text_from("1"),
            Token::text_from("2"),
            Token::abbrev_from("d"),
            Token::text_from("3"),
            Token::abbrev_from("b"),
        ]);
        abbrevs.resolve(&mut value);
        assert_eq!(
            value,
            Value(vec![
                Token::text_from("1"),
                Token::abbrev_from("b"),
                Token::text_from("2"),
                Token::text_from("1"),
                Token::text_from("2"),
                Token::abbrev_from("d"),
                Token::text_from("3"),
                Token::text_from("2"),
            ]),
        );
    }
}
