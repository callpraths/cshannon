use crate::tokenizer::generic::{Token, Tokens};

use unicode_segmentation::{self, UnicodeSegmentation};

use std::fmt;
use std::hash::Hash;

#[derive(Debug, PartialEq, Eq, Hash)]
pub struct Word<'a>(&'a str);

impl std::fmt::Display for Word<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl Token for Word<'_> {
    fn bit_count(&self) -> usize {
        self.0.len() * 8
    }
}

pub struct Words<'a>(unicode_segmentation::UnicodeWords<'a>);

impl Words<'_> {
    pub fn new<'a>(text: &'a str) -> Words<'a> {
        Words(UnicodeSegmentation::unicode_words(text))
    }
}

impl<'a> Iterator for Words<'a> {
    type Item = Word<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.0.next() {
            Some(b) => Some(Word(b)),
            None => None,
        }
    }
}

impl Tokens for Words<'_> {}