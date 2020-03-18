// Tokens must be usable as keys in std::collections::HashMap
pub trait Token: ToString + Eq + std::hash::Hash {
    // The number of bits of source text contained in this Token.
    fn bit_count() -> usize;
}

// We'd like to be able to define a single fn pointer type that returns an
// Iterator over some type that satisfies Token, but this is not possible yet.
// https://github.com/rust-lang/rfcs/blob/master/text/1522-conservative-impl-trait.md
pub type Tokenizer<'a, I>
where
    I: 'a,
    I: std::iter::Iterator,
    I::Item: Token,
= fn(&'a str) -> I;
