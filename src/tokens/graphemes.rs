//! graphemes module implements tokenization of a string into unicode grapheme
//! clusters.
//!
//! The stream makes zero copies internally while iterating over the stream.

use super::string_parts;
use crate::tokens::Token;

use std::convert::{From, Into};
use std::fmt;
use std::hash::Hash;

/// A `Token` consisting of a Unicode Grapheme.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Grapheme(String);

/// An iterator for `Grapheme`s read from a `Read`er.
pub type GraphemeIter = string_parts::StringPartsIter<Grapheme>;

/// A `TokenPacker` for packing `Grapheme`s.
pub type GraphemePacker = string_parts::StringPartsPacker<Grapheme>;

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

#[cfg(test)]
mod tests {
    use super::super::{TokenIter, TokenPacker};
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
        let d = GraphemeIter::unpack(&mut r);
        let i = d.map(|t| t.unwrap());
        let mut wc: Cursor<Vec<u8>> = Cursor::new(vec![]);
        GraphemePacker::pack(i, &mut wc).unwrap();
        let got = std::str::from_utf8(&wc.get_ref()[..]).unwrap();
        assert_eq!(got, TEXT);
    }
}
