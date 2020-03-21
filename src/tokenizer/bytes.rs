//! bytes module implements tokenization of a string into bytes.
//!
//! The stream makes zero copies internally while iterating over the stream.

use crate::tokenizer::generic::{Token, Tokens};

use std::fmt;
use std::hash::Hash;

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

impl std::iter::Iterator for Bytes<'_> {
    type Item = Byte;

    fn next(&mut self) -> Option<Self::Item> {
        match self.0.next() {
            Some(b) => Some(Byte(b)),
            None => None,
        }
    }
}

impl<'a> Tokens<'a> for Bytes<'a> {
    fn from_text(text: &'a str) -> Self {
        Bytes(text.bytes())
    }
    fn to_text(self) -> Result<String, String> {
        let b: Vec<u8> = self.0.collect();
        let s = std::str::from_utf8(&b).map_err(|e| e.to_string())?;
        Ok(s.to_string())
    }
}
