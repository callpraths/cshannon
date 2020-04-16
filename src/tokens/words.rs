//! words module implements tokenization of a string into unicode words.
//!
//! Unicode words do not include punctuation marks, spaces etc. Thus, the
//! original string is not always recoverable by concatenating the tokens.
//!
//! The stream makes zero copies internally while iterating over the stream.

use super::string_parts;
use crate::tokens::{Result, Token, TokenIter, Tokens};

use unicode_segmentation::{self, UnicodeSegmentation};

use std::convert::{From, Into};
use std::fmt;
use std::hash::Hash;

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Word(String);

impl From<String> for Word {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl Into<String> for Word {
    fn into(self) -> String {
        self.0
    }
}

impl std::fmt::Display for Word {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl Token for Word {
    fn bit_count(&self) -> usize {
        self.0.len() * 8
    }
}

pub struct Words<'a>(unicode_segmentation::UnicodeWords<'a>);

impl<'a> Iterator for Words<'a> {
    type Item = Word;

    fn next(&mut self) -> Option<Self::Item> {
        match self.0.next() {
            Some(b) => Some(Word(b.to_owned())),
            None => None,
        }
    }
}

impl<'a> Tokens<'a> for Words<'a> {
    fn from_text(text: &'a str) -> Words<'a> {
        Words(UnicodeSegmentation::unicode_words(text))
    }
    fn to_text(self) -> Result<String> {
        Ok(self
            .0
            .map(|s| s.to_owned())
            .collect::<Vec<String>>()
            .join(" "))
    }
}

pub fn unpack<R: std::io::Read>(r: &mut R) -> impl TokenIter<T = Word> {
    string_parts::unpack::<Word, R>(r)
}

pub fn pack<I, W>(i: I, w: &mut W) -> Result<()>
where
    I: std::iter::Iterator<Item = Word>,
    W: std::io::Write,
{
    string_parts::pack::<Word, I, W>(i, w)
}
