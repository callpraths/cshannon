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

use crate::code::{Alphabet, Letter};
use crate::tokens::Token;
use anyhow::Result;
use log::{debug, log_enabled, Level};
use std::collections::HashMap;

pub mod balanced_tree;
pub mod fano;
pub mod shannon;

/// An `Encoding` maps `Token`s to `Letter`s.
///
/// `Encoding`s are usually created by processing a `Model`.
#[derive(Debug, Eq, PartialEq)]
pub struct Encoding<T: Token> {
    map: HashMap<T, Letter>,
    alphabet: Alphabet,
}

impl<T: Token> Encoding<T> {
    /// The `Alphabet` of `Letter`s used by this `Encoding`.
    ///
    /// The `Letter`s are returned in a stable order.
    pub fn alphabet(&self) -> &Alphabet {
        &self.alphabet
    }

    /// The `Token`s covered by this `Encoding`.
    ///
    /// The `Token`s are returned in a stable order corresponding to the order
    /// of `Letter`s in self.alphabet()
    pub fn tokens(&self) -> Vec<T> {
        let m = self.reverse_map();
        let mut letters: Vec<&Letter> = self.map.values().collect();
        letters.sort();
        letters.into_iter().map(|l| m[l].clone()).collect()
    }

    /// The `Encoding` map.
    ///
    /// Exposes the internal HashMap via an immutable reference.
    pub fn map(&self) -> &HashMap<T, Letter> {
        &self.map
    }

    /// `Encoding` implementations should use `new` to create an `Encoding`.
    fn new(map: HashMap<T, Letter>) -> Result<Self> {
        let mut letters: Vec<&Letter> = map.values().collect();
        letters.sort();
        let alphabet = Alphabet::new(letters.into_iter().cloned().collect())?;
        log_encoder_ring(&map);
        Ok(Self { map, alphabet })
    }

    fn reverse_map(&self) -> HashMap<&Letter, &T> {
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
fn from_pairs<T: Token>(data: &[(T, Letter)]) -> Result<Encoding<T>> {
    Encoding::new(data.iter().cloned().collect())
}

fn log_encoder_ring<T: Token>(m: &HashMap<T, Letter>) {
    if !log_enabled!(Level::Debug) {
        return;
    }
    debug!("Encoder ring:");
    for (t, l) in m.iter() {
        debug!("  |{}|: |{}|", t, l);
    }
}
