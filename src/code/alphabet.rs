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

use super::common::{pack_u64, unpack_u64};
use super::letter::{Letter, Peephole as lPeephole};
use anyhow::{anyhow, Result};

/// Alphabet is an ordered list of unique Letters.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Alphabet(Vec<Letter>);

impl Alphabet {
    /// Create a new Alphabet with the given [`Letter`]s.
    ///
    /// The order of letters is significant.
    /// [`Self::pack()`] & [`Self::unpack()`] conserve the order.
    pub fn new(letters: Vec<Letter>) -> Result<Self> {
        let a = Alphabet(letters);
        a.validate()?;
        Ok(a)
    }

    /// Number of letters in the alphpabet.
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Check whether this alphabet is empty.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Return a reference to the ordered [`Letter`]s in this alphabet.
    pub fn letters(&self) -> &Vec<Letter> {
        &self.0
    }
}

impl Alphabet {
    fn validate(&self) -> Result<()> {
        for l in self.0.iter() {
            l.validate()?;
        }
        self.tree()?;
        Ok(())
    }
}

/// An alphabet may be generated from an iterator over letters.
///
/// This operation clones each provided letter.
impl<'a> std::iter::FromIterator<&'a Letter> for Alphabet {
    fn from_iter<I: IntoIterator<Item = &'a Letter>>(i: I) -> Self {
        let mut a = Alphabet(Vec::new());
        for l in i {
            a.0.push(l.clone());
        }
        a
    }
}

impl Alphabet {
    /// Serialize the alphabet to a [`Write`er](std::io::Write).
    ///
    /// Can be deserialized back to an alphabet with [`Self::unpack()`].
    pub fn pack<W: std::io::Write>(self, w: &mut W) -> Result<()> {
        let letter_count = self.0.len();
        w.write_all(&pack_u64(letter_count as u64))?;
        for l in self.0.into_iter() {
            l.pack(w)?;
        }
        Ok(())
    }

    /// Deserialize a data generated with [`Self::pack()`] from a
    /// [`Read`er](std::io::Read).
    pub fn unpack<R: std::io::Read>(mut r: R) -> Result<Self> {
        let letter_count = unpack_u64(&mut r)?;
        let mut letters = Vec::new();
        for _ in 0..letter_count {
            let l = Letter::unpack(&mut r)?;
            letters.push(l);
        }
        Ok(Alphabet(letters))
    }
}

/// Provides deeper access for sibling modules than the public API.
pub trait Peephole {
    fn tree<'a>(&'a self) -> Result<Node<'a>>;
}

impl Peephole for Alphabet {
    fn tree<'a>(&'a self) -> Result<Node<'a>> {
        let mut root = Node::Internal {
            zero: None,
            one: None,
        };
        if self.0.is_empty() {
            // Attempt to parse _any_ text will fail, since the Alphabet is
            // empty.
            return Ok(root);
        }
        for l in self.0.iter() {
            let (tip, offset) = Alphabet::follow_branch(&mut root, l, 0)?;
            let tail = Alphabet::tail(l, offset + 1);
            match tip {
                Node::Internal { zero, one } => {
                    if l.at(offset)? {
                        *one = Some(Box::new(tail));
                    } else {
                        *zero = Some(Box::new(tail));
                    }
                }
                Node::Leaf { .. } => panic!("tip can not be a Leaf"),
            }
        }
        Ok(root)
    }
}

impl Alphabet {
    fn follow_branch<'a, 'b>(
        tree: &'b mut Node<'a>,
        l: &'a Letter,
        offset: usize,
    ) -> Result<(&'b mut Node<'a>, usize)> {
        match l.at(offset) {
            Ok(false) => {
                match tree {
                    Node::Internal {
                        zero: Some(zero), ..
                    } => {
                        return Alphabet::follow_branch(zero, l, offset + 1);
                    }
                    // This error message actually needs a slice l[0:offset]
                    Node::Leaf { .. } => return Err(anyhow!("Duplicate prefix {}", l)),
                    _ => return Ok((tree, offset)),
                }
            }
            Ok(true) => {
                match tree {
                    Node::Internal { one: Some(one), .. } => {
                        return Alphabet::follow_branch(one, l, offset + 1);
                    }
                    // This error message actually needs a slice l[0:offset]
                    Node::Leaf { .. } => return Err(anyhow!("Duplicate prefix {}", l)),
                    _ => return Ok((tree, offset)),
                }
            }
            Err(_) => Err(anyhow!("Duplicate prefix {}", l)),
        }
    }

    fn tail<'a>(l: &'a Letter, offset: usize) -> Node<'a> {
        match l.at(offset) {
            Ok(false) => Node::Internal {
                zero: Some(Box::new(Alphabet::tail(l, offset + 1))),
                one: None,
            },
            Ok(true) => Node::Internal {
                zero: None,
                one: Some(Box::new(Alphabet::tail(l, offset + 1))),
            },
            Err(_) => Node::Leaf { letter: l },
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum Node<'a> {
    Internal {
        zero: Option<Box<Node<'a>>>,
        one: Option<Box<Node<'a>>>,
    },
    Leaf {
        letter: &'a Letter,
    },
}

impl Node<'_> {
    // These methods are used only by unit tests.
    #![allow(dead_code)]

    fn new0<'a>(zero: Node<'a>) -> Node<'a> {
        Node::Internal {
            zero: Some(Box::new(zero)),
            one: None,
        }
    }

    pub fn new1<'a>(one: Node<'a>) -> Node<'a> {
        Node::Internal {
            zero: None,
            one: Some(Box::new(one)),
        }
    }

    pub fn newl<'a>(l: &'a Letter) -> Node<'a> {
        Node::Leaf { letter: l }
    }

    pub fn new<'a>(zero: Node<'a>, one: Node<'a>) -> Node<'a> {
        Node::Internal {
            zero: Some(Box::new(zero)),
            one: Some(Box::new(one)),
        }
    }
}

#[cfg(test)]
mod pack_tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn roundtrip_trivial() {
        let a = Alphabet::new(vec![]).unwrap();
        let mut packed = Vec::<u8>::new();
        assert!(a.pack(&mut packed).is_ok());
        let got = Alphabet::unpack(Cursor::new(packed)).unwrap();
        assert_eq!(got.0.len(), 0);
    }

    #[test]
    fn roundtrip_single_letter() {
        let v = vec![Letter::from_bytes(&[0b10000001])];
        let a = Alphabet::new(v.clone()).unwrap();
        let mut packed = Vec::<u8>::new();
        assert!(a.pack(&mut packed).is_ok());
        let got = Alphabet::unpack(Cursor::new(packed)).unwrap();
        assert_eq!(got.0, v);
    }

    #[test]
    fn roundtrip_byte_letters() {
        let v = vec![
            Letter::from_bytes(&[0b10000001]),
            Letter::from_bytes(&[0b10000000]),
            Letter::from_bytes(&[0b00000111]),
        ];
        let a = Alphabet::new(v.clone()).unwrap();
        let mut packed = Vec::<u8>::new();
        assert!(a.pack(&mut packed).is_ok());
        let got = Alphabet::unpack(Cursor::new(packed)).unwrap();
        assert_eq!(got.0, v);
    }

    #[test]
    fn roundtrip_large_letters() {
        let v = vec![
            Letter::from_bytes(&[0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, 0x88, 0x99]),
            Letter::from_bytes(&[0xa1, 0xa2, 0xa3, 0xa4, 0xa5, 0xa6, 0xa7, 0xa8, 0xa9]),
        ];
        let a = Alphabet::new(v.clone()).unwrap();
        let mut packed = Vec::<u8>::new();
        assert!(a.pack(&mut packed).is_ok());
        let got = Alphabet::unpack(Cursor::new(packed)).unwrap();
        assert_eq!(got.0, v);
    }

    #[test]
    fn roundtrip_many_letters() {
        let v = vec![
            Letter::from_bytes(&[0x11]),
            Letter::from_bytes(&[0x12]),
            Letter::from_bytes(&[0x13]),
            Letter::from_bytes(&[0x14]),
            Letter::from_bytes(&[0x15]),
            Letter::from_bytes(&[0x16]),
            Letter::from_bytes(&[0x17]),
            Letter::from_bytes(&[0x18]),
            Letter::from_bytes(&[0x19]),
        ];
        let a = Alphabet::new(v.clone()).unwrap();
        let mut packed = Vec::<u8>::new();
        assert!(a.pack(&mut packed).is_ok());
        let got = Alphabet::unpack(Cursor::new(packed)).unwrap();
        assert_eq!(got.0, v);
    }

    #[test]
    fn roundtrip_different_lengths() {
        let v = vec![
            Letter::from_bytes(&[0x01]),
            Letter::from_bytes(&[0xa1, 0xa2]),
            Letter::from_bytes(&[0xb1, 0xb2, 0xb3]),
            Letter::from_bytes(&[0xc1, 0xc2]),
            Letter::from_bytes(&[0xd1]),
        ];
        let a = Alphabet::new(v.clone()).unwrap();
        let mut packed = Vec::<u8>::new();
        assert!(a.pack(&mut packed).is_ok());
        let got = Alphabet::unpack(Cursor::new(packed)).unwrap();
        assert_eq!(got.0, v);
    }

    #[test]
    fn roundtrip_unaligned_lengths() {
        let v = vec![
            Letter::new(&[0b1110_0000], 3),
            Letter::new(&[0b1000_0000, 0x11], 11),
        ];
        let a = Alphabet::new(v.clone()).unwrap();
        let mut packed = Vec::<u8>::new();
        assert!(a.pack(&mut packed).is_ok());
        let got = Alphabet::unpack(Cursor::new(packed)).unwrap();
        assert_eq!(got.0, v);
    }
}

#[cfg(test)]
mod tree_tests {
    use super::*;

    #[test]
    fn empty() {
        let a = Alphabet::new(vec![]).unwrap();
        assert_eq!(
            a.tree().unwrap(),
            Node::Internal {
                zero: None,
                one: None
            }
        );
    }

    #[test]
    fn leaf1() {
        let l = Letter::new(&[0b1000_0000], 1);
        let a = Alphabet::new(vec![l.clone()]).unwrap();
        assert_eq!(a.tree().unwrap(), Node::new1(Node::newl(&l)));
    }

    #[test]
    fn leaf0() {
        let l = Letter::new(&[0b1000_0000], 2);
        let a = Alphabet::new(vec![l.clone()]).unwrap();
        assert_eq!(a.tree().unwrap(), Node::new1(Node::new0(Node::newl(&l))),);
    }

    #[test]
    fn multi_byte_letter() {
        let l = Letter::new(&[0b1000_0000, 0b1100_0000], 10);
        let a = Alphabet::new(vec![l.clone()]).unwrap();
        assert_eq!(
            a.tree().unwrap(),
            Node::new1(Node::new0(Node::new0(Node::new0(Node::new0(Node::new0(
                Node::new0(Node::new0(Node::new1(Node::new1(Node::newl(&l)))))
            )))))),
        );
    }

    #[test]
    fn unshared_letters() {
        let l0 = Letter::new(&[0b0100_0000], 2);
        let l1 = Letter::new(&[0b1000_0000], 1);
        let a = Alphabet::new(vec![l0.clone(), l1.clone()]).unwrap();
        assert_eq!(
            a.tree().unwrap(),
            Node::new(Node::new1(Node::newl(&l0)), Node::newl(&l1)),
        )
    }

    #[test]
    fn shared_letters() {
        let l0 = Letter::new(&[0b0100_0000], 3);
        let l1 = Letter::new(&[0b0110_0000], 4);
        let a = Alphabet::new(vec![l0.clone(), l1.clone()]).unwrap();
        assert_eq!(
            a.tree().unwrap(),
            Node::new0(Node::new1(Node::new(
                Node::newl(&l0),
                Node::new0(Node::newl(&l1)),
            ))),
        );
    }

    #[test]
    fn multi_byte_shared() {
        let l0 = Letter::new(&[0b1000_0000, 0b1100_0000], 10);
        let l1 = Letter::new(&[0b1000_0000, 0b0000_0000], 10);
        let l2 = Letter::new(&[0b1010_0000], 3);
        let l3 = Letter::new(&[0b0010_0000], 3);
        let a = Alphabet::new(vec![l0.clone(), l1.clone(), l2.clone(), l3.clone()]).unwrap();
        assert_eq!(
            a.tree().unwrap(),
            Node::new(
                Node::new0(Node::new1(Node::newl(&l3))),
                Node::new0(Node::new(
                    Node::new0(Node::new0(Node::new0(Node::new0(Node::new0(Node::new(
                        Node::new0(Node::newl(&l1)),
                        Node::new1(Node::newl(&l0)),
                    )))))),
                    Node::newl(&l2),
                )),
            ),
        );
    }
}
