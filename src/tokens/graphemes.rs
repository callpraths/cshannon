//! graphemes module implements tokenization of a string into unicode grapheme
//! clusters.
//!
//! The stream makes zero copies internally while iterating over the stream.

use super::string_parts;
use crate::tokens::{Result, Token, TokenIter, Tokens};

use unicode_segmentation::{self, UnicodeSegmentation};

use std::convert::{From, Into};
use std::fmt;
use std::hash::Hash;

/// A `Token` consisting of a Unicode Grapheme.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Grapheme(String);

/// Pack an iterator of `Grapheme`s to a `Write`er.
pub fn pack<I, W>(i: I, w: &mut W) -> Result<()>
where
    I: std::iter::Iterator<Item = Grapheme>,
    W: std::io::Write,
{
    string_parts::pack::<Grapheme, I, W>(i, w)
}

/// Unpack `Grapheme`s from a `Read`er into a `TokenIter`.
pub fn unpack<R: std::io::Read>(r: &mut R) -> impl TokenIter<T = Grapheme> {
    string_parts::unpack::<Grapheme, R>(r)
}

/// Deprecated Tokens implementation for Grapheme.
pub struct Graphemes<'a>(unicode_segmentation::Graphemes<'a>);

impl From<String> for Grapheme {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl Into<String> for Grapheme {
    fn into(self) -> String {
        self.0
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
