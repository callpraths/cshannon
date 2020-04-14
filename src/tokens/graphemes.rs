//! graphemes module implements tokenization of a string into unicode grapheme
//! clusters.
//!
//! The stream makes zero copies internally while iterating over the stream.

use crate::tokens::{Result, Token, TokenIter, Tokens};

use unicode_segmentation::{self, UnicodeSegmentation};

use std::fmt;
use std::hash::Hash;

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Grapheme(String);

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
    GraphemeIter::new(r)
}

pub struct GraphemeIter(Option<Result<std::vec::IntoIter<Grapheme>>>);

impl GraphemeIter {
    fn new<R>(r: &mut R) -> Self
    where
        R: std::io::Read,
    {
        let mut data = Vec::<u8>::new();
        if let Err(e) = r.read_to_end(&mut data) {
            return GraphemeIter(Some(Err(e.to_string())));
        }
        match std::str::from_utf8(&data) {
            Err(e) => GraphemeIter(Some(Err(e.to_string()))),
            Ok(s) => {
                let mut parts = Vec::<Grapheme>::new();
                for g in s.graphemes(true) {
                    parts.push(Grapheme(g.to_owned()));
                }
                GraphemeIter(Some(Ok(parts.into_iter())))
            }
        }
    }
}

impl TokenIter<'_> for GraphemeIter {
    type T = Grapheme;
}

impl<'a> std::iter::Iterator for GraphemeIter {
    type Item = Result<Grapheme>;

    fn next(&mut self) -> Option<Self::Item> {
        let mut store = None;
        std::mem::swap(&mut self.0, &mut store);
        let result = match &mut store {
            None => {
                return None;
            }
            Some(n) => match n {
                Err(e) => return Some(Err((*e).to_string())),
                Ok(i) => match i.next() {
                    None => None,
                    Some(g) => Some(Ok(g)),
                },
            },
        };
        std::mem::swap(&mut store, &mut self.0);
        result
    }
}
