use std::fmt::Display;

pub mod bytes;
pub mod graphemes;
mod string_parts;
pub mod test_utils;
pub mod words;

pub type Result<T> = std::result::Result<T, String>;

/// A single item in the tokenized stream from a string input.
///
/// Tokens may be used as keys in std::collections::HashMap.
pub trait Token: Clone + Display + Eq + std::hash::Hash {
    // The number of bits of source text contained in this Token.
    fn bit_count(&self) -> usize;
}

/// An iterator for `Token`s read from a `Read`er.
///
/// Errors in reading tokens are reported in-stream.
/// All token implementations return TokenIter from the associated unpack()
/// functions.
pub trait TokenIter<R>: std::iter::Iterator<Item = Result<<Self as TokenIter<R>>::T>>
where
    R: std::io::Read,
{
    type T: Token;

    fn unpack(r: R) -> Self;
}

pub trait TokenPacker<W>
where
    W: std::io::Write,
{
    type T: Token;

    fn pack<I>(i: I, w: &mut W) -> Result<()>
    where
        I: std::iter::Iterator<Item = Self::T>;
}
