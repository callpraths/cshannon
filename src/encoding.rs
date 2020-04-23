use crate::code::{Alphabet, Letter};
use crate::tokens::Token;
use std::collections::HashMap;

pub mod balanced_tree;

/// Alias for results returned from encodings.
pub type Result<T> = std::result::Result<T, String>;

/// An `Encoding` maps `Token`s to `Letter`s.
///
/// `Encoding`s are usually created by processing a `Model`.
pub struct Encoding<T: Token>(HashMap<T, Letter>);

impl<T: Token> Encoding<T> {
    /// The `Alphabet` of `Letter`s used by this `Encoding`.
    pub fn alphabet(&self) -> Alphabet {
        let mut letters: Vec<Letter> = self.0.values().cloned().collect();
        letters.sort_unstable();
        Alphabet::new(letters)
    }

    /// The `Encoding` map.
    ///
    /// Exposes the internal HashMap via an immutable reference.
    pub fn map(&self) -> &HashMap<T, Letter> {
        &self.0
    }
}
