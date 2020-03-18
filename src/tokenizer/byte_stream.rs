use std::fmt;
use std::hash::Hash;

use crate::tokenizer::generic::{Token, Tokenizer};

#[derive(Debug, PartialEq, Eq, Hash)]
pub struct Byte {}

impl std::fmt::Display for Byte {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        Ok(())
    }
}

impl Token for Byte {
    fn bit_count() -> usize {
        0
    }
}

pub struct ByteStream<'a> {
    text: &'a str,
}

impl std::iter::Iterator for ByteStream<'_> {
    type Item = Byte;

    fn next(&mut self) -> Option<Self::Item> {
        None
    }
}

pub fn byte_tokenizer<'a>(text: &'a str) -> ByteStream<'a> {
    ByteStream { text }
}

// validate type.
static TOKENIZER_FN: Tokenizer<ByteStream> = byte_tokenizer;
