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

//! Defines the [`Encoding`] struct that maps an [`Token`] to a
//! [`Letter`].
//!
//! An [`Encoding`] can be generated from a [`Model`](crate::model::Model) by
//! calling the `new()` function defined in one of the sub-modules:
//! [balanced_tree], [shannon], [fano], or [huffman].

use crate::code::{Alphabet, Letter};
use crate::model::Model;
use crate::tokens::{Token, TokenPacker, Tokenizer};
use anyhow::{anyhow, Result};
use log::{debug, log_enabled, Level};
use std::collections::HashMap;
use std::convert::TryFrom;
use std::io::Cursor;

pub mod balanced_tree;
pub mod fano;
pub mod huffman;
pub mod shannon;

/// Encoding schems (i.e. the compression algorithms) supported by this library.
pub enum EncodingScheme {
    /// Create a new balanced tree encoding.
    ///
    /// All letters in this encoding are fixed width bit strings. The smallest
    /// width necessary to generate the required number of letters for all
    /// input tokens is used.
    BalancedTree,
    /// Create a new [Fano encoding].
    ///
    /// [Fano encoding]: https://en.wikipedia.org/wiki/Shannon%E2%80%93Fano_coding
    Fano,
    /// Create a new [Shannon encoding].
    ///
    /// The Shannon encoding scheme is defined thus:
    ///
    /// Let the tokens, sorted in decreasing order of frequency be
    /// `t1, t2, t3 ...`
    ///
    /// Let the probability of occurrence of the Tokens be `f1, f2, f3 ...`
    ///
    /// Define the numbers `l1, l2, l3 ...` such that `lk` = `ceil(log2(1/fk))`
    ///
    /// Let the (computed) cumulative proportions be `c1, c2, c3 ...`
    ///
    /// Then, the code is `e1, e2, e3 ...`
    /// such that `ek` = first `lk` bits of the binary expansion of `Fk`.
    ///
    /// [Shannon encoding]: https://en.wikipedia.org/wiki/Shannon%E2%80%93Fano_coding
    Shannon,
    /// Create a new [Huffman encoding].
    ///
    /// Of the algorithms implemented in this library, this is closest to practical
    /// compression libraries used widely - e.g., deflate, jpeg and mp3 use a
    /// Huffman-like encoding in their backends.
    ///
    /// [Huffman encoding]: https://en.wikipedia.org/wiki/Huffman_coding
    Huffman,
}

pub fn new_encoder<T: Token>(
    encoding_scheme: &EncodingScheme,
    model: Model<T>,
) -> Result<Encoding<T>> {
    let constructor = match encoding_scheme {
        EncodingScheme::BalancedTree => balanced_tree::new::<T>,
        EncodingScheme::Fano => fano::new::<T>,
        EncodingScheme::Huffman => huffman::new::<T>,
        EncodingScheme::Shannon => shannon::new::<T>,
    };
    constructor(model)
}

/// Maps a [`Token`] to a [`Letter`].
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Encoding<T: Token> {
    map: HashMap<T, Letter>,
    alphabet: Alphabet,
}

impl<T: Token> Encoding<T> {
    /// `Encoding` implementations should use `new` to create an `Encoding`.
    fn new(map: HashMap<T, Letter>) -> Result<Self> {
        let mut letters: Vec<&Letter> = map.values().collect();
        letters.sort();
        let alphabet = Alphabet::new(letters.into_iter().cloned().collect())?;
        log_encoder_ring(&map);
        Ok(Self { map, alphabet })
    }

    /// The [`Alphabet`] used by this encoding.
    pub fn alphabet(&self) -> &Alphabet {
        &self.alphabet
    }

    /// The encoding map.
    ///
    /// Exposes the internal [`HashMap`](std::collections::HashMap) via an
    /// immutable reference.
    pub fn map(&self) -> &HashMap<T, Letter> {
        &self.map
    }

    pub fn reverse_map(&self) -> HashMap<&Letter, &T> {
        let mut m = HashMap::new();
        for (t, l) in &self.map {
            m.insert(l, t);
        }
        m
    }

    pub fn pack<W: std::io::Write>(&self, w: &mut W) -> Result<()> {
        let tokens = self.tokens();
        let size = tokens.iter().fold(0, |sum, t| sum + t.bit_count()) / 8;
        w.write_all(&pack_u64(size as u64))?;
        T::Packer::pack(tokens.into_iter(), w)?;

        self.alphabet().clone().pack(w)?;

        Ok(())
    }

    pub fn unpack<R: std::io::Read>(r: &mut R) -> Result<Self> {
        let size = unpack_u64(r)?;
        let safe_size = usize::try_from(size)?;
        let mut buf = vec![0u8; safe_size];
        r.read_exact(&mut buf)?;
        let tokens: Result<Vec<T>> = T::Tokenizer::tokenize(Cursor::new(buf)).unwrap().collect();
        let tokens = tokens?;

        let alphabet = crate::code::Alphabet::unpack(r)?;
        let letters = alphabet.letters().into_iter().cloned();
        if letters.len() != tokens.len() {
            return Err(anyhow!(
                "Extracted letter count {} does not match token count {}",
                letters.len(),
                tokens.len(),
            ));
        }

        let map: HashMap<T, Letter> = tokens.iter().cloned().zip(letters.into_iter()).collect();
        log_encoder_ring(&map);
        Ok(Self { map, alphabet })
    }

    /// The set of [`Token`]s covered by this encoding.
    ///
    /// The returned set is sorted in a stable order corresponding to the order
    /// of letters in [`Self::alphabet()`]
    fn tokens(&self) -> Vec<T> {
        let m = self.reverse_map();
        let mut letters: Vec<&Letter> = self.map.values().collect();
        letters.sort();
        letters.into_iter().map(|l| m[l].clone()).collect()
    }
}

/// Helper function to create a new `Encoding` from known mapping.
///
/// This is a private function useful for checking expected Encoding in
/// unit tests.
#[allow(dead_code)]
fn from_pairs<T: Token>(data: &[(T, Letter)]) -> Result<Encoding<T>> {
    Encoding::new(data.iter().cloned().collect())
}

fn log_encoder_ring<T: Token>(m: &HashMap<T, Letter>) {
    if !log_enabled!(Level::Debug) {
        return;
    }
    debug!("Encoder ring:");
    for (k, l) in m.iter() {
        debug!("  |{:?}|: |{:?}|", k, l);
    }
}

// TODO: dedup with code::common::pack_u64()
fn pack_u64(s: u64) -> Vec<u8> {
    s.to_be_bytes().to_vec()
}

// TODO: dedup with code::common::unpack_u64()
fn unpack_u64<R: std::io::Read>(r: &mut R) -> Result<u64> {
    let mut buf: [u8; 8] = [0; 8];
    r.read_exact(&mut buf)?;
    Ok(u64::from_be_bytes(buf))
}

#[cfg(test)]
mod roundtrip_with_len_tests {

    use super::*;
    use crate::tokens::{bytes::Byte, graphemes::Grapheme};
    use std::io::{Cursor, Read};
    #[test]

    fn empty() {
        let encoding: Encoding<Byte> = Encoding::new(HashMap::new()).unwrap();
        let mut buf = Vec::<u8>::new();
        assert!(encoding.pack(&mut buf).is_ok());
        let got: Encoding<Byte> = Encoding::unpack(&mut Cursor::new(&mut buf)).unwrap();
        assert_eq!(got.map(), encoding.map());
    }

    #[test]
    fn non_empty() {
        let map = (vec![
            (Byte::from(0), Letter::from_bytes(&vec![0u8, 1u8])),
            (Byte::from(1), Letter::from_bytes(&vec![0u8, 0u8, 1u8])),
            (Byte::from(2), Letter::from_bytes(&vec![0u8, 0u8, 0u8, 1u8])),
            (
                Byte::from(3),
                Letter::from_bytes(&vec![0u8, 0u8, 0u8, 0u8, 1u8]),
            ),
            (Byte::from(0), Letter::from_bytes(&vec![1u8, 1u8])),
            (Byte::from(4), Letter::from_bytes(&vec![1u8, 0u8, 1u8])),
            (Byte::from(5), Letter::from_bytes(&vec![1u8, 0u8, 0u8, 1u8])),
        ])
        .into_iter()
        .collect();
        let encoding: Encoding<Byte> = Encoding::new(map).unwrap();

        let mut buf = Vec::<u8>::new();
        assert!(encoding.pack(&mut buf).is_ok());
        let got: Encoding<Byte> = Encoding::unpack(&mut Cursor::new(&mut buf)).unwrap();
        assert_eq!(got.map(), encoding.map());
    }

    #[test]
    fn trailing_data_byte() {
        let map = (vec![
            (Byte::from(0), Letter::from_bytes(&vec![0u8, 1u8])),
            (Byte::from(1), Letter::from_bytes(&vec![0u8, 0u8, 1u8])),
            (Byte::from(2), Letter::from_bytes(&vec![0u8, 0u8, 0u8, 1u8])),
        ])
        .into_iter()
        .collect();
        let encoding: Encoding<Byte> = Encoding::new(map).unwrap();

        let mut buf = Vec::<u8>::new();
        assert!(encoding.pack(&mut buf).is_ok());

        // The following trailing data should be ignored.
        buf.push(0b1111_1111);

        let mut r = Cursor::new(buf);
        let got: Encoding<Byte> = Encoding::unpack(&mut r).unwrap();
        assert_eq!(got.map(), encoding.map());

        // The buffer should not be read beyond the trailing data.
        let mut buf = Vec::<u8>::new();
        assert_eq!(r.read_to_end(&mut buf).unwrap(), 1);
        assert_eq!(buf, vec![0b1111_1111u8]);
    }

    #[test]
    fn trailing_data_grapheme() {
        let map = (vec![
            (
                Grapheme::from("a".to_owned()),
                Letter::from_bytes(&vec![0u8, 1u8]),
            ),
            (
                Grapheme::from("b".to_owned()),
                Letter::from_bytes(&vec![0u8, 0u8, 1u8]),
            ),
        ])
        .into_iter()
        .collect();
        let encoding: Encoding<Grapheme> = Encoding::new(map).unwrap();

        let mut buf = Vec::<u8>::new();
        assert!(encoding.pack(&mut buf).is_ok());

        // The following trailing data should be ignored.
        buf.push(0b1111_1111);

        let mut r = Cursor::new(buf);
        let got: Encoding<Grapheme> = Encoding::unpack(&mut r).unwrap();
        assert_eq!(got.map(), encoding.map());

        // The buffer should not be read beyond the trailing data.
        let mut buf = Vec::<u8>::new();
        assert_eq!(r.read_to_end(&mut buf).unwrap(), 1);
        assert_eq!(buf, vec![0b1111_1111u8]);
    }
}
