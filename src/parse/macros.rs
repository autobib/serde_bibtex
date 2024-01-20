use std::collections::HashMap;

use super::{Token, Variable};

#[derive(Debug, Clone)]
pub struct MacroDictionary<S: AsRef<str>, B: AsRef<[u8]>> {
    map: HashMap<Variable<S>, Vec<Token<S, B>>>,
    scratch: Vec<Token<S, B>>,
}

impl<S: AsRef<str>, B: AsRef<[u8]>> Default for MacroDictionary<S, B> {
    fn default() -> Self {
        Self::new(HashMap::default())
    }
}

impl<S: AsRef<str>, B: AsRef<[u8]>> MacroDictionary<S, B> {
    pub fn new(map: HashMap<Variable<S>, Vec<Token<S, B>>>) -> Self {
        Self {
            map,
            scratch: Vec::default(),
        }
    }

    pub fn into_inner(self) -> HashMap<Variable<S>, Vec<Token<S, B>>> {
        self.map
    }
}

impl<S, B> MacroDictionary<S, B>
where
    S: AsRef<str>,
    B: AsRef<[u8]>,
{
    /// Convert to an owned version.
    ///
    /// Note that this clones the underlying values even if they are already owned.
    pub fn own(&self) -> MacroDictionary<String, Vec<u8>> {
        let new_map = HashMap::from_iter(self.map.iter().map(|(Variable(key), val)| {
            (
                Variable::new_unchecked(key.as_ref().to_string()),
                val.iter().map(|t| Token::<S, B>::own(t)).collect(),
            )
        }));

        MacroDictionary::new(new_map)
    }
}

impl<S, B> MacroDictionary<S, B>
where
    S: AsRef<str> + Eq + std::hash::Hash + From<&'static str>,
    B: AsRef<[u8]>,
{
    /// Set "month macros", such as `@string{apr = {4}}`.
    pub fn set_month_macros(&mut self) {
        macro_rules! ins {
            ($var:expr, $text:expr) => {
                self.insert_raw_tokens(
                    Variable::new_unchecked($var.into()),
                    vec![Token::str_unchecked($text.into())],
                );
            };
        }

        ins!("jan", "1");
        ins!("feb", "2");
        ins!("mar", "3");
        ins!("apr", "4");
        ins!("may", "5");
        ins!("jun", "6");
        ins!("jul", "7");
        ins!("aug", "8");
        ins!("sep", "9");
        ins!("oct", "10");
        ins!("nov", "11");
        ins!("dec", "12");
    }
}

impl<S, B> MacroDictionary<S, B>
where
    S: AsRef<str> + Eq + std::hash::Hash,
    B: AsRef<[u8]>,
{
    pub(crate) fn insert_raw_tokens(
        &mut self,
        identifier: Variable<S>,
        tokens: Vec<Token<S, B>>,
    ) -> Option<Vec<Token<S, B>>> {
        self.map.insert(identifier, tokens)
    }

    /// Get the tokens associated with an identifier.
    pub fn get(&self, identifier: &Variable<S>) -> Option<&[Token<S, B>]> {
        self.map.get(identifier).map(|v| v.as_slice())
    }
}

impl<S, B> MacroDictionary<S, B>
where
    S: AsRef<str> + Eq + std::hash::Hash + Clone,
    B: AsRef<[u8]> + Clone,
{
    /// Insert a new identifier and associated tokens.
    pub fn insert(&mut self, identifier: Variable<S>, mut tokens: Vec<Token<S, B>>) {
        self.resolve(&mut tokens);
        self.insert_raw_tokens(identifier, tokens);
    }

    /// Resolve tokens in-place using the macros stored in the dictionary.
    pub fn resolve(&mut self, tokens: &mut Vec<Token<S, B>>) {
        self.scratch.clear();
        for token in tokens.drain(..) {
            if let Token::Variable(ref identifier) = token {
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
        let mut abbrevs = MacroDictionary::<&str, &[u8]>::default();
        abbrevs.insert(
            Variable::new_unchecked("a"),
            vec![Token::str_unchecked("1"), Token::variable_unchecked("b")],
        );
        abbrevs.insert(
            Variable::new_unchecked("b"),
            vec![Token::str_unchecked("2")],
        );
        assert_eq!(
            abbrevs.get(&Variable::new_unchecked("a")),
            Some(&[Token::str_unchecked("1"), Token::variable_unchecked("b")][..])
        );

        abbrevs.insert(
            Variable::new_unchecked("c"),
            vec![
                Token::variable_unchecked("a"),
                Token::variable_unchecked("b"),
            ],
        );
        assert_eq!(
            abbrevs.get(&Variable::new_unchecked("c")),
            Some(
                &[
                    Token::str_unchecked("1"),
                    Token::variable_unchecked("b"),
                    Token::str_unchecked("2")
                ][..]
            )
        );

        let mut value = vec![
            Token::variable_unchecked("c"),
            Token::str_unchecked("1"),
            Token::str_unchecked("2"),
            Token::variable_unchecked("d"),
            Token::str_unchecked("3"),
            Token::variable_unchecked("b"),
        ];
        abbrevs.resolve(&mut value);
        assert_eq!(
            value,
            vec![
                Token::str_unchecked("1"),
                Token::variable_unchecked("b"),
                Token::str_unchecked("2"),
                Token::str_unchecked("1"),
                Token::str_unchecked("2"),
                Token::variable_unchecked("d"),
                Token::str_unchecked("3"),
                Token::str_unchecked("2"),
            ],
        );
    }

    #[test]
    fn test_set_month() {
        let mut abbrevs = MacroDictionary::<&str, &[u8]>::default();
        abbrevs.set_month_macros();

        assert_eq!(
            abbrevs.get(&Variable::new_unchecked("feb")),
            Some(&[Token::str_unchecked("2")][..])
        );

        assert_eq!(
            abbrevs.get(&Variable::new_unchecked("dec")),
            Some(&[Token::str_unchecked("12")][..])
        );
    }

    #[test]
    fn test_case_insensitive() {
        let mut abbrevs = MacroDictionary::<&str, &[u8]>::default();
        abbrevs.insert(
            Variable::new_unchecked("ss"),
            vec![Token::str_unchecked("0")],
        );
        abbrevs.insert(
            Variable::new_unchecked("ß"),
            vec![Token::str_unchecked("1")],
        );
        abbrevs.insert(
            Variable::new_unchecked("SS"),
            vec![Token::str_unchecked("2")],
        );
        assert_eq!(
            abbrevs.get(&Variable::new_unchecked("ss")),
            Some(&[Token::str_unchecked("2")][..])
        );
        assert_eq!(
            abbrevs.get(&Variable::new_unchecked("ß")),
            Some(&[Token::str_unchecked("2")][..])
        );
        assert_eq!(
            abbrevs.get(&Variable::new_unchecked("SS")),
            Some(&[Token::str_unchecked("2")][..])
        );
    }
}
