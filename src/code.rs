use bit_vec::BitVec;
use std::u64;
use std::usize;

/// A Letter represents an indivisible code point.
type Letter = BitVec;

/// Alphabet is an ordered list of unique Letters.
#[derive(Debug)]
pub struct Alphabet(Vec<Letter>);

impl Alphabet {
    /// Serialize to a stream of bytes.
    ///
    /// Can be deserialized back to an Alphabet with pack().
    pub fn pack(self) -> impl IntoIterator<Item = u8> {
        PackedAlphabet::new(self.0)
    }

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

impl Alphabet {
    /// Deserialize a stream of bytes generated with pack().
    pub fn unpack<T>(data: T) -> Result<Self, String>
    where
        T: IntoIterator<Item = u8>,
    {
        let mut iter = data.into_iter();
        let letter_count = Alphabet::unpack_usize(&mut iter)?;
        let size_width = Alphabet::unpack_usize(&mut iter)?;
        let _sizes = Alphabet::unpack_sizes(&mut iter, letter_count, size_width);

        Err("not implemented".to_owned())
    }

    fn unpack_usize<I>(iter: &mut I) -> Result<usize, String>
    where
        I: Iterator<Item = u8>,
    {
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

    fn unpack_sizes<I>(iter: &mut I, count: usize, width: usize) -> Result<Vec<usize>, String>
    where
        I: Iterator<Item = u8>,
    {
        let bit_count = count * width;
        let byte_count = (bit_count - 1) / 8 + 1;
        let mut bytes = Vec::with_capacity(byte_count);
        for _ in 0..(byte_count) {
            match iter.next() {
                Some(u) => bytes.push(u),
                None => return Err("too few elements".to_owned()),
            }
        }
        let mut bits = BitVec::from_bytes(&bytes);
        let mut sizes = Vec::with_capacity(count);

        Ok(sizes)
    }
}

// TODO: Move all static methods to Alphabet and directly return Vec<u8> making
// this struct redundant.
struct PackedAlphabet(Vec<u8>);

impl PackedAlphabet {
    pub fn new(letters: Vec<Letter>) -> Self {
        let letter_count = letters.len();
        let letter_size_width = log_2(PackedAlphabet::max_letter_size(&letters) as u64);

        let mut data: Vec<u8> = Vec::new();
        data.append(&mut (letter_count as u64).to_be_bytes().to_vec());
        data.append(&mut (letter_size_width).to_be_bytes().to_vec());
        data.append(&mut PackedAlphabet::pack_sizes(
            letter_size_width as usize,
            &letters,
        ));
        data.append(&mut PackedAlphabet::pack_letters(letters));

        Self(data)
    }

    fn max_letter_size(letters: &Vec<Letter>) -> usize {
        let mut m = 0;
        for i in letters.iter() {
            if i.len() > m {
                m = i.len();
            }
        }
        m
    }

    fn pack_sizes(width: usize, letters: &Vec<Letter>) -> Vec<u8> {
        let mut sizes = BitVec::with_capacity(width * letters.len());
        for l in letters.iter() {
            let mut size = l.clone();
            size.shrink_to_fit();
            for _ in 0..(width - size.len()) {
                sizes.push(false);
            }
            sizes.append(&mut size);
        }
        sizes.to_bytes()
    }

    fn pack_letters(letters: Vec<Letter>) -> Vec<u8> {
        let mut packed = BitVec::with_capacity(PackedAlphabet::total_size(&letters));
        for mut l in letters.into_iter() {
            packed.append(&mut l)
        }
        packed.to_bytes()
    }

    fn total_size(letters: &Vec<Letter>) -> usize {
        letters.iter().fold(0, |s, i| s + i.len())
    }
}

impl IntoIterator for PackedAlphabet {
    type Item = u8;
    type IntoIter = std::vec::IntoIter<u8>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

const fn num_bits<T>() -> u64 {
    (std::mem::size_of::<T>() * 8) as u64
}

fn log_2(x: u64) -> u64 {
    assert!(x > 0);
    num_bits::<u64>() - u64::from(x.leading_zeros())
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
