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

//! This module provides traits for tokenizing text.
//!
//! The [`Token`] trait is the primary exported type from this module.
//!
//! The [`TokenIter`] trait provides a method to unpack text into a [`Token`]
//! stream. The [`TokenPacker`] trait provides the opposite method to convert a
//! [`Token`] stream to text.
//!
//! Additionally, two methods [`pack_all`] and [`unpack_all`] are provided to
//! work with a [`Token`] set when the number of tokens is known apriori. In
//! particular, [`unpack_all`] is guaranteed to only consume the required amount
//! of data from the input.
//!
//! Three concrete tokenization schemes are exported from sub-modules:
//! [bytes], [graphemes] and [words].
//!
//! [bytes]: bytes/index.html
//! [graphemes]: graphemes/index.html
//! [`pack_all`]: fn.pack_all.html
//! [`Token`]: trait.Token.html
//! [`TokenIter`]: trait.TokenIter.html
//! [`TokenPacker`]: trait.TokenPacker.html
//! [`unpack_all`]: fn.unpack_all.html
//! [words]: words/index.html

use anyhow::Result;
use std::fmt::Display;

pub mod bytes;
pub mod graphemes;
mod string_parts;
pub mod test_utils;
pub mod words;

/// A single item in the tokenized stream from a string input.
///
/// Tokens may be used as keys in a [`HashMap`](std::collections::HashMap).
pub trait Token: Clone + std::fmt::Debug + Display + Eq + std::hash::Hash {
    type Tokenizer: Tokenizer<T = Self>;
    type Packer: TokenPacker<T = Self>;

    // The number of bits of source text contained in this Token.
    fn bit_count(&self) -> usize;
}

pub trait Tokenizer {
    type T: Token;
    type Iter<R: std::io::Read>: std::iter::Iterator<Item = Result<Self::T>>;

    fn tokenize<R: std::io::Read>(r: R) -> Result<Self::Iter<R>>;
}

/// Provides a method to pack a [`Token`] stream to text.
pub trait TokenPacker {
    type T: Token;

    fn pack<I, W: std::io::Write>(i: I, w: &mut W) -> Result<()>
    where
        I: std::iter::Iterator<Item = Self::T>;
}
