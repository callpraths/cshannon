use crate::code::{Alphabet, Letter};
use crate::tokens::Token;
use std::collections::HashMap;

pub mod balanced_tree;

/// An `Encoding` maps `Token`s to `Letter`s.
///
/// `Encoding`s are usually created by processing a `Model`.
pub struct Encoding<T: Token>(HashMap<T, Letter>);

impl<T: Token> Encoding<T> {
    /// The `Alphabet` of `Letter`s used by this `Encoding`.
    ///
    /// The `Letter`s are returned in a stable order.
    pub fn alphabet(&self) -> Alphabet {
        Alphabet::new(self.sorted_letters().into_iter().cloned().collect())
    }

    /// The `Token`s covered by this `Encoding`.
    ///
    /// The `Token`s are returned in a stable order corresponding to the order
    /// of `Letter`s in self.alphabet()
    pub fn tokens(&self) -> Vec<T> {
        let m = self.reverse_map();
        self.sorted_letters()
            .into_iter()
            .map(|l| m[l].clone())
            .collect()
    }

    /// The `Encoding` map.
    ///
    /// Exposes the internal HashMap via an immutable reference.
    pub fn map(&self) -> &HashMap<T, Letter> {
        &self.0
    }

    fn sorted_letters(&self) -> Vec<&Letter> {
        let mut letters: Vec<&Letter> = self.0.values().collect();
        letters.sort();
        letters
    }

    fn reverse_map(&self) -> HashMap<&Letter, &T> {
        let mut m = HashMap::new();
        for (t, l) in &self.0 {
            m.insert(l, t);
        }
        m
    }
}
