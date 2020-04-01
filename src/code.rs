use bit_vec::BitVec;
use std::convert::TryInto;
use std::u64;
use std::usize;

/// A Letter represents an indivisible code point.
type Letter = BitVec;

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
        data.append(&mut (letter_count as u64).to_be_bytes().to_vec());
        data.append(&mut self.pack_sizes());
        data.append(&mut self.pack_letters());

        data
    }

    fn pack_sizes(&self) -> Vec<u8> {
        let mut sizes = BitVec::with_capacity(64 * self.0.len());
        for l in self.0.iter() {
            let size = BitVec::from_bytes(&mut (l.len() as u64).to_be_bytes().to_vec());
            bitvec_slow_append(&mut sizes, &size)
        }
        sizes.to_bytes()
    }

    fn pack_letters(self) -> Vec<u8> {
        let mut packed = BitVec::with_capacity(self.total_size());
        for l in self.0.into_iter() {
            bitvec_slow_append(&mut packed, &l);
        }
        packed.to_bytes()
    }

    fn total_size(&self) -> usize {
        self.0.iter().fold(0, |s, i| s + i.len())
    }
}

impl Alphabet {
    /// Deserialize a vector of bytes generated with pack().
    pub fn unpack(data: Vec<u8>) -> Result<Self, String> {
        let mut iter = data.into_iter();
        let letter_count = Alphabet::unpack_usize(&mut iter)?;
        let letter_sizes = Alphabet::unpack_sizes(&mut iter, letter_count)?;
        let letters = Alphabet::unpack_letters(&mut iter, letter_sizes)?;
        Ok(Alphabet(letters))
    }

    fn unpack_usize(iter: &mut std::vec::IntoIter<u8>) -> Result<usize, String> {
        let mut buf: [u8; 8] = [0; 8];
        for i in 0..8 {
            match iter.next() {
                Some(u) => buf[i] = u,
                None => return Err("too few elements".to_owned()),
            }
        }
        let c = u64::from_be_bytes(buf);
        if c > (usize::max_value() as u64) {
            return Err("count too large".to_owned());
        }
        Ok(c as usize)
    }

    fn unpack_sizes(iter: &mut std::vec::IntoIter<u8>, count: usize) -> Result<Vec<usize>, String> {
        let byte_count = count * 8;
        let mut bytes = Vec::with_capacity(byte_count);
        for _ in 0..byte_count {
            match iter.next() {
                Some(u) => bytes.push(u),
                None => return Err("too few elements".to_owned()),
            }
        }
        let mut bits = BitVec::from_bytes(&bytes);
        let mut sizes = Vec::with_capacity(count);

        for _ in 0..count {
            assert!(bits.len() >= 64);
            let word = bits.split_off(bits.len() - 64);
            assert!(word.len() == 64);
            let buf: [u8; 8] = word.to_bytes().as_slice().try_into().unwrap();
            let s = u64::from_be_bytes(buf);
            if s > (usize::max_value() as u64) {
                return Err(format!("size {} too large", s).to_owned());
            }
            sizes.push(s as usize)
        }

        Ok(sizes)
    }

    fn unpack_letters(
        iter: &mut std::vec::IntoIter<u8>,
        sizes: Vec<usize>,
    ) -> Result<Vec<Letter>, String> {
        let mut bits = BitVec::from_bytes(&iter.collect::<Vec<u8>>());
        let mut letters = Vec::with_capacity(sizes.len());
        for size in sizes.iter() {
            if bits.len() < *size {
                return Err("ran out of buts".to_owned());
            }
            letters.push(bits.split_off(bits.len() - *size));
        }
        letters.reverse();
        Ok(letters)
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

// Can't use BitVec::append() because of
// https://github.com/contain-rs/bit-vec/issues/63
fn bitvec_slow_append(v: &mut BitVec, o: &BitVec) {
    for i in 0..o.len() {
        v.push(o.get(i).unwrap())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn alphabet_roundtrip_trivial() {
        let a = Alphabet::new(vec![]);
        let packed = a.pack();
        let got = Alphabet::unpack(packed).unwrap();
        assert_eq!(got.0.len(), 0);
    }

    #[test]
    fn alphabet_roundtrip_single_letter() {
        let v = vec![BitVec::from_bytes(&[0b10000001])];
        let a = Alphabet::new(v.clone());
        let packed = a.pack();
        let got = Alphabet::unpack(packed).unwrap();
        assert_eq!(got.0, v);
    }

    #[test]
    fn alphabet_roundtrip_single_letter_zeroes() {
        let v = vec![BitVec::from_bytes(&[0])];
        let a = Alphabet::new(v.clone());
        let packed = a.pack();
        let got = Alphabet::unpack(packed).unwrap();
        assert_eq!(got.0, v);
    }
    #[test]
    fn alphabet_roundtrip_byte_letters() {
        let v = vec![
            BitVec::from_bytes(&[0b10000001]),
            BitVec::from_bytes(&[0b10000000]),
            BitVec::from_bytes(&[0b00000111]),
        ];
        let a = Alphabet::new(v.clone());
        let packed = a.pack();
        let got = Alphabet::unpack(packed).unwrap();
        assert_eq!(got.0.len(), 3);
        assert_eq!(got.0, v);
    }
}
