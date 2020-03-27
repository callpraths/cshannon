pub mod bytes;
pub mod graphemes;
pub mod words;

/// A single item in the tokenized stream from a string input.
///
/// Tokens may be used as keys in std::collections::HashMap.
pub trait Token: ToString + Eq + std::hash::Hash {
    // The number of bits of source text contained in this Token.
    fn bit_count(&self) -> usize;
}

/// A stream of Tokens corresponding to a raw string input.
pub trait Tokens<'a>: std::iter::IntoIterator<Item: Token> {
    fn from_text(text: &'a str) -> Self;
    /// Convert back to text that would tokenize to this Token stream.
    ///
    /// Some Tokens may be lossy for specific text. Thus,
    ///   to_text(from_text(text));
    /// may not be the same as text.
    fn to_text(self) -> Result<String, String>;
}
