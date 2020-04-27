use std::convert::TryFrom;
use std::fmt::Display;
use std::io::Cursor;

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
/// See unpack_all() for the reverse operation.
pub fn pack_all<W, T, TP>(tokens: Vec<T>, w: &mut W) -> Result<()>
where
    W: std::io::Write,
    T: Token,
    TP: TokenPacker<W, T = T>,
{
    let size = tokens.iter().fold(0, |sum, t| sum + t.bit_count()) / 8;
    w.write_all(&pack_u64(size as u64))
        .map_err(|e| e.to_string())?;
    TP::pack(tokens.into_iter(), w)?;
    Ok(())
}

/// Unpacks a vector of tokens previously packed with pack_with_len().
pub fn unpack_all<R, T, TI>(mut r: R) -> Result<Vec<T>>
where
    R: std::io::Read,
    T: Token,
    TI: TokenIter<Cursor<Vec<u8>>, T = T>,
{
    let size = unpack_u64(&mut r)?;
    let safe_size = usize::try_from(size).map_err(|e| e.to_string())?;
    let mut buf = vec![0u8; safe_size];
    r.read_exact(&mut buf).map_err(|e| e.to_string()).unwrap();
    TI::unpack(Cursor::new(buf)).collect()
}

// TODO: dedup with code::common::pack_u64()
fn pack_u64(s: u64) -> Vec<u8> {
    s.to_be_bytes().to_vec()
}

// TODO:: dedup with code::common::unpack_u64()
pub fn unpack_u64<R: std::io::Read>(mut r: R) -> Result<u64> {
    let mut buf: [u8; 8] = [0; 8];
    r.read_exact(&mut buf).map_err(|e| e.to_string()).unwrap();
    Ok(u64::from_be_bytes(buf))
}

#[cfg(test)]
mod roundtrip_with_len_tests {
    use super::bytes::{self, Byte, ByteIter, BytePacker};
    use super::graphemes::{Grapheme, GraphemeIter, GraphemePacker};
    use super::*;
    use std::io::{Cursor, Read};
    #[test]
    fn empty() {
        let tokens = Vec::<Byte>::new();
        let mut buf = Vec::<u8>::new();
        assert!(pack_all::<_, _, BytePacker>(tokens.clone(), &mut buf).is_ok());
        let got = unpack_all::<_, _, ByteIter<_>>(Cursor::new(&mut buf)).unwrap();
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
        assert!(pack_all::<_, _, BytePacker>(tokens.clone(), &mut buf).is_ok());
        let got = unpack_all::<_, _, ByteIter<_>>(Cursor::new(&mut buf)).unwrap();
        assert_eq!(got, tokens);
    }

    #[test]
    fn trailing_byte_data() {
        let tokens = vec![bytes::new(0), bytes::new(1), bytes::new(2)];
        let mut buf = Vec::<u8>::new();
        assert!(pack_all::<_, _, BytePacker>(tokens.clone(), &mut buf).is_ok());

        // The following trailing data should be ignored.
        buf.push(0b1111_1111);

        let mut r = Cursor::new(buf);
        let got = unpack_all::<_, _, ByteIter<_>>(&mut r).unwrap();
        assert_eq!(got, tokens);

        // The buffer should not be read beyond the trailing data.
        let mut buf = Vec::<u8>::new();
        assert_eq!(r.read_to_end(&mut buf).unwrap(), 1);
        assert_eq!(buf, vec![0b1111_1111u8]);
    }

    #[test]
    fn trailing_grapheme_data() {
        let tokens = vec![
            Grapheme::from("a".to_owned()),
            Grapheme::from("b".to_owned()),
        ];
        let mut buf = Vec::<u8>::new();
        assert!(pack_all::<_, _, GraphemePacker>(tokens.clone(), &mut buf).is_ok());

        // The following trailing data should be ignored.
        buf.push(0b1111_1111);

        let mut r = Cursor::new(buf);
        let got = unpack_all::<_, _, GraphemeIter>(&mut r).unwrap();
        assert_eq!(got, tokens);

        // The buffer should not be read beyond the trailing data.
        let mut buf = Vec::<u8>::new();
        assert_eq!(r.read_to_end(&mut buf).unwrap(), 1);
        assert_eq!(buf, vec![0b1111_1111u8]);
    }
}
