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

//! Contains Token implementations intended to help write unit tests.

use crate::tokens::{TokenPacker, Tokenizer};

use super::Token;
use anyhow::Result;
use std::fmt;

/// A [`Token`] that wraps i32 values.
///
/// Useful for unittests against the [`Token`] trait.
///
/// [`Token`]: ../trait.Token.html
#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct I32Token(pub i32);

impl fmt::Display for I32Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl Token for I32Token {
    type Tokenizer = I32Tokenizer;
    type Packer = I32TokenPacker;

    fn bit_count(&self) -> usize {
        4
    }
}

pub struct I32Tokenizer;

impl Tokenizer for I32Tokenizer {
    type T = I32Token;
    type Iter<R: std::io::Read> = I32TokenIter<R>;

    fn tokenize<R: std::io::Read>(r: R) -> Result<Self::Iter<R>> {
        Ok(I32TokenIter(r))
    }
}

pub struct I32TokenIter<R: std::io::Read>(R);

impl<R: std::io::Read> std::iter::Iterator for I32TokenIter<R> {
    type Item = Result<I32Token>;
    fn next(&mut self) -> Option<Self::Item> {
        panic!("Not implemented!");
    }
}

pub struct I32TokenPacker;

impl TokenPacker for I32TokenPacker {
    type T = I32Token;

    fn pack<I, W: std::io::Write>(_i: I, _w: W) -> Result<()>
    where
        I: std::iter::Iterator<Item = Self::T>,
    {
        panic!("Not implemented!")
    }
}
