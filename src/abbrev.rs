use std::borrow::Cow;
use std::collections::HashMap;

use crate::bib::{Identifier, Token};

#[derive(Debug, Default)]
pub struct Abbreviations<'r> {
    abbrevs: HashMap<Identifier<'r>, Vec<Token<'r>>>,
    buffer: Vec<Token<'r>>,
}

// pub fn

/// A helper function to merge a Token into a cow, owning if required.
fn try_merge_closure<'r>(cow: Cow<'r, str>, token: &Token<'r>) -> Option<Cow<'r, str>> {
    match token {
        Token::Text(new) => {
            if new.len() > 0 {
                let mut s = cow.into_owned();
                s.push_str(new);
                Some(Cow::Owned(s))
            } else {
                Some(cow)
            }
        }
        _ => None,
    }
}

fn try_merge<'r>(tokens: &Vec<Token<'r>>) -> Option<Cow<'r, str>> {
    if tokens.len() == 0 {
        Some(Cow::Borrowed(""))
    } else {
        let acc = match &tokens[0] {
            Token::Text(cow) => cow.clone(),
            _ => return None,
        };
        tokens[1..].iter().try_fold(acc, try_merge_closure)
    }
}

impl<'r> Abbreviations<'r> {
    pub fn insert(&mut self, identifier: Identifier<'r>, mut value: Vec<Token<'r>>) {
        assert!(!value.is_empty());
        self.resolve(&mut value);
        self.abbrevs.insert(identifier, value);
    }

    pub fn get(&self, identifier: &Identifier<'r>) -> Option<&[Token<'r>]> {
        self.abbrevs.get(identifier).map(Vec::as_slice)
    }

    pub fn get_merged(&self, identifier: &Identifier<'r>) -> Option<Cow<'r, str>> {
        try_merge(self.abbrevs.get(identifier)?)
    }

    pub fn resolve(&mut self, tokens: &mut Vec<Token<'r>>) {
        self.buffer.clear();
        for token in tokens.drain(..) {
            if let Token::Abbrev(ref identifier) = token {
                match self.abbrevs.get(identifier) {
                    Some(sub) => self.buffer.extend(sub.iter().cloned()),
                    None => self.buffer.push(token),
                };
            } else {
                self.buffer.push(token);
            }
        }
        tokens.append(&mut self.buffer);
    }
}
