use super::common::{pack_u64, unpack_u64, BIT_HOLE_MASKS};
use super::types::Result;

use std::convert::TryInto;
use std::fmt;

/// A Letter represents an indivisible code point.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Letter {
    data: Vec<u8>,
    // TODO: Store as usize
    bit_count: u64,
}

/// Provides deeper access for sibling modules than the public API.
pub trait Peephole {
    fn data<'a>(&'a self) -> &'a Vec<u8>;
    fn bit_count(&self) -> u64;
}

impl Peephole for Letter {
    fn data<'a>(&'a self) -> &'a Vec<u8> {
        &self.data
    }

    fn bit_count(&self) -> u64 {
        self.bit_count
    }
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
    pub fn new(data: &[u8], bit_count: u64) -> Self {
        Self {
            data: data.to_vec(),
            bit_count: bit_count,
        }
    }
    pub fn from_bytes(bytes: &[u8]) -> Self {
        Self::new(bytes, 8 * bytes.len() as u64)
    }

    /// Return whether the bit at given index is set.
    ///
    /// Returns an error if the index is out of bounds.
    pub fn at(&self, i: usize) -> Result<bool> {
        if i as u64 >= self.bit_count {
            return Err(format!(
                "index {} out of bounds of letter sized {}",
                i, self.bit_count,
            ));
        }
        let b = i / 8;
        let o = i % 8;
        Ok(self.data[b] & BIT_HOLE_MASKS[o] > 0)
    }

    pub fn pack(mut self) -> Vec<u8> {
        let mut p = Vec::new();
        p.append(&mut pack_u64(self.bit_count));
        p.append(&mut self.data);
        p
    }

    pub fn unpack(iter: &mut std::vec::IntoIter<u8>) -> core::result::Result<Self, String> {
        let bit_count = unpack_u64(iter)?;
        let data = Letter::unpack_data(iter, bit_count)?;
        Ok(Self {
            bit_count: bit_count,
            data: data,
        })
    }

    fn unpack_data(
        iter: &mut std::vec::IntoIter<u8>,
        bit_count: u64,
    ) -> core::result::Result<Vec<u8>, String> {
        let byte_count = (bit_count + 7) / 8;
        let mut data = Vec::with_capacity(byte_count.try_into().unwrap());
        for _ in 0..byte_count {
            match iter.next() {
                Some(d) => {
                    let dd = d;
                    data.push(dd);
                }
                None => return Err("too few elements".to_owned()),
            }
        }
        Ok(data)
    }
}

#[cfg(test)]
mod letter_tests {
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
}
