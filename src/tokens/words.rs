// Copyright 2020 Prathmesh Prabhu
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! words module implements tokenization of a string into [Unicode words].
//!
//! Unicode words do not include punctuation marks, spaces etc. Thus, the
//! original string is not always recoverable by concatenating the tokens.
//!
//! The stream makes zero copies internally while iterating over the stream.
//!
//! [Unicode words]: http://www.unicode.org/reports/tr29/

use super::string_parts;
use crate::tokens::{Token, TokenIter, Tokenizer};

use anyhow::Result;
use std::convert::{From, Into};
use std::fmt;
use std::hash::Hash;

/// A [`Token`] consisting of a Unicode word.
///
/// Tokenizing to `Word` is lossy because non-word characters (e.g.
/// punctuations) are lost.
#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Word(String);

/// Provides a method to create a [`Word`] stream from text.
pub type WordIter = string_parts::StringPartsIter<Word>;

pub struct WordTokenizer;

impl Tokenizer for WordTokenizer {
    type T = Word;
    type Iter<R: std::io::Read> = WordIter;

    fn tokenize<R: std::io::Read>(r: R) -> Result<Self::Iter<R>> {
        WordIter::unpack(r)
    }
}

/// Provides a method to pack a [`Word`] stream to text.
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
    type Tokenizer = WordTokenizer;
    type Packer = WordPacker;

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
        let d = WordIter::unpack(&mut r).unwrap();
        let i = d.map(|i| match i {
            Err(e) => panic!("{}", e),
            Ok(b) => b,
        });
        let mut wc: Cursor<Vec<u8>> = Cursor::new(vec![]);
        WordPacker::pack(i, &mut wc).unwrap();
        let got = std::str::from_utf8(&wc.get_ref()[..]).unwrap();
        assert_eq!(got, TEXT);
    }
}
