// Tokens must be usable as keys in std::collections::HashMap
pub trait Token: ToString + Eq + std::hash::Hash {
    // The number of bits of source text contained in this Token.
    fn bit_count() -> usize;
}

pub trait TokenStream: std::iter::IntoIterator<Item: Token> {}
