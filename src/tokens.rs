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

//! This module provides traits for tokenizing text.
//!
//! The [`Token`] trait is the primary exported type from this module.
//!
//! The [`TokenIter`] trait provides a method to unpack text into a [`Token`]
//! stream. The [`TokenPacker`] trait provides the opposite method to convert a
//! [`Token`] stream to text.
//!
//! Additionally, two methods [`pack_all`] and [`unpack_all`] are provided to
//! work with a [`Token`] set when the number of tokens is known apriori. In
//! particular, [`unpack_all`] is guaranteed to only consume the required amount
//! of data from the input.
//!
//! Three concrete tokenization schemes are exported from sub-modules:
//! [bytes], [graphemes] and [words].
//!
//! [bytes]: bytes/index.html
//! [graphemes]: graphemes/index.html
//! [`pack_all`]: fn.pack_all.html
//! [`Token`]: trait.Token.html
//! [`TokenIter`]: trait.TokenIter.html
//! [`TokenPacker`]: trait.TokenPacker.html
//! [`unpack_all`]: fn.unpack_all.html
//! [words]: words/index.html

use anyhow::Result;
use log::trace;
use std::convert::TryFrom;
use std::fmt::Display;
use std::io::Cursor;

pub mod bytes;
pub mod graphemes;
mod string_parts;
pub mod test_utils;
pub mod words;

/// A single item in the tokenized stream from a string input.
///
/// Tokens may be used as keys in a [`HashMap`](std::collections::HashMap).
pub trait Token: Clone + std::fmt::Debug + Display + Eq + std::hash::Hash {
    // The number of bits of source text contained in this Token.
    fn bit_count(&self) -> usize;
}

/// Provides a method to create a [`Token`] stream from text.
///
/// Errors in reading tokens may be reported upfront or in-stream.
/// All token implementations return a type that implements this treat from the
/// associated `unpack` function.
pub trait TokenIter<R>: std::iter::Iterator<Item = Result<<Self as TokenIter<R>>::T>>
where
    R: std::io::Read,
{
    type T: Token;

    fn unpack(r: R) -> Result<Self>
    where
        Self: Sized;
}

/// Provides a method to pack a [`Token`] stream to text.
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
/// See [`unpack_all()`] for the reverse operation.
pub fn pack_all<W, T, TP>(tokens: Vec<T>, w: &mut W) -> Result<()>
where
    W: std::io::Write,
    T: Token,
    TP: TokenPacker<W, T = T>,
{
    // FIXME: This assumes that tokens are at least a byte wide.
    let size = tokens.iter().fold(0, |sum, t| sum + t.bit_count()) / 8;
    w.write_all(&pack_u64(size as u64))?;
    trace!("wrote size {} as {:?}", size, pack_u64(size as u64));
    TP::pack(tokens.into_iter(), w)?;
    Ok(())
}

/// Unpacks a vector of tokens previously packed with [`pack_all()`].
pub fn unpack_all<R, T, TI>(mut r: &mut R) -> Result<Vec<T>>
where
    R: std::io::Read,
    T: Token,
    TI: TokenIter<Cursor<Vec<u8>>, T = T>,
{
    let size = unpack_u64(&mut r)?;
    trace!("unpacked size {}", size);
    let safe_size = usize::try_from(size)?;
    let mut buf = vec![0u8; safe_size];
    r.read_exact(&mut buf)?;
    trace!("read {} bytes to unpack into tokens", buf.len());
    TI::unpack(Cursor::new(buf)).unwrap().collect()
}

// TODO: dedup with code::common::pack_u64()
fn pack_u64(s: u64) -> Vec<u8> {
    s.to_be_bytes().to_vec()
}

// TODO: dedup with code::common::unpack_u64()
fn unpack_u64<R: std::io::Read>(mut r: R) -> Result<u64> {
    let mut buf: [u8; 8] = [0; 8];
    r.read_exact(&mut buf)?;
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
        let got = unpack_all::<_, _, ByteIter<_>>(&mut Cursor::new(&mut buf)).unwrap();
        assert_eq!(got, tokens);
    }

    #[test]
    fn non_empty() {
        let tokens = vec![
            bytes::Byte::from(0),
            bytes::Byte::from(1),
            bytes::Byte::from(2),
            bytes::Byte::from(3),
            bytes::Byte::from(0),
            bytes::Byte::from(4),
            bytes::Byte::from(5),
            bytes::Byte::from(0),
            bytes::Byte::from(1),
            bytes::Byte::from(0),
        ];
        let mut buf = Vec::<u8>::new();
        assert!(pack_all::<_, _, BytePacker>(tokens.clone(), &mut buf).is_ok());
        let got = unpack_all::<_, _, ByteIter<_>>(&mut Cursor::new(&mut buf)).unwrap();
        assert_eq!(got, tokens);
    }

    #[test]
    fn trailing_byte_data() {
        let tokens = vec![
            bytes::Byte::from(0),
            bytes::Byte::from(1),
            bytes::Byte::from(2),
        ];
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
