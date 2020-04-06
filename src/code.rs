use std::convert::TryInto;
use std::fmt;
use std::io::Write;
use std::u64;

/// core::result::Result alias with uniform error type.
///
/// All public functions from this package return results of this type.
type Result<R> = core::result::Result<R, String>;

/// A Letter represents an indivisible code point.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
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

static BIT_HOLE_MASKS: [u8; 8] = [
    0b1000_0000,
    0b0100_0000,
    0b0010_0000,
    0b0001_0000,
    0b0000_1000,
    0b0000_0100,
    0b0000_0010,
    0b0000_0001,
];

impl Letter {
    pub fn from_bytes(bytes: &[u8]) -> Self {
        Letter {
            bit_count: 8 * bytes.len() as u64,
            data: bytes.to_vec(),
        }
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

    fn pack(mut self) -> Vec<u8> {
        let mut p = Vec::new();
        p.append(&mut pack_u64(self.bit_count));
        p.append(&mut self.data);
        p
    }

    fn unpack(iter: &mut std::vec::IntoIter<u8>) -> core::result::Result<Self, String> {
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

fn pack_u64(s: u64) -> Vec<u8> {
    s.to_be_bytes().to_vec()
}

fn unpack_u64(iter: &mut std::vec::IntoIter<u8>) -> Result<u64> {
    let mut buf: [u8; 8] = [0; 8];
    for i in 0..8 {
        match iter.next() {
            Some(u) => buf[i] = u,
            None => return Err("too few elements".to_owned()),
        }
    }
    Ok(u64::from_be_bytes(buf))
}
/// Alphabet is an ordered list of unique Letters.
#[derive(Debug)]
pub struct Alphabet(Vec<Letter>);

impl Alphabet {
    /// Create a new Alphabet with the given Letters.Alphabet
    ///
    /// The order of Letters is significant. pack()/unpack() conserve the order.
    pub fn new(letters: Vec<Letter>) -> Self {
        Alphabet(letters)
    }
}

impl Alphabet {
    /// Serialize to a vector of bytes.
    ///
    /// Can be deserialized back to an Alphabet with unpack().
    pub fn pack(self) -> Vec<u8> {
        let letter_count = self.0.len();
        let mut data: Vec<u8> = Vec::new();
        data.append(&mut pack_u64(letter_count as u64));
        for l in self.0.into_iter() {
            data.append(&mut l.pack())
        }
        data
    }

    /// Deserialize a vector of bytes generated with pack().
    pub fn unpack(data: Vec<u8>) -> Result<Self> {
        let mut iter = data.into_iter();
        let letter_count = unpack_u64(&mut iter)?;
        let mut letters = Vec::new();
        for _ in 0..letter_count {
            let l = Letter::unpack(&mut iter)?;
            letters.push(l);
        }
        Ok(Alphabet(letters))
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
    pub fn new0<'a>(zero: Node<'a>) -> Node<'a> {
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

impl Alphabet {
    pub fn tree<'a>(&'a self) -> Result<Node<'a>> {
        if self.0.len() == 0 {
            return Err("no letters".to_owned());
        }
        let mut root = Node::Internal {
            zero: None,
            one: None,
        };
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
                    Node::Leaf { .. } => return Err(format!("Duplicate prefix {}", l)),
                    _ => return Ok((tree, offset)),
                }
            }
            Ok(true) => {
                match tree {
                    Node::Internal { one: Some(one), .. } => {
                        return Alphabet::follow_branch(one, l, offset + 1);
                    }
                    // This error message actually needs a slice l[0:offset]
                    Node::Leaf { .. } => return Err(format!("Duplicate prefix {}", l)),
                    _ => return Ok((tree, offset)),
                }
            }
            Err(_) => Err(format!("Duplicate prefix {}", l)),
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

/// An alphabet may be generated from an iterator over Letter.
///
/// This operation clone()s the Letters.
impl<'a> std::iter::FromIterator<&'a Letter> for Alphabet {
    fn from_iter<I: IntoIterator<Item = &'a Letter>>(i: I) -> Self {
        let mut a = Alphabet(Vec::new());
        for l in i {
            a.0.push(l.clone());
        }
        a
    }
}

/// Read previously pack()ed text given the corresponding Alphabet.
pub fn parse<R>(_a: Alphabet, _r: R) -> impl std::iter::Iterator<Item = Result<Letter>>
where
    R: std::io::Read,
{
    vec![Err("not implemented".to_owned())].into_iter()
}

/// Write a packed stream of letters.
///
/// Returns the number of bytes written.
pub fn pack<I, W>(letters: I, w: W) -> core::result::Result<usize, String>
where
    I: Iterator<Item = Letter>,
    W: std::io::Write,
{
    let mut bytes_written: usize = 0;
    let mut bw = std::io::BufWriter::new(w);
    let mut byte_buffer_len: u64 = 0; // In practice <= 7
    let mut byte_buffer: u8 = 0;
    for l in letters {
        let mut has_more_bytes = true;
        let mut remaining_bit_count = l.bit_count;
        for b in l.data.into_iter() {
            assert!(has_more_bytes);
            byte_buffer |= b >> byte_buffer_len;
            if byte_buffer_len + remaining_bit_count >= 8 {
                bytes_written += bw.write(&[byte_buffer]).map_err(|e| e.to_string())?;

                byte_buffer = safe_overflow_leftshift(b, 8 - byte_buffer_len);
                remaining_bit_count -= 8;
            } else {
                byte_buffer_len += remaining_bit_count;
                has_more_bytes = false;
            }
        }
    }
    if byte_buffer_len > 0 {
        bytes_written += bw.write(&[byte_buffer]).map_err(|e| e.to_string())?;
    }
    bw.flush().map_err(|e| e.to_string())?;
    Ok(bytes_written)
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
mod alphabet_pack_tests {
    use super::*;

    #[test]
    fn roundtrip_trivial() {
        let a = Alphabet::new(vec![]);
        let packed = a.pack();
        let got = Alphabet::unpack(packed).unwrap();
        assert_eq!(got.0.len(), 0);
    }

    #[test]
    fn roundtrip_single_letter() {
        let v = vec![Letter::from_bytes(&[0b10000001])];
        let a = Alphabet::new(v.clone());
        let packed = a.pack();
        let got = Alphabet::unpack(packed).unwrap();
        assert_eq!(got.0, v);
    }

    #[test]
    fn roundtrip_single_letter_zeroes() {
        let v = vec![Letter::from_bytes(&[0])];
        let a = Alphabet::new(v.clone());
        let packed = a.pack();
        let got = Alphabet::unpack(packed).unwrap();
        assert_eq!(got.0, v);
    }
    #[test]
    fn roundtrip_byte_letters() {
        let v = vec![
            Letter::from_bytes(&[0b10000001]),
            Letter::from_bytes(&[0b10000000]),
            Letter::from_bytes(&[0b00000111]),
        ];
        let a = Alphabet::new(v.clone());
        let packed = a.pack();
        let got = Alphabet::unpack(packed).unwrap();
        assert_eq!(got.0, v);
    }

    #[test]
    fn roundtrip_large_letters() {
        let v = vec![
            Letter::from_bytes(&[0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, 0x88, 0x99]),
            Letter::from_bytes(&[0xa1, 0xa2, 0xa3, 0xa4, 0xa5, 0xa6, 0xa7, 0xa8, 0xa9]),
        ];
        let a = Alphabet::new(v.clone());
        let packed = a.pack();
        let got = Alphabet::unpack(packed).unwrap();
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
        let a = Alphabet::new(v.clone());
        let packed = a.pack();
        let got = Alphabet::unpack(packed).unwrap();
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
        let a = Alphabet::new(v.clone());
        let packed = a.pack();
        let got = Alphabet::unpack(packed).unwrap();
        assert_eq!(got.0, v);
    }

    #[test]
    fn roundtrip_unaligned_lengths() {
        let v = vec![
            Letter {
                bit_count: 3,
                data: vec![0b111],
            },
            Letter {
                bit_count: 11,
                data: vec![0b100, 0x11],
            },
        ];
        let a = Alphabet::new(v.clone());
        let packed = a.pack();
        let got = Alphabet::unpack(packed).unwrap();
        assert_eq!(got.0, v);
    }
}

#[cfg(test)]
mod text_pack_tests {
    use super::*;

    #[test]
    fn empty() {
        let letters: Vec<Letter> = vec![];
        let mut got = Vec::new();
        assert_eq!(pack(letters.into_iter(), &mut got), Ok(0));
        assert_eq!(got, Vec::new());
    }

    #[test]
    fn zero_letter() {
        let letters: Vec<Letter> = vec![Letter::from_bytes(&[])];
        let mut got = Vec::new();
        assert_eq!(pack(letters.into_iter(), &mut got), Ok(0));
        assert_eq!(got, Vec::new());
    }
    #[test]
    fn single_byte() {
        let letters = vec![Letter::from_bytes(&[0x11])];
        let mut got = Vec::new();
        assert_eq!(pack(letters.into_iter(), &mut got), Ok(1));
        assert_eq!(got, [0x11].to_vec());
    }

    #[test]
    fn single_aligned_letter() {
        let letters = vec![Letter::from_bytes(&[
            0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, 0x88, 0x99, 0xaa,
        ])];
        let mut got = Vec::new();
        let want: Vec<u8> = vec![0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, 0x88, 0x99, 0xaa];
        assert_eq!(pack(letters.into_iter(), &mut got), Ok(want.len()));
        assert_eq!(got, want);
    }

    #[test]
    fn single_unaligned_short_letter() {
        let letters = vec![Letter {
            data: vec![0b1101_1000],
            bit_count: 5,
        }];
        let mut got = Vec::new();
        let want: Vec<u8> = vec![0b1101_1000];
        assert_eq!(pack(letters.into_iter(), &mut got), Ok(want.len()));
        assert_eq!(got, want);
    }

    #[test]
    fn single_unaligned_long_letter() {
        let letters = vec![Letter {
            data: vec![0b11011000, 0b11100000],
            bit_count: 13,
        }];
        let mut got = Vec::new();
        let want: Vec<u8> = vec![0b1101_1000, 0b1110_0000];
        assert_eq!(pack(letters.into_iter(), &mut got), Ok(want.len()));
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
        assert_eq!(pack(letters.into_iter(), &mut got), Ok(want.len()));
        assert_eq!(got, want);
    }

    #[test]
    fn multiple_unaligned_letters() {
        let letters = vec![
            Letter {
                data: vec![0b1101_1000, 0b1000_0000],
                bit_count: 9,
            },
            Letter {
                data: vec![0b1101_0000],
                bit_count: 4,
            },
            Letter::from_bytes(&[0b1101_1101]),
        ];
        let mut got = Vec::new();
        let want: Vec<u8> = vec![0b1101_1000, 0b1110_1110, 0b1110_1000];
        assert_eq!(pack(letters.into_iter(), &mut got), Ok(want.len()));
        assert_eq!(got, want);
    }
}

#[cfg(test)]
mod letter_tests {
    use super::*;

    #[test]
    fn at() {
        let l = Letter {
            data: vec![0b0010_0010],
            bit_count: 7,
        };
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

#[cfg(test)]
mod alphabet_tree_tests {
    use super::*;

    #[test]
    fn empty() {
        let a = Alphabet::new(vec![]);
        assert!(a.tree().is_err())
    }

    #[test]
    fn leaf0() {
        let l = Letter {
            data: vec![0b0],
            bit_count: 1,
        };
        let a = Alphabet::new(vec![l.clone()]);
        assert_eq!(a.tree().unwrap(), Node::new0(Node::newl(&l)),);
    }

    #[test]
    fn leaf1() {
        let l = Letter {
            data: vec![0b1000_0000],
            bit_count: 1,
        };
        let a = Alphabet::new(vec![l.clone()]);
        assert_eq!(a.tree().unwrap(), Node::new1(Node::newl(&l)));
    }

    #[test]
    fn multi_byte_letter() {
        let l = Letter {
            data: vec![0b1000_0000, 0b1100_0000],
            bit_count: 10,
        };
        let a = Alphabet::new(vec![l.clone()]);
        assert_eq!(
            a.tree().unwrap(),
            Node::new1(Node::new0(Node::new0(Node::new0(Node::new0(Node::new0(
                Node::new0(Node::new0(Node::new1(Node::new1(Node::newl(&l)))))
            )))))),
        );
    }

    #[test]
    fn unshared_letters() {
        let l0 = Letter {
            data: vec![0b0],
            bit_count: 1,
        };
        let l1 = Letter {
            data: vec![0b1000_0000],
            bit_count: 1,
        };
        let a = Alphabet::new(vec![l0.clone(), l1.clone()]);
        assert_eq!(
            a.tree().unwrap(),
            Node::new(Node::newl(&l0), Node::newl(&l1)),
        )
    }

    #[test]
    fn shared_letters() {
        let l0 = Letter {
            data: vec![0b0100_0000],
            bit_count: 3,
        };
        let l1 = Letter {
            data: vec![0b0110_0000],
            bit_count: 4,
        };
        let a = Alphabet::new(vec![l0.clone(), l1.clone()]);
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
        let l0 = Letter {
            data: vec![0b1000_0000, 0b1100_0000],
            bit_count: 10,
        };
        let l1 = Letter {
            data: vec![0b1000_0000, 0b0000_0000],
            bit_count: 10,
        };
        let l2 = Letter {
            data: vec![0b1010_0000],
            bit_count: 3,
        };
        let l3 = Letter {
            data: vec![0b0000_0000],
            bit_count: 3,
        };
        let a = Alphabet::new(vec![l0.clone(), l1.clone(), l2.clone(), l3.clone()]);
        assert_eq!(
            a.tree().unwrap(),
            Node::new(
                Node::new0(Node::new0(Node::newl(&l3))),
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
