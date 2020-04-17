//! bytes module implements tokenization of a string into bytes.
//!
//! The stream makes zero copies internally while iterating over the stream.

use crate::tokens::{Result, Token, TokenIter, Tokens};
use std::fmt;
use std::hash::Hash;
use std::io::Write;

/// A `Token` consisting of a single byte of data.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Byte(u8);

/// Pack an iterator for `Byte`s to a `Write`er.
pub fn pack<I, W>(i: I, w: &mut W) -> Result<()>
where
    I: std::iter::Iterator<Item = Byte>,
    W: std::io::Write,
{
    let mut bw = std::io::BufWriter::new(w);
    let mut buf: [u8; 1] = [0; 1];
    for b in i {
        buf[0] = b.0;
        if let Err(e) = bw.write_all(&buf[..]) {
            return Err(e.to_string());
        }
    }
    bw.flush().map_err(|e| e.to_string())?;
    Ok(())
}

/// Unpack `Byte`s from a `Read`er into a `TokenIter`.
pub fn unpack<'a, R: std::io::Read>(r: &'a mut R) -> impl TokenIter<'a, T = Byte> {
    BytesIter::<'a, R>(r)
}

/// Deprecated Tokens implementation for Byte.
pub struct Bytes<'a>(std::str::Bytes<'a>);

impl std::fmt::Display for Byte {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl Token for Byte {
    fn bit_count(&self) -> usize {
        8
    }
}

impl std::iter::Iterator for Bytes<'_> {
    type Item = Byte;

    fn next(&mut self) -> Option<Self::Item> {
        match self.0.next() {
            Some(b) => Some(Byte(b)),
            None => None,
        }
    }
}

impl<'a> Tokens<'a> for Bytes<'a> {
    fn from_text(text: &'a str) -> Self {
        Bytes(text.bytes())
    }
    fn to_text(self) -> Result<String> {
        let b: Vec<u8> = self.0.collect();
        let s = std::str::from_utf8(&b).map_err(|e| e.to_string())?;
        Ok(s.to_string())
    }
}

struct BytesIter<'a, R: std::io::Read>(&'a mut R);

impl<'a, R: std::io::Read> TokenIter<'a> for BytesIter<'a, R> {
    type T = Byte;
}

impl<R: std::io::Read> std::iter::Iterator for BytesIter<'_, R> {
    type Item = Result<Byte>;
    fn next(&mut self) -> Option<Self::Item> {
        let mut buf: [u8; 1] = [0; 1];
        match self.0.read(&mut buf[..]) {
            Err(e) => Some(Err(e.to_string())),
            Ok(0) => None,
            Ok(1) => Some(Ok(Byte(buf[0]))),
            Ok(l) => panic!("read {} bytes in 1 byte buffer", l),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    const TEXT: &str = "
Ah! well a-day! what evil looks
Had I from old and young!
Instead of the cross, the Albatross
About my neck was hung.
";
    #[test]
    fn roundtrip() {
        let mut r = Cursor::new(TEXT);
        let d = unpack(&mut r);
        let i = d.map(|i| match i {
            Err(e) => panic!(e),
            Ok(b) => b,
        });
        let mut wc: Cursor<Vec<u8>> = Cursor::new(vec![]);
        pack(i, &mut wc).unwrap();
        let got = std::str::from_utf8(&wc.get_ref()[..]).unwrap();
        assert_eq!(got, TEXT);
    }
}
