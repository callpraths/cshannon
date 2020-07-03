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

use super::common::{pack_u64, unpack_u64, BIT_HOLE_MASKS};
use anyhow::{anyhow, Result};
use log::trace;
use std::convert::TryInto;
use std::fmt;

/// An indivisible code point with the [prefix property].
///
/// [prefix property]: https://en.wikipedia.org/wiki/Prefix_code
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Letter {
    data: Vec<u8>,
    // TODO: Store as usize
    bit_count: u64,
}

impl fmt::Display for Letter {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for i in 0..(self.bit_count as usize) {
            if self.at(i).unwrap() {
                write!(f, "1")?;
            } else {
                write!(f, "0")?;
            }
        }
        Ok(())
    }
}

impl Letter {
    /// Create a new letter with given data and number of bits.
    ///
    /// Trailing bits in data (beyond `bit_count`) are ignored.
    pub fn new(data: &[u8], bit_count: u64) -> Self {
        let num_bytes = (bit_count + 7) as usize / 8;
        let mut l = Self {
            data: data[0..num_bytes].to_vec(),
            bit_count,
        };
        l.clear_trailing_bits();
        l
    }

    fn clear_trailing_bits(&mut self) {
        if self.bit_count % 8 == 0 {
            return;
        }
        let mut mask: u8 = 0;
        for i in 0..self.bit_count % 8 {
            mask |= BIT_HOLE_MASKS[i as usize];
        }
        let last = self.data.len() - 1;
        self.data[last] &= mask;
    }

    /// Create a new letter with 0 bits.
    ///
    /// Useful for incremental construction using `self.push0()` and
    /// `self.push1()`.
    pub fn empty() -> Self {
        Letter::with_capacity(0)
    }

    /// Create a new letter with 0 bits but with capacity for `hint` bits.
    ///
    /// Useful for incremental construction using `self.push0()` and
    /// `self.push1()`.
    pub fn with_capacity(capacity: u64) -> Self {
        Self {
            data: Vec::with_capacity(((capacity + 7) / 8) as usize),
            bit_count: 0,
        }
    }

    /// Create a new letter with the given data.
    ///
    /// The created letter has `bit_count` of `8 * len(bytes)`.
    pub fn from_bytes(bytes: &[u8]) -> Self {
        Self::new(bytes, 8 * bytes.len() as u64)
    }

    /// Extend this Letter with a `0` bit.
    pub fn push0(&mut self) {
        self.push(false);
    }

    /// Extend this Letter with a `1` bit.
    pub fn push1(&mut self) {
        self.push(true)
    }

    /// Return whether the bit at given index is set.
    ///
    /// Returns an error if the index is out of bounds.
    pub fn at(&self, i: usize) -> Result<bool> {
        if i as u64 >= self.bit_count {
            return Err(anyhow!(
                "index {} out of bounds of letter sized {}",
                i,
                self.bit_count,
            ));
        }
        let b = i / 8;
        let o = i % 8;
        Ok(self.data[b] & BIT_HOLE_MASKS[o] > 0)
    }

    fn push(&mut self, bit: bool) {
        if self.bit_count == 8 * self.data.len() as u64 {
            self.data.push(0u8);
        }
        self.bit_count += 1;
        if bit {
            self.set1((self.bit_count - 1) as usize);
        }
    }

    fn set1(&mut self, i: usize) {
        if i as u64 >= self.bit_count {
            // set1() is not a public function.
            // Out-of-bounds here is programming error.
            panic!(
                "index {} out of bounds of letter sized {}",
                i, self.bit_count,
            );
        }
        let b = i / 8;
        let o = i % 8;
        self.data[b] |= BIT_HOLE_MASKS[o];
    }
}

/// Provides deeper access for sibling modules than the public API.
pub trait Peephole {
    fn validate(&self) -> Result<()>;
    fn data<'a>(&'a self) -> &'a Vec<u8>;
    fn bit_count(&self) -> u64;
    fn pack<W: std::io::Write>(self, w: &mut W) -> Result<()>;
    fn unpack<R: std::io::Read>(r: R) -> Result<Self>
    where
        Self: Sized;
}

impl Peephole for Letter {
    fn validate(&self) -> Result<()> {
        if self.all_zeroes() {
            Err(anyhow!("letter {} is all 0s", self))
        } else {
            Ok(())
        }
    }

    fn data<'a>(&'a self) -> &'a Vec<u8> {
        &self.data
    }

    fn bit_count(&self) -> u64 {
        self.bit_count
    }

    fn pack<W: std::io::Write>(self, w: &mut W) -> Result<()> {
        trace!("pack: |{}|", &self);
        w.write_all(&pack_u64(self.bit_count))?;
        w.write_all(&self.data)?;
        Ok(())
    }

    fn unpack<R: std::io::Read>(mut r: R) -> Result<Self> {
        let bit_count = unpack_u64(&mut r)?;
        let byte_count = (bit_count + 7) / 8;
        let mut data = vec![0u8; byte_count.try_into().unwrap()];
        r.read_exact(&mut data)?;
        let l = Self { bit_count, data };
        trace!("unpack: |{}|", &l);
        Ok(l)
    }
}

impl Letter {
    fn all_zeroes(&self) -> bool {
        for i in 0..self.bit_count {
            if self.at(i as usize).unwrap() {
                return false;
            }
        }
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn at() {
        let l = Letter::new(&[0b0010_0010], 7);
        assert_eq!(l.at(0).unwrap(), false);
        assert_eq!(l.at(1).unwrap(), false);
        assert_eq!(l.at(2).unwrap(), true);
        assert_eq!(l.at(3).unwrap(), false);
        assert_eq!(l.at(4).unwrap(), false);
        assert_eq!(l.at(5).unwrap(), false);
        assert_eq!(l.at(6).unwrap(), true);
        assert!(l.at(7).is_err());
    }

    #[test]
    fn new_data_truncation() {
        assert_eq!(Letter::new(&[0b0111_1100], 3).data(), &vec![0b0110_0000u8]);
        assert_eq!(
            Letter::new(&[0b0000_1111, 0b0111_1100], 11).data(),
            &vec![0b0000_1111u8, 0b0110_0000u8]
        );
        assert_eq!(
            Letter::new(&[0b1000_1111, 0b0111_1100], 3).data(),
            &vec![0b1000_0000u8]
        );
        assert_eq!(Letter::new(&[0b0000_1111], 4).data(), &vec![0b0000_0000u8]);
    }

    #[test]
    fn empty() {
        assert_eq!(Letter::empty(), Letter::new(&[0u8], 0));
    }

    #[test]
    fn with_capacity() {
        assert_eq!(Letter::with_capacity(24), Letter::new(&[0u8], 0));
        assert_eq!(Letter::with_capacity(8).data.capacity(), 1);
    }

    #[test]
    fn incremental_build() {
        let mut l = Letter::empty();
        l.push0();
        l.push1();
        l.push0();
        l.push1();
        assert_eq!(l, Letter::new(&[0b0101_0000], 4));
    }

    #[test]
    fn incremental_build_extends_data() {
        let mut l = Letter::with_capacity(16);
        for _ in 0..11 {
            l.push1();
        }
        assert_eq!(l, Letter::new(&[0xFF, 0b1110_0000], 11));
    }
}
