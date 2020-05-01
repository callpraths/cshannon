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

use super::alphabet::{Alphabet, Node, Peephole as aPeephole};
use super::common::BIT_HOLE_MASKS;
use super::letter::{Letter, Peephole as lPeephole};
use anyhow::{anyhow, Error, Result};
use log::trace;
use std::u64;

/// Write a packed stream of letters.
///
/// Returns the number of bytes written.
pub fn pack<'a, I, W>(letters: I, w: &mut W) -> Result<usize>
where
    I: Iterator<Item = &'a Letter>,
    W: std::io::Write,
{
    let mut bytes_written: usize = 0;
    let mut byte_buffer_len: u64 = 0; // In practice <= 7
    let mut byte_buffer: u8 = 0;
    for l in letters {
        trace!("pack: |{}|", &l);
        let mut has_more_bytes = true;
        let mut remaining_bit_count = l.bit_count();
        for b in l.data().iter() {
            assert!(has_more_bytes);
            byte_buffer |= b >> byte_buffer_len;
            if byte_buffer_len + remaining_bit_count >= 8 {
                bytes_written += w.write(&[byte_buffer])?;

                byte_buffer = safe_overflow_leftshift(*b, 8 - byte_buffer_len);
                if remaining_bit_count > 8 {
                    remaining_bit_count -= 8;
                } else {
                    byte_buffer_len = (byte_buffer_len + remaining_bit_count) % 8;
                    remaining_bit_count = 0;
                }
            } else {
                byte_buffer_len += remaining_bit_count;
                has_more_bytes = false;
            }
        }
    }
    if byte_buffer_len > 0 {
        bytes_written += w.write(&[byte_buffer])?;
    }
    w.flush()?;
    Ok(bytes_written)
}

/// Read previously pack()ed text given the corresponding Alphabet.
pub fn parse<'a, R>(
    a: &'a Alphabet,
    r: R,
) -> Result<impl std::iter::Iterator<Item = Result<&'a Letter>>>
where
    R: std::io::Read,
{
    Ok(TextParser::new(a.tree()?, r))
}

struct TextParser<'a, R>
where
    R: std::io::Read,
{
    root: Node<'a>,
    state: TextParserState<R>,
}

struct TextParserState<R>
where
    R: std::io::Read,
{
    r: std::io::Bytes<R>,
    current_byte: u8,
    current_bit_offset: usize,
    eof: bool,
}

impl<'a, R> TextParser<'a, R>
where
    R: std::io::Read,
{
    pub fn new(root: Node<'a>, r: R) -> Self {
        TextParser {
            root: root,
            state: TextParserState {
                r: r.bytes(),
                current_byte: 0,
                current_bit_offset: 8,
                eof: false,
            },
        }
    }
}

impl<'a, R> std::iter::Iterator for TextParser<'a, R>
where
    R: std::io::Read,
{
    type Item = Result<&'a Letter>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.state.eof {
            return None;
        }
        Self::parse_one(&mut self.state, &self.root, true)
    }
}

impl<'a, R> TextParser<'a, R>
where
    R: std::io::Read,
{
    fn parse_one<'b>(
        state: &'b mut TextParserState<R>,
        node: &'b Node<'a>,
        trivial_tail: bool,
    ) -> Option<Result<&'a Letter>> {
        if let Node::Leaf { letter } = node {
            trace!("parse: |{}|", &letter);
            return Some(Ok(letter));
        }

        let b = match state.next_bit() {
            None => {
                state.eof = true;
                if trivial_tail {
                    return None;
                } else {
                    return Some(Err(anyhow!("trailing data")));
                }
            }
            Some(Err(e)) => {
                state.eof = true;
                return Some(Err(e));
            }
            Some(Ok(b)) => b,
        };

        let next = match node {
            Node::Internal { zero, one } => {
                if b {
                    one
                } else {
                    zero
                }
            }
            _ => panic!("neither an internal node nor leaf"),
        };

        match next {
            None => {
                state.eof = true;
                if trivial_tail {
                    loop {
                        match state.next_bit() {
                            None => {
                                return None;
                            }
                            Some(Err(e)) => {
                                return Some(Err(e));
                            }
                            Some(Ok(true)) => {
                                // Found a non-trivial bit in the extended trail.
                                break;
                            }
                            Some(Ok(false)) => {}
                        }
                    }
                }
                return Some(Err(anyhow!("trailing data")));
            }
            Some(next) => {
                return Self::parse_one(state, &*next, trivial_tail & !b);
            }
        }
    }
}

impl<R> TextParserState<R>
where
    R: std::io::Read,
{
    fn next_bit(&mut self) -> Option<Result<bool>> {
        if self.current_bit_offset == 8 {
            match self.r.next() {
                None => return None,
                Some(Err(e)) => return Some(Err(Error::new(e))),
                Some(Ok(b)) => {
                    self.current_byte = b;
                }
            }
            self.current_bit_offset = 0;
        }
        let b = Some(Ok(self.current_byte
            & BIT_HOLE_MASKS[self.current_bit_offset]
            > 0));
        self.current_bit_offset += 1;
        b
    }
}

const RIGHT_MASK_7: u8 = 0b1;
const RIGHT_MASK_6: u8 = 0b11;
const RIGHT_MASK_5: u8 = 0b111;
const RIGHT_MASK_4: u8 = 0b1111;
const RIGHT_MASK_3: u8 = 0b11111;
const RIGHT_MASK_2: u8 = 0b111111;
const RIGHT_MASK_1: u8 = 0b1111111;

// TODO: Generate with macros.
fn safe_overflow_leftshift(b: u8, s: u64) -> u8 {
    assert!(s <= 8);
    if s == 8 {
        return 0;
    }
    let masked = match s {
        0 => b,
        1 => b & RIGHT_MASK_1,
        2 => b & RIGHT_MASK_2,
        3 => b & RIGHT_MASK_3,
        4 => b & RIGHT_MASK_4,
        5 => b & RIGHT_MASK_5,
        6 => b & RIGHT_MASK_6,
        7 => b & RIGHT_MASK_7,
        _ => panic!("No mask for {}", s),
    };
    masked << s
}

#[cfg(test)]
mod pack_tests {
    use super::*;

    #[test]
    fn empty() {
        let letters: Vec<Letter> = vec![];
        let mut got = Vec::new();
        assert_eq!(pack(letters.iter(), &mut got).unwrap(), 0);
        assert_eq!(got, Vec::new());
    }

    #[test]
    fn zero_letter() {
        let letters: Vec<Letter> = vec![Letter::from_bytes(&[])];
        let mut got = Vec::new();
        assert_eq!(pack(letters.iter(), &mut got).unwrap(), 0);
        assert_eq!(got, Vec::new());
    }
    #[test]
    fn single_byte() {
        let letters = vec![Letter::from_bytes(&[0x11])];
        let mut got = Vec::new();
        assert_eq!(pack(letters.iter(), &mut got).unwrap(), 1);
        assert_eq!(got, [0x11].to_vec());
    }

    #[test]
    fn single_aligned_letter() {
        let letters = vec![Letter::from_bytes(&[
            0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, 0x88, 0x99, 0xaa,
        ])];
        let mut got = Vec::new();
        let want: Vec<u8> = vec![0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, 0x88, 0x99, 0xaa];
        assert_eq!(pack(letters.iter(), &mut got).unwrap(), want.len());
        assert_eq!(got, want);
    }

    #[test]
    fn single_unaligned_short_letter() {
        let letters = vec![Letter::new(&[0b1101_1000], 5)];
        let mut got = Vec::new();
        let want: Vec<u8> = vec![0b1101_1000];
        assert_eq!(pack(letters.iter(), &mut got).unwrap(), want.len());
        assert_eq!(got, want);
    }

    #[test]
    fn single_unaligned_long_letter() {
        let letters = vec![Letter::new(&[0b11011000, 0b11100000], 13)];
        let mut got = Vec::new();
        let want: Vec<u8> = vec![0b1101_1000, 0b1110_0000];
        assert_eq!(pack(letters.iter(), &mut got).unwrap(), want.len());
        assert_eq!(got, want);
    }

    #[test]
    fn multiple_aligned_letters() {
        let letters = vec![
            Letter::from_bytes(&[0x11, 0x22]),
            Letter::from_bytes(&[0x33, 0x44, 0x55]),
            Letter::from_bytes(&[0x66, 0x11]),
        ];
        let mut got = Vec::new();
        let want: Vec<u8> = vec![0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x11];
        assert_eq!(pack(letters.iter(), &mut got).unwrap(), want.len());
        assert_eq!(got, want);
    }

    #[test]
    fn multiple_unaligned_letters() {
        let letters = vec![
            Letter::new(&[0b1101_1000, 0b1000_0000], 9),
            Letter::new(&[0b1101_0000], 4),
            Letter::from_bytes(&[0b1101_1101]),
        ];
        let mut got = Vec::new();
        let want: Vec<u8> = vec![0b1101_1000, 0b1110_1110, 0b1110_1000];
        assert_eq!(pack(letters.iter(), &mut got).unwrap(), want.len());
        assert_eq!(got, want);
    }

    #[test]
    fn multiple_unaligned_short_letters() {
        let letters = vec![
            Letter::new(&[0b1100_0000], 3),
            Letter::new(&[0b1101_0000], 6),
            Letter::new(&[0b1100_0000], 3),
        ];
        let mut got = Vec::new();
        let want: Vec<u8> = vec![0b110__1101_0, 0b0__110__0000];
        assert_eq!(pack(letters.iter(), &mut got).unwrap(), want.len());
        assert_eq!(got, want);
    }
}

#[cfg(test)]
mod parse_tests {
    use super::*;

    use std::io::Cursor;

    #[test]
    fn empty() {
        let a = Alphabet::new(vec![Letter::from_bytes(&[0xff])]);
        let t: Vec<u8> = vec![];
        let r: Result<Vec<&Letter>> = parse(&a, Cursor::new(t)).unwrap().collect();
        let c = r.unwrap();
        assert_eq!(c.len(), 0);
    }

    #[test]
    fn single_byte_disjoint() {
        let l0 = Letter::from_bytes(&[0x00]);
        let l1 = Letter::from_bytes(&[0x11]);
        let l2 = Letter::from_bytes(&[0x22]);
        let l3 = Letter::from_bytes(&[0x33]);

        let a = Alphabet::new(vec![l0.clone(), l1.clone(), l2.clone(), l3.clone()]);
        let t: Vec<u8> = vec![0x00, 0x22, 0x33, 0x22, 0x33, 0x00];
        let r: Result<Vec<&Letter>> = parse(&a, Cursor::new(t)).unwrap().collect();
        let c = r.unwrap();
        assert_eq!(c, vec![&l0, &l2, &l3, &l2, &l3, &l0]);
    }

    #[test]
    fn single_byte_common_prefix() {
        let l0 = Letter::from_bytes(&[0b0000_0000]);
        let l1 = Letter::from_bytes(&[0b0000_0010]);
        let l2 = Letter::from_bytes(&[0b0010_1111]);
        let l3 = Letter::from_bytes(&[0b0011_0000]);

        let a = Alphabet::new(vec![l0.clone(), l1.clone(), l2.clone(), l3.clone()]);
        let t: Vec<u8> = vec![
            0b0000_0000,
            0b0010_1111,
            0b0011_0000,
            0b0010_1111,
            0b0011_0000,
            0b0000_0000,
        ];
        let r: Result<Vec<&Letter>> = parse(&a, Cursor::new(t)).unwrap().collect();
        let c = r.unwrap();
        assert_eq!(c, vec![&l0, &l2, &l3, &l2, &l3, &l0]);
    }

    #[test]
    fn multi_byte_common_prefix() {
        let l0 = Letter::from_bytes(&[0x00, 0x11]);
        let l1 = Letter::from_bytes(&[0x00, 0x10]);
        let l2 = Letter::from_bytes(&[0x00, 0x01]);
        let l3 = Letter::from_bytes(&[0x11]);

        let a = Alphabet::new(vec![l0.clone(), l1.clone(), l2.clone(), l3.clone()]);
        let t: Vec<u8> = vec![0x00, 0x11, 0x00, 0x01, 0x11, 0x00, 0x01, 0x11, 0x00, 0x11];
        let r: Result<Vec<&Letter>> = parse(&a, Cursor::new(t)).unwrap().collect();
        let c = r.unwrap();
        assert_eq!(c, vec![&l0, &l2, &l3, &l2, &l3, &l0]);
    }

    #[test]
    fn short_unaligned_fit() {
        let l0 = Letter::new(&[0b1000_0000], 3);
        let l1 = Letter::new(&[0b0100_0000], 2);

        let a = Alphabet::new(vec![l0.clone(), l1.clone()]);
        let t: Vec<u8> = vec![0b100_01_100, 0b01_100_100];
        let r: Result<Vec<&Letter>> = parse(&a, Cursor::new(t)).unwrap().collect();
        let c = r.unwrap();
        assert_eq!(c, vec![&l0, &l1, &l0, &l1, &l0, &l0]);
    }

    #[test]
    fn short_unaligned_trailing_zeros() {
        let l0 = Letter::new(&[0b1000_0000], 3);
        let l1 = Letter::new(&[0b0100_0000], 2);

        let a = Alphabet::new(vec![l0.clone(), l1.clone()]);
        let t: Vec<u8> = vec![0b100_01_100, 0b01_01_0000];
        let r: Result<Vec<&Letter>> = parse(&a, Cursor::new(t)).unwrap().collect();
        let c = r.unwrap();
        assert_eq!(c, vec![&l0, &l1, &l0, &l1, &l1]);
    }

    #[test]
    fn complex_long_unaligned_shared_trailing_zeros() {
        let l0 = Letter::new(&[0b1000_0001, 0b1100_0000], 10);
        let l1 = Letter::new(&[0b1000_0001, 0b1000_0000], 13);

        let a = Alphabet::new(vec![l0.clone(), l1.clone()]);
        let t: Vec<u8> = vec![
            0b1000_0001,
            0b11__1000_00,
            0b01_1000_0__1,
            0b000_0001_1,
            0b000_0__1000,
            0b0001_11__00,
            0b0000_0000,
            0b0000_0000,
        ];
        let r: Result<Vec<&Letter>> = parse(&a, Cursor::new(t)).unwrap().collect();
        let c = r.unwrap();
        assert_eq!(c, vec![&l0, &l1, &l1, &l0]);
    }

    #[test]
    fn incomplete() {
        let l0 = Letter::from_bytes(&[0x11, 0x00]);

        let a = Alphabet::new(vec![l0.clone()]);
        let t: Vec<u8> = vec![0x11];
        let r: Result<Vec<&Letter>> = parse(&a, Cursor::new(t)).unwrap().collect();
        assert!(r.is_err());
    }

    #[test]
    fn nonexistent_letter() {
        let l0 = Letter::from_bytes(&[0x11, 0x00]);

        let a = Alphabet::new(vec![l0.clone()]);
        let t: Vec<u8> = vec![0x10, 0x00];
        let r: Result<Vec<&Letter>> = parse(&a, Cursor::new(t)).unwrap().collect();
        assert!(r.is_err());
    }

    #[test]
    fn trailing_data() {
        let l0 = Letter::from_bytes(&[0x11]);

        let a = Alphabet::new(vec![l0.clone()]);
        let t: Vec<u8> = vec![0x11, 0x11, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01];
        let r: Result<Vec<&Letter>> = parse(&a, Cursor::new(t)).unwrap().collect();
        assert!(r.is_err());
    }
}
