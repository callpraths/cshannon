use std::convert::TryInto;
use std::u64;

/// A Letter represents an indivisible code point.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Letter {
    data: Vec<u8>,
    bit_count: u64,
}

impl Letter {
    pub fn from_bytes(bytes: &[u8]) -> Self {
        Letter {
            bit_count: 8 * bytes.len() as u64,
            data: bytes.to_vec(),
        }
    }

    fn pack(mut self) -> Vec<u8> {
        let mut p = Vec::new();
        p.append(&mut pack_u64(self.bit_count));
        p.append(&mut self.data);
        p
    }

    fn unpack(iter: &mut std::vec::IntoIter<u8>) -> Result<Self, String> {
        let bit_count = unpack_u64(iter)?;
        let data = Letter::unpack_data(iter, bit_count)?;
        Ok(Self {
            bit_count: bit_count,
            data: data,
        })
    }

    fn unpack_data(iter: &mut std::vec::IntoIter<u8>, bit_count: u64) -> Result<Vec<u8>, String> {
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

fn unpack_u64(iter: &mut std::vec::IntoIter<u8>) -> Result<u64, String> {
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
    pub fn unpack(data: Vec<u8>) -> Result<Self, String> {
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

impl Alphabet {
    /// Parse a stream of bytes coded with this Alphabet into a Text.
    ///
    /// See Text::pack() for the reverse operation.
    pub fn parse<T>(&self, _data: T) -> Text
    where
        T: IntoIterator<Item = u8>,
    {
        Text(Vec::new())
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

/// A coded stream in some Alphabet.
///
/// Text is a zero copy abstraction over a byte stream that allows iterating
/// over the underlying byte stream in the relevant Alphabet avoiding memory
/// copy or fragmentation.
#[derive(Debug)]
pub struct Text(Vec<Letter>);

/// Iterate over the Text in the underlying Alphabet.
///
/// The iteration avoids memory copy or fragmentation.
impl<'a> IntoIterator for &'a Text {
    type Item = &'a Letter;
    type IntoIter = std::slice::Iter<'a, Letter>;

    fn into_iter(self) -> Self::IntoIter {
        (&self.0).into_iter()
    }
}

impl Text {
    /// Serialize Text into a byte stream.
    ///
    /// May be deserialized (with known Alphabet) via Alphabet::parse()
    pub fn pack(self) -> std::vec::IntoIter<u8> {
        Vec::new().into_iter()
    }
}

#[cfg(test)]
mod alphabet_tests {
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
