use std::collections::HashMap;

use crate::bib::token::{Token, Variable};

#[derive(Debug, Default, Clone)]
pub struct MacroDictionary<'r> {
    map: HashMap<Variable<'r>, Vec<Token<'r>>>,
    scratch: Vec<Token<'r>>,
}

impl<'r> MacroDictionary<'r> {
    pub fn insert(&mut self, identifier: Variable<'r>, mut tokens: Vec<Token<'r>>) {
        self.resolve(&mut tokens);
        self.insert_raw_tokens(identifier, tokens);
    }

    pub fn into_inner(self) -> HashMap<Variable<'r>, Vec<Token<'r>>> {
        self.map
    }

    pub(crate) fn insert_raw_tokens(&mut self, identifier: Variable<'r>, tokens: Vec<Token<'r>>) {
        self.map.insert(identifier, tokens);
    }

    pub fn replace(&mut self, identifier: Variable<'r>, new: &mut Vec<Token<'r>>) {
        // get a mutable reference to the existing value, creating it if it does not exist
        let existing = self
            .map
            .entry(identifier)
            .and_modify(|v| v.clear())
            .or_default();

        // insert the new value
        std::mem::swap(existing, new);
    }

    pub fn get(&self, identifier: &Variable<'r>) -> Option<&[Token<'r>]> {
        self.map.get(identifier).map(|v| v.as_slice())
    }

    pub fn resolve(&mut self, tokens: &mut Vec<Token<'r>>) {
        self.scratch.clear();
        for token in tokens.drain(..) {
            if let Token::Macro(ref identifier) = token {
                match self.map.get(identifier) {
                    Some(sub) => {
                        self.scratch.extend(sub.iter().cloned());
                    }
                    None => self.scratch.push(token),
                };
            } else {
                self.scratch.push(token);
            }
        }
        tokens.append(&mut self.scratch);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_insert() {
        let mut abbrevs = MacroDictionary::default();
        abbrevs.insert(
            Variable::from_str_unchecked("a"),
            vec![Token::text_from("1"), Token::macro_from("b")],
        );
        abbrevs.insert(
            Variable::from_str_unchecked("b"),
            vec![Token::text_from("2")],
        );
        assert_eq!(
            abbrevs.get(&Variable::from_str_unchecked("a")),
            Some(&[Token::text_from("1"), Token::macro_from("b")][..])
        );

        abbrevs.insert(
            Variable::from_str_unchecked("c"),
            vec![Token::macro_from("a"), Token::macro_from("b")],
        );
        assert_eq!(
            abbrevs.get(&Variable::from_str_unchecked("c")),
            Some(
                &[
                    Token::text_from("1"),
                    Token::macro_from("b"),
                    Token::text_from("2")
                ][..]
            )
        );

        let mut value = vec![
            Token::macro_from("c"),
            Token::text_from("1"),
            Token::text_from("2"),
            Token::macro_from("d"),
            Token::text_from("3"),
            Token::macro_from("b"),
        ];
        abbrevs.resolve(&mut value);
        assert_eq!(
            value,
            vec![
                Token::text_from("1"),
                Token::macro_from("b"),
                Token::text_from("2"),
                Token::text_from("1"),
                Token::text_from("2"),
                Token::macro_from("d"),
                Token::text_from("3"),
                Token::text_from("2"),
            ],
        );
    }

    #[test]
    fn test_case_insensitive() {
        let mut abbrevs = MacroDictionary::default();
        abbrevs.insert(
            Variable::from_str_unchecked("ss"),
            vec![Token::text_from("0")],
        );
        abbrevs.insert(
            Variable::from_str_unchecked("ß"),
            vec![Token::text_from("1")],
        );
        abbrevs.insert(
            Variable::from_str_unchecked("SS"),
            vec![Token::text_from("2")],
        );
        assert_eq!(
            abbrevs.get(&Variable::from_str_unchecked("ss")),
            Some(&[Token::text_from("2")][..])
        );
        assert_eq!(
            abbrevs.get(&Variable::from_str_unchecked("ß")),
            Some(&[Token::text_from("2")][..])
        );
        assert_eq!(
            abbrevs.get(&Variable::from_str_unchecked("SS")),
            Some(&[Token::text_from("2")][..])
        );
    }
}
