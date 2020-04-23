///! Contains Token implementations intended to help write unit tests.
use super::Token;
use std::fmt;

/// A `Token` that wraps i32 values.
///
/// Useful for unittests against the Token trait.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct I32Token(pub i32);

impl fmt::Display for I32Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl Token for I32Token {
    fn bit_count(&self) -> usize {
        4
    }
}
