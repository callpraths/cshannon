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

//! Exports [`Model`], a statically computed zero order model over a [`Token`]
//! stream.
//!
//! The function [`from`] consumes a [`TokenIter`] to generate a [`Model`].
//!
//! [`from`]: fn.from.html
//! [`Model`]: struct.Model.html
//! [`Token`]: ../tokens/trait.Token.html
//! [`TokenIter`]: ../tokens/trait.TokenIter.html

use crate::tokens::Token;
use std::collections::HashMap;

/// A statically computed zero order model for compression.
///
/// The model exports certain statistics on input [`Token`] set that are useful
/// for statistical compression techniques.
///
/// [`Token`]: ../tokens/trait.Token.html
pub struct Model<T: Token>(HashMap<T, Stats>);

struct Stats {
    f: u64,
    p: f64,
}

impl<T: Token> Model<T> {
    /// Frequency of occurrence of a [`Token`].
    ///
    /// [`Token`]: ../tokens/trait.Token.html
    pub fn frequency(&self, t: &T) -> u64 {
        match self.0.get(t) {
            Some(s) => s.f,
            None => 0,
        }
    }

    /// Probability of occurrence of a [`Token`].
    ///
    /// [`Token`]: ../tokens/trait.Token.html
    pub fn probability(&self, t: &T) -> f64 {
        match self.0.get(t) {
            Some(s) => s.p,
            None => 0.0,
        }
    }

    /// [`Token`] count in the model.
    ///
    /// [`Token`]: ../tokens/trait.Token.html
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Return whether this model is empty.
    ///
    /// Semantically equivalent, but possibly faster, implementation of
    /// `(self.len() == 0)`
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Return the [`Token`] set in the model sorted by relative frequencies,
    /// highest first.
    ///
    /// [`Token`]: ../tokens/trait.Token.html
    pub fn tokens_sorted(&self) -> Vec<T> {
        let mut keys = Vec::with_capacity(self.0.len());
        for k in self.0.keys() {
            keys.push((*k).clone());
        }
        keys.sort_unstable_by(|x, y| self.frequency(y).cmp(&self.frequency(x)));
        keys
    }
}

/// Generate a zero order model from the given [`Token`] stream.
///
/// [`Token`]: ../tokens/trait.Token.html
pub fn from<T, TS>(ts: TS) -> Model<T>
where
    T: Token,
    TS: std::iter::IntoIterator<Item = T>,
{
    let mut m = Model::<T>(HashMap::new());
    let mut d: i64 = 0;
    for t in ts {
        let s = m.0.entry(t).or_insert(Stats { f: 0, p: 0.0 });
        (*s).f += 1;
        d += 1;
    }
    for s in m.0.values_mut() {
        (*s).p = ((*s).f as f64) / (d as f64);
    }
    m
}

/// Instantiate a zero order model from the given precomputed frequencies.
///
/// Intended to be used only from unit-tests, to avoid dependence on internal
/// computation of frequencies in [`from`].
///
/// [`from`]: fn.from.html
/// [`Token`]: ../tokens/trait.Token.html
pub fn with_frequencies<T: Token>(fs: &[(T, u64)]) -> Model<T> {
    let fs: HashMap<T, u64> = fs.to_vec().into_iter().collect();
    let total = fs.values().sum::<u64>() as f64;
    let mut m = Model::<T>(HashMap::new());
    for (t, f) in fs.into_iter() {
        m.0.insert(
            t,
            Stats {
                f,
                p: (f as f64) / total,
            },
        );
    }
    m
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tokens::test_utils::I32Token;

    #[test]
    fn basic() {
        let tokens = vec![
            I32Token(2),
            I32Token(3),
            I32Token(1),
            I32Token(2),
            I32Token(5),
            I32Token(11),
        ];
        let m = from(tokens);
        assert_eq!(m.frequency(&I32Token(1)), 1);
        assert_eq!(m.frequency(&I32Token(2)), 2);
        assert_eq!(m.frequency(&I32Token(13)), 0);

        // f64 equality is inexact.
        assert!(m.probability(&I32Token(5)) > 0.166);
        assert!(m.probability(&I32Token(5)) < 0.167);
    }

    #[test]
    fn with_frequencies() {
        let m = super::with_frequencies(&[
            (I32Token(2), 2),
            (I32Token(3), 1),
            (I32Token(1), 1),
            (I32Token(5), 1),
            (I32Token(11), 1),
        ]);
        assert_eq!(m.frequency(&I32Token(1)), 1);
        assert_eq!(m.frequency(&I32Token(2)), 2);
        assert_eq!(m.frequency(&I32Token(13)), 0);

        // f64 equality is inexact.
        assert!(m.probability(&I32Token(5)) > 0.166);
        assert!(m.probability(&I32Token(5)) < 0.167);
    }
}
