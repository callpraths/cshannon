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
use crate::model::ModelKey;
use anyhow::Result;
use log::{debug, log_enabled, Level};
use std::collections::HashMap;

pub mod balanced_tree;
pub mod fano;
pub mod huffman;
pub mod shannon;

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
