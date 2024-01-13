use std::collections::HashMap;

use crate::value::{Identifier, Token, Value};

#[derive(Debug, Default, Clone)]
pub struct MacroDictionary<'r> {
    macro_hash: HashMap<Identifier<'r>, Value<'r>>,
    scratch: Vec<Token<'r>>,
}

impl<'r> MacroDictionary<'r> {
    pub fn insert(&mut self, identifier: Identifier<'r>, mut value: Value<'r>) {
        self.resolve(&mut value);
        self.macro_hash.insert(identifier, value);
    }

    pub(crate) fn insert_raw_tokens(&mut self, identifier: Identifier<'r>, tokens: Vec<Token<'r>>) {
        self.macro_hash.insert(identifier, Value(tokens));
    }

    pub fn replace(&mut self, identifier: Identifier<'r>, new: &mut Vec<Token<'r>>) {
        // get a mutable reference to the existing value, creating it if it does not exist
        let existing = self
            .macro_hash
            .entry(identifier)
            .and_modify(|v| v.0.clear())
            .or_insert(Value::default());

        // insert the new value
        std::mem::swap(&mut existing.0, new);
    }

    pub fn get(&self, identifier: &Identifier<'r>) -> Option<&[Token<'r>]> {
        self.macro_hash.get(identifier).map(|v| v.0.as_slice())
    }

    pub fn resolve_tokens(&mut self, tokens: &mut Vec<Token<'r>>) {
        self.scratch.clear();
        for token in tokens.drain(..) {
            if let Token::Macro(ref identifier) = token {
                match self.macro_hash.get(identifier) {
                    Some(sub) => {
                        self.scratch.extend(sub.0.iter().cloned());
                    }
                    None => self.scratch.push(token),
                };
            } else {
                self.scratch.push(token);
            }
        }
        tokens.append(&mut self.scratch);
    }

    pub fn resolve(&mut self, value: &mut Value<'r>) {
        self.resolve_tokens(&mut value.0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::value::{Identifier, Token, Value};

    #[test]
    fn test_abbreviations() {
        let mut abbrevs = MacroDictionary::default();
        abbrevs.insert(
            Identifier::from_str_unchecked("a"),
            Value(vec![Token::text_from("1"), Token::macro_from("b")]),
        );
        abbrevs.insert(
            Identifier::from_str_unchecked("b"),
            Value(vec![Token::text_from("2")]),
        );
        assert_eq!(
            abbrevs.get(&Identifier::from_str_unchecked("a")),
            Some(&[Token::text_from("1"), Token::macro_from("b")][..])
        );

        abbrevs.insert(
            Identifier::from_str_unchecked("c"),
            Value(vec![Token::macro_from("a"), Token::macro_from("b")]),
        );
        assert_eq!(
            abbrevs.get(&Identifier::from_str_unchecked("c")),
            Some(
                &[
                    Token::text_from("1"),
                    Token::macro_from("b"),
                    Token::text_from("2")
                ][..]
            )
        );

        let mut value = Value(vec![
            Token::macro_from("c"),
            Token::text_from("1"),
            Token::text_from("2"),
            Token::macro_from("d"),
            Token::text_from("3"),
            Token::macro_from("b"),
        ]);
        abbrevs.resolve(&mut value);
        assert_eq!(
            value,
            Value(vec![
                Token::text_from("1"),
                Token::macro_from("b"),
                Token::text_from("2"),
                Token::text_from("1"),
                Token::text_from("2"),
                Token::macro_from("d"),
                Token::text_from("3"),
                Token::text_from("2"),
            ]),
        );
    }
}
