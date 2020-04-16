//! graphemes module implements tokenization of a string into unicode grapheme
//! clusters.
//!
//! The stream makes zero copies internally while iterating over the stream.

use super::string_parts;
use crate::tokens::{Result, Token, TokenIter, Tokens};

use unicode_segmentation::{self, UnicodeSegmentation};

use std::convert::From;
use std::fmt;
use std::hash::Hash;

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Grapheme(String);

impl From<String> for Grapheme {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl std::fmt::Display for Grapheme {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl Token for Grapheme {
    fn bit_count(&self) -> usize {
        self.0.len() * 8
    }
}

pub struct Graphemes<'a>(unicode_segmentation::Graphemes<'a>);

impl<'a> Iterator for Graphemes<'a> {
    type Item = Grapheme;

    fn next(&mut self) -> Option<Self::Item> {
        match self.0.next() {
            Some(b) => Some(Grapheme(b.to_owned())),
            None => None,
        }
    }
}

impl<'a> Tokens<'a> for Graphemes<'a> {
    fn from_text(text: &'a str) -> Self {
        Graphemes(UnicodeSegmentation::graphemes(text, true))
    }
    fn to_text(self) -> Result<String> {
        Ok(self.0.collect())
    }
}

pub fn unpack<R: std::io::Read>(r: &mut R) -> impl TokenIter<T = Grapheme> {
    string_parts::unpack::<Grapheme, R>(r)
}
