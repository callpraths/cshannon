use super::{Encoding, Result};
use crate::code::Letter;
use crate::model::Model;
use crate::tokens::Token;
use std::collections::HashMap;

/// Create a new balanced tree `Encoding`.
///
/// All letters in this encoding are fixed width bit strings. The smallest width
/// necessary to generate the required number of `Letter`s is used.
///
/// The generated `Encoding` is stable: calling `new()` on a `Model` repeatedly
/// will yield the same `Encoding`.
pub fn new<T>(m: Model<T>) -> Result<Encoding<T>>
where
    T: Token,
{
    let mut encoding = Encoding::<T>(HashMap::new());
    let mut letter_generator = LetterGenerator::new(log2(m.len() as u64))?;
    for t in m.tokens_sorted() {
        match letter_generator.next() {
            // Programming error, since bit_count should guarantee we never run
            // out of letters.
            None => panic!("Ran out of letters".to_owned()),
            Some(l) => {
                encoding.0.insert(t, l);
            }
        }
    }
    Ok(encoding)
}

struct LetterGenerator {
    bit_count: u64,
    current: u64,
    max: u64,
}

impl LetterGenerator {
    pub fn new(bit_count: u64) -> Result<Self> {
        if bit_count > 64 {
            return Err("model has too many keys".to_owned());
        }
        Ok(Self {
            bit_count,
            current: 0,
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
fn log2(mut n: u64) -> u64 {
    if n > 0 {
        n -= 1;
    }
    let max = std::mem::size_of::<u64>() as u64 * 8;
    let zeroes = n.leading_zeros() as u64;
    let bit_count = max - zeroes;
    if bit_count == 0 {
        1
    } else {
        bit_count
    }
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
        let m = model::with_frequencies(HashMap::<I32Token, i64>::new());
        let t = new(m).unwrap();
        assert!(t.alphabet().is_empty());
        assert!(t.map().is_empty());
    }

    #[test]
    fn one_token() {
        let m = model::with_frequencies([(I32Token(1), 2)].iter().cloned().collect());
        let t = new(m).unwrap();
        assert_eq!(t.alphabet().len(), 1);
        let want: HashMap<I32Token, Letter> = [(I32Token(1), Letter::new(&[0x00], 1))]
            .iter()
            .cloned()
            .collect();
        assert_eq!(t.map(), &want);
    }

    #[test]
    fn four_tokens() {
        let m = model::with_frequencies(
            [
                (I32Token(1), 1),
                (I32Token(2), 2),
                (I32Token(3), 3),
                (I32Token(4), 4),
            ]
            .iter()
            .cloned()
            .collect(),
        );
        let t = new(m).unwrap();
        assert_eq!(t.alphabet().len(), 4);
        let want: HashMap<I32Token, Letter> = [
            (I32Token(4), Letter::new(&[0b0000_0000], 2)),
            (I32Token(3), Letter::new(&[0b0100_0000], 2)),
            (I32Token(2), Letter::new(&[0b1000_0000], 2)),
            (I32Token(1), Letter::new(&[0b1100_0000], 2)),
        ]
        .iter()
        .cloned()
        .collect();
        assert_eq!(t.map(), &want);
    }
}
