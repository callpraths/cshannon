//! words module implements tokenization of a string into unicode words.
//!
//! Unicode words do not include punctuation marks, spaces etc. Thus, the
//! original string is not always recoverable by concatenating the tokens.
//!
//! The stream makes zero copies internally while iterating over the stream.

use super::string_parts;
use crate::tokens::Token;

use std::convert::{From, Into};
use std::fmt;
use std::hash::Hash;

/// A `Token` consisting of a Unicode word.
///
/// Tokenizing to `Word`s is lossy because non-word characters (e.g.
/// punctuations) are lost.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Word(String);

/// An iterator for `Word`s read from a `Read`er.
pub type WordIter = string_parts::StringPartsIter<Word>;

/// A `TokenPacker` to pack `Word`s.
pub type WordPacker = string_parts::StringPartsPacker<Word>;

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
        let d = WordIter::unpack(&mut r);
        let i = d.map(|i| match i {
            Err(e) => panic!(e),
            Ok(b) => b,
        });
        let mut wc: Cursor<Vec<u8>> = Cursor::new(vec![]);
        WordPacker::pack(i, &mut wc).unwrap();
        let got = std::str::from_utf8(&wc.get_ref()[..]).unwrap();
        assert_eq!(got, TEXT);
    }
}
