use bit_vec::BitVec;

/// A Letter represents an indivisible code point.
///
/// Letters are essentially bit strings, endowed with useful methods for the
/// current use-case.
///
/// Letters must be built up from scratch by repeatedly adding or extending on
/// the right.
#[derive(Clone, Debug)]
pub struct Letter(BitVec);

impl Letter {
    /// Seed an empty Letter.
    pub fn new() -> Self {
        Letter(BitVec::new())
    }

    /// Create a new Letter with a new value appended to the right.
    ///
    /// Examples:
    /// ```
    /// let l = Letter::new();
    /// let nl = l.add(true);
    /// assert_eq!(l.to_string(), "");
    /// assert_eq!(nl.to_string(), "1");
    /// ```
    pub fn add(&self, value: bool) -> Self {
        let mut o = self.0.clone();
        o.push(value);
        Letter(o)
    }

    /// Create a new Letter with a new value appended to the right.
    ///
    /// Examples:
    /// ```
    /// let l = Letter::new();
    /// l.extend(true);
    /// assert_eq!(l.to_string(), "1")
    /// ```
    pub fn extend(&mut self, value: bool) {
        self.0.push(value)
    }
}

impl ToString for Letter {
    fn to_string(&self) -> String {
        let mut s = String::with_capacity(self.0.len());
        for v in self.0.iter() {
            if v {
                s.push_str("1")
            } else {
                s.push_str("0")
            }
        }
        s
    }
}

/// Alphabet is an ordered list of unique Letters.
#[derive(Debug)]
pub struct Alphabet(Vec<Letter>);

impl Alphabet {
    /// Deserialize a stream of bytes generated with pack().
    pub fn unpack<T>(_data: T) -> Self
    where
        T: IntoIterator<Item = u8>,
    {
        Alphabet(Vec::new())
    }

    /// Serialize to a stream of bytes.
    ///
    /// Can be deserialized back to an Alphabet with pack().
    pub fn pack(self) -> std::vec::IntoIter<u8> {
        Vec::<u8>::new().into_iter()
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
