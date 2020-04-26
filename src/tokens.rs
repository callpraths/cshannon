use std::convert::TryFrom;
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

/// Packs a vector of tokens prefixed with the length of the vector.
///
/// See unpack_with_len() for the reverse operation.
pub fn pack_with_len<W, T, TP>(tokens: Vec<T>, w: &mut W) -> Result<()>
where
    W: std::io::Write,
    T: Token,
    TP: TokenPacker<W, T = T>,
{
    w.write_all(&pack_u64(tokens.len() as u64))
        .map_err(|e| e.to_string())?;
    TP::pack(tokens.into_iter(), w)?;
    Ok(())
}

/// Unpacks a vector of tokens previously packed with pack_with_len().
pub fn unpack_with_len<R, T, TI>(mut r: R) -> Result<Vec<T>>
where
    R: std::io::Read,
    T: Token,
    TI: TokenIter<R, T = T>,
{
    let len = unpack_u64(&mut r)?;
    let safe_len = usize::try_from(len).map_err(|e| e.to_string())?;
    TI::unpack(r).take(safe_len).collect()
}

// TODO: dedup with code::common::pack_u64()
fn pack_u64(s: u64) -> Vec<u8> {
    s.to_be_bytes().to_vec()
}

// TODO:: dedup with code::common::unpack_u64()
pub fn unpack_u64<R: std::io::Read>(mut r: R) -> Result<u64> {
    let mut buf: [u8; 8] = [0; 8];
    r.read_exact(&mut buf).map_err(|e| e.to_string())?;
    Ok(u64::from_be_bytes(buf))
}

#[cfg(test)]
mod roundtrip_with_len_tests {
    use super::bytes::{self, Byte, ByteIter, BytePacker};
    use super::*;
    use std::io::Cursor;
    #[test]
    fn empty() {
        let tokens = Vec::<Byte>::new();
        let mut buf = Vec::<u8>::new();
        assert!(pack_with_len::<_, _, BytePacker>(tokens.clone(), &mut buf).is_ok());
        let got = unpack_with_len::<_, _, ByteIter<_>>(Cursor::new(&mut buf)).unwrap();
        assert_eq!(got, tokens);
    }

    #[test]
    fn non_empty() {
        let tokens = vec![
            bytes::new(0),
            bytes::new(1),
            bytes::new(2),
            bytes::new(3),
            bytes::new(0),
            bytes::new(4),
            bytes::new(5),
            bytes::new(0),
            bytes::new(1),
            bytes::new(0),
        ];
        let mut buf = Vec::<u8>::new();
        assert!(pack_with_len::<_, _, BytePacker>(tokens.clone(), &mut buf).is_ok());
        let got = unpack_with_len::<_, _, ByteIter<_>>(Cursor::new(&mut buf)).unwrap();
        assert_eq!(got, tokens);
    }
}
