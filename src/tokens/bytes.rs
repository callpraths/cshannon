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

//! bytes module implements tokenization of a string into bytes.
//!
//! The stream makes zero copies internally while iterating over the stream.

use crate::tokens::{Token, TokenIter, TokenPacker};
use anyhow::{Error, Result};
use std::fmt;
use std::hash::Hash;

/// A `Token` consisting of a single byte of data.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Byte(u8);

/// An iterator for `Byte`s read from a `Read`er.
pub struct ByteIter<R: std::io::Read>(R);

/// A `TokenPacker` for packing `Byte`s.
pub struct BytePacker();

/// Used for tests in tokens module, not re-exported.
pub fn new(v: u8) -> Byte {
    Byte(v)
}

impl std::fmt::Display for Byte {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl Token for Byte {
    fn bit_count(&self) -> usize {
        8
    }
}

impl<'b, R: std::io::Read> TokenIter<R> for ByteIter<R> {
    type T = Byte;

    fn unpack(r: R) -> Self {
        Self(r)
    }
}

impl<R: std::io::Read> std::iter::Iterator for ByteIter<R> {
    type Item = Result<Byte>;
    fn next(&mut self) -> Option<Self::Item> {
        let mut buf: [u8; 1] = [0; 1];
        match self.0.read(&mut buf[..]) {
            Err(e) => Some(Err(Error::new(e))),
            Ok(0) => None,
            Ok(1) => Some(Ok(Byte(buf[0]))),
            Ok(l) => panic!("read {} bytes in 1 byte buffer", l),
        }
    }
}

impl<W: std::io::Write> TokenPacker<W> for BytePacker
where
    W: std::io::Write,
{
    type T = Byte;

    fn pack<I>(i: I, w: &mut W) -> Result<()>
    where
        I: std::iter::Iterator<Item = Self::T>,
    {
        let mut buf: [u8; 1] = [0; 1];
        for b in i {
            buf[0] = b.0;
            if let Err(e) = w.write_all(&buf[..]) {
                return Err(Error::new(e));
            }
        }
        w.flush()?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    const TEXT: &str = "
Ah! well a-day! what evil looks
Had I from old and young!
Instead of the cross, the Albatross
About my neck was hung.
";
    #[test]
    fn roundtrip() {
        let mut r = Cursor::new(TEXT);
        let d = ByteIter::unpack(&mut r);
        let i = d.map(|t| t.unwrap());
        let mut wc: Cursor<Vec<u8>> = Cursor::new(vec![]);
        BytePacker::pack(i, &mut wc).unwrap();
        let got = std::str::from_utf8(&wc.get_ref()[..]).unwrap();
        assert_eq!(got, TEXT);
    }
}
