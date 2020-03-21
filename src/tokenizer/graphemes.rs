//! graphemes module implements tokenization of a string into unicode grapheme
//! clusters.
//!
//! The stream makes zero copies internally while iterating over the stream.

use crate::tokenizer::generic::{Token, Tokens};

use unicode_segmentation::{self, UnicodeSegmentation};

use std::fmt;
use std::hash::Hash;

#[derive(Debug, PartialEq, Eq, Hash)]
pub struct Grapheme<'a>(&'a str);

impl std::fmt::Display for Grapheme<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl Token for Grapheme<'_> {
    fn bit_count(&self) -> usize {
        self.0.len() * 8
    }
}

pub struct Graphemes<'a>(unicode_segmentation::Graphemes<'a>);

impl<'a> Iterator for Graphemes<'a> {
    type Item = Grapheme<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.0.next() {
            Some(b) => Some(Grapheme(b)),
            None => None,
        }
    }
}

impl<'a> Tokens<'a> for Graphemes<'a> {
    fn from_text(text: &'a str) -> Self {
        Graphemes(UnicodeSegmentation::graphemes(text, true))
    }
}
