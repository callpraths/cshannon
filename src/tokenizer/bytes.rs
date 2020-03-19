use std::fmt;
use std::hash::Hash;

use crate::tokenizer::generic::{Token, Tokens};

#[derive(Debug, PartialEq, Eq, Hash)]
pub struct Byte(u8);

impl std::fmt::Display for Byte {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl Token for Byte {
    fn bit_count(&self) -> usize {
        8
    }
}

pub struct Bytes<'a>(std::str::Bytes<'a>);

impl Bytes<'_> {
    pub fn new<'a>(text: &'a str) -> Bytes<'a> {
        Bytes(text.bytes())
    }
}

impl std::iter::Iterator for Bytes<'_> {
    type Item = Byte;

    fn next(&mut self) -> Option<Self::Item> {
        match self.0.next() {
            Some(b) => Some(Byte(b)),
            None => None,
        }
    }
}

impl Tokens for Bytes<'_> {}
