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

//! Create a new balanced tree [`Encoding`].
//!
//! All letters in this encoding are fixed width bit strings. The smallest width
//! necessary to generate the required number of [`Letter`]s is used.
//!
//! The generated [`Encoding`] is stable: calling `new` on a [`Model`]
//! repeatedly yields the same [`Encoding`].

use super::Encoding;
use crate::code::Letter;
use crate::model::Model;
use crate::tokens::Token;
use anyhow::{anyhow, Result};
use std::collections::HashMap;

/// Create a new balanced tree encoding.
///
/// See [package documentation] for details.
///
/// [package documentation]: index.html
pub fn new<T>(m: Model<T>) -> Result<Encoding<T>>
where
    T: Token,
{
    let mut map = HashMap::new();
    let mut letter_generator = LetterGenerator::new(log2(m.len() as u64))?;
    for t in m.tokens_sorted() {
        match letter_generator.next() {
            // Programming error, since bit_count should guarantee we never run
            // out of letters.
            None => panic!("Ran out of letters".to_owned()),
            Some(l) => {
                map.insert(t, l);
            }
        }
    }
    Ok(Encoding::new(map)?)
}

struct LetterGenerator {
    bit_count: u64,
    current: u64,
    max: u64,
}

impl LetterGenerator {
    pub fn new(bit_count: u64) -> Result<Self> {
        if bit_count > 64 {
            return Err(anyhow!("model has too many keys"));
        }
        Ok(Self {
            bit_count,
            // Do not use the code all-0s as it is indistinguishable from
            // trailing 0s when decompressing text.
            current: 1,
            max: 1 << bit_count,
        })
    }
}

impl Iterator for LetterGenerator {
    type Item = Letter;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current >= self.max {
            return None;
        }
        let l = Letter::new(
            &(self.current << (64 - self.bit_count))
                .to_be_bytes()
                .to_vec(),
            self.bit_count as u64,
        );
        self.current += 1;
        Some(l)
    }
}

// TODO: rename
fn log2(n: u64) -> u64 {
    let max = std::mem::size_of::<u64>() as u64 * 8;
    let zeroes = n.leading_zeros() as u64;
    max - zeroes
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::code::Letter;
    use crate::model;
    use crate::tokens::test_utils::I32Token;
    use std::collections::HashMap;

    #[test]
    fn empty() {
        let m: Model<I32Token> = model::with_frequencies(&[]);
        let t = new(m).unwrap();
        assert!(t.alphabet().is_empty());
        assert!(t.map().is_empty());
    }

    #[test]
    fn one_token() {
        let m = model::with_frequencies(&[(I32Token(1), 2)]);
        let t = new(m).unwrap();
        assert_eq!(t.alphabet().len(), 1);
        let want: HashMap<I32Token, Letter> = [(I32Token(1), Letter::new(&[0b1000_0000], 1))]
            .iter()
            .cloned()
            .collect();
        assert_eq!(t.map(), &want);
    }

    #[test]
    fn max_tokens_for_some_tree_height() {
        let m = model::with_frequencies(&[(I32Token(1), 1), (I32Token(2), 2), (I32Token(3), 3)]);
        let t = new(m).unwrap();
        assert_eq!(t.alphabet().len(), 3);
        let want: HashMap<I32Token, Letter> = [
            (I32Token(3), Letter::new(&[0b0100_0000], 2)),
            (I32Token(2), Letter::new(&[0b1000_0000], 2)),
            (I32Token(1), Letter::new(&[0b1100_0000], 2)),
        ]
        .iter()
        .cloned()
        .collect();
        assert_eq!(t.map(), &want);
    }

    #[test]
    fn max_tokens_for_some_tree_height_then_one() {
        let m = model::with_frequencies(&[
            (I32Token(1), 1),
            (I32Token(2), 2),
            (I32Token(3), 3),
            (I32Token(4), 4),
        ]);
        let t = new(m).unwrap();
        assert_eq!(t.alphabet().len(), 4);
        let want: HashMap<I32Token, Letter> = [
            (I32Token(4), Letter::new(&[0b0010_0000], 3)),
            (I32Token(3), Letter::new(&[0b0100_0000], 3)),
            (I32Token(2), Letter::new(&[0b0110_0000], 3)),
            (I32Token(1), Letter::new(&[0b1000_0000], 3)),
        ]
        .iter()
        .cloned()
        .collect();
        assert_eq!(t.map(), &want);
    }
}
