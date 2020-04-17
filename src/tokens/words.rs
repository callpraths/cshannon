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

/// A `Token` consisting of a Unicode word.
///
/// Tokenizing to `Word`s is lossy because non-word characters (e.g.
/// punctuations) are lost.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Word(String);

/// Pack an iterator of `Word`s to a `Write`er.
pub fn pack<I, W>(i: I, w: &mut W) -> Result<()>
where
    I: std::iter::Iterator<Item = Word>,
    W: std::io::Write,
{
    string_parts::pack::<Word, I, W>(i, w)
}

/// Unpack `Word`s from a `Read`er into a `TokenIter`.
pub fn unpack<R: std::io::Read>(r: &mut R) -> impl TokenIter<T = Word> {
    string_parts::unpack::<Word, R>(r)
}
/// Deprecated Tokens implementation for Word.
pub struct Words<'a>(unicode_segmentation::UnicodeWords<'a>);

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

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    const TEXT: &str = "
Ah! well a-day! what evil looks
Had I from old and young!
Instead of the cross, the Albatross
About my neck was hung.
";
    #[test]
    fn roundtrip() {
        let mut r = Cursor::new(TEXT);
        let d = unpack(&mut r);
        let i = d.map(|i| match i {
            Err(e) => panic!(e),
            Ok(b) => b,
        });
        let mut wc: Cursor<Vec<u8>> = Cursor::new(vec![]);
        pack(i, &mut wc).unwrap();
        let got = std::str::from_utf8(&wc.get_ref()[..]).unwrap();
        assert_eq!(got, TEXT);
    }
}
