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

//! Defines the [`Encoding`] struct that maps an [`EncodingKey`] to a
//! [`Letter`].
//!
//! An [`Encoding`] can be generated from a [`Model`](crate::model::Model) by
//! calling the `new()` function defined in one of the sub-modules:
//! [balanced_tree], [shannon], [fano], or [huffman].

use crate::code::{Alphabet, Letter};
use crate::model::{Model, ModelKey};
use anyhow::Result;
use log::{debug, log_enabled, Level};
use std::collections::HashMap;

pub mod balanced_tree;
pub mod fano;
pub mod huffman;
pub mod shannon;

/// Encoding shcemes supported by this library.
pub enum EncodingScheme {
    /// Create a new balanced tree [`Encoding`].
    ///
    /// All letters in this encoding are fixed width bit strings. The smallest width
    /// necessary to generate the required number of [`Letter`]s is used.
    ///
    /// The generated [`Encoding`] is stable: calling `new` on a [`Model`]
    /// repeatedly yields the same [`Encoding`].
    BalancedTree,
    /// Create a new [Fano encoding].
    ///
    /// [Fano encoding]: https://en.wikipedia.org/wiki/Shannon%E2%80%93Fano_coding
    Fano,
    // Create a new [Huffman encoding].
    //
    // [Huffman encoding]: https://en.wikipedia.org/wiki/Huffman_coding
    Huffman,
    /// Create a new [Shannon encoding].
    ///
    /// The Shannon encoding scheme is defined thus:
    ///
    /// Let the tokens, sorted in decreasing order of frequency be
    /// `t1, t2, t3 ...`
    ///
    /// Let the probability of occurrence of the Tokens be `f1, f2, f3 ...`
    ///
    /// Define the numbers `l1, l2, l3 ...` such that `lk` = `ceil(log2(1/fk))`
    ///
    /// Let the (computed) cumulative proportions be `c1, c2, c3 ...`
    ///
    /// Then, the code is `e1, e2, e3 ...`
    /// such that `ek` = first `lk` bits of the binary expansion of `Fk`.
    ///
    /// [Shannon encoding]: https://en.wikipedia.org/wiki/Shannon%E2%80%93Fano_coding
    Shannon,
}

pub type EncodingConstructor<T> = fn(Model<T>) -> Result<Encoding<T>>;

/// Return the type-appropriate appropriate constructor function for the given
/// encoding scheme.
pub fn encoder_constructor<K: EncodingKey>(scheme: EncodingScheme) -> EncodingConstructor<K> {
    match scheme {
        EncodingScheme::BalancedTree => balanced_tree::new::<K>,
        EncodingScheme::Fano => fano::new::<K>,
        EncodingScheme::Huffman => huffman::new::<K>,
        EncodingScheme::Shannon => shannon::new::<K>,
    }
}

/// Shorthand for trait bounds required for keys in an [`Encoding`].
pub trait EncodingKey: Clone + std::fmt::Debug + Eq + std::hash::Hash {}

impl<K: EncodingKey> ModelKey for K {}

/// Maps an [`EncdingKey`] to a [`Letter`].
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Encoding<K: EncodingKey> {
    map: HashMap<K, Letter>,
    alphabet: Alphabet,
}

impl<K: EncodingKey> Encoding<K> {
    /// The [`Alphabet`] used by this encoding.
    pub fn alphabet(&self) -> &Alphabet {
        &self.alphabet
    }

    /// The set of [`Token`]s covered by this encoding.
    ///
    /// The returned set is sorted in a stable order corresponding to the order
    /// of letters in [`Self::alphabet()`]
    pub fn tokens(&self) -> Vec<K> {
        let m = self.reverse_map();
        let mut letters: Vec<&Letter> = self.map.values().collect();
        letters.sort();
        letters.into_iter().map(|l| m[l].clone()).collect()
    }

    /// The encoding map.
    ///
    /// Exposes the internal [`HashMap`](std::collections::HashMap) via an
    /// immutable reference.
    pub fn map(&self) -> &HashMap<K, Letter> {
        &self.map
    }

    /// `Encoding` implementations should use `new` to create an `Encoding`.
    fn new(map: HashMap<K, Letter>) -> Result<Self> {
        let mut letters: Vec<&Letter> = map.values().collect();
        letters.sort();
        let alphabet = Alphabet::new(letters.into_iter().cloned().collect())?;
        log_encoder_ring(&map);
        Ok(Self { map, alphabet })
    }

    fn reverse_map(&self) -> HashMap<&Letter, &K> {
        let mut m = HashMap::new();
        for (t, l) in &self.map {
            m.insert(l, t);
        }
        m
    }
}

/// Helper function to create a new `Encoding` from known mapping.
///
/// This is a private function useful for checking expected Encoding in
/// unit tests.
#[allow(dead_code)]
fn from_pairs<K: EncodingKey>(data: &[(K, Letter)]) -> Result<Encoding<K>> {
    Encoding::new(data.iter().cloned().collect())
}

fn log_encoder_ring<K: EncodingKey>(m: &HashMap<K, Letter>) {
    if !log_enabled!(Level::Debug) {
        return;
    }
    debug!("Encoder ring:");
    for (k, l) in m.iter() {
        debug!("  |{:?}|: |{:?}|", k, l);
    }
}
