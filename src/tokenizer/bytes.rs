use std::fmt;
use std::hash::Hash;

use crate::tokenizer::generic::{Token, TokenStream};

#[derive(Debug, PartialEq, Eq, Hash)]
pub struct Byte(u8);

impl std::fmt::Display for Byte {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl Token for Byte {
    fn bit_count() -> usize {
        8
    }
}

pub struct ByteStream<'a>(std::str::Bytes<'a>);

impl ByteStream<'_> {
    pub fn new<'a>(text: &'a str) -> ByteStream<'a> {
        ByteStream(text.bytes())
    }
}

impl std::iter::Iterator for ByteStream<'_> {
    type Item = Byte;

    fn next(&mut self) -> Option<Self::Item> {
        match self.0.next() {
            Some(b) => Some(Byte(b)),
            None => None,
        }
    }
}

impl TokenStream for ByteStream<'_> {}
