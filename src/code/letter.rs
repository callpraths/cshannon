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

/// A Letter represents an indivisible code point.
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
    /// Create a new Letter with given data and number of bits.
    ///
    /// Trailing bits in data (beyond bit_count) are ignored.
    pub fn new(data: &[u8], bit_count: u64) -> Self {
        let num_bytes = (bit_count + 7) as usize / 8;
        let mut l = Self {
            data: data[0..num_bytes].to_vec(),
            bit_count,
        };
        if bit_count % 8 == 0 {
            return l;
        }

        let mut mask: u8 = 0;
        for i in 0..bit_count % 8 {
            mask |= BIT_HOLE_MASKS[i as usize];
        }
        l.data[num_bytes - 1] &= mask;
        l
    }

    pub fn from_bytes(bytes: &[u8]) -> Self {
        Self::new(bytes, 8 * bytes.len() as u64)
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
}

/// Provides deeper access for sibling modules than the public API.
pub trait Peephole {
    fn data<'a>(&'a self) -> &'a Vec<u8>;
    fn bit_count(&self) -> u64;
    fn pack<W: std::io::Write>(self, w: &mut W) -> Result<()>;
    fn unpack<R: std::io::Read>(r: R) -> Result<Self>
    where
        Self: Sized;
}

impl Peephole for Letter {
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
}
