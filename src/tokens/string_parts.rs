//! The stream makes zero copies internally while iterating over the stream.

use crate::tokens::{Result, Token, TokenIter};
use unicode_segmentation::{self, UnicodeSegmentation};

use std::convert::{From, Into};
use std::io::Write;

pub fn unpack<S, R>(r: &mut R) -> impl TokenIter<T = S>
where
    S: From<String> + Token,
    R: std::io::Read,
{
    StringPartsIter::<S>::new(r)
}

pub struct StringPartsIter<S>(Option<Result<std::vec::IntoIter<S>>>)
where
    S: From<String> + Token;

impl<S> StringPartsIter<S>
where
    S: From<String> + Token,
{
    fn new<R>(r: &mut R) -> Self
    where
        R: std::io::Read,
    {
        let mut data = Vec::<u8>::new();
        if let Err(e) = r.read_to_end(&mut data) {
            return Self(Some(Err(e.to_string())));
        }
        match std::str::from_utf8(&data) {
            Err(e) => Self(Some(Err(e.to_string()))),
            Ok(s) => {
                let mut parts = Vec::<S>::new();
                for g in s.graphemes(true) {
                    parts.push(S::from(g.to_owned()));
                }
                Self(Some(Ok(parts.into_iter())))
            }
        }
    }
}

impl<S> TokenIter<'_> for StringPartsIter<S>
where
    S: From<String> + Token,
{
    type T = S;
}

impl<S> std::iter::Iterator for StringPartsIter<S>
where
    S: From<String> + Token,
{
    type Item = Result<S>;

    fn next(&mut self) -> Option<Self::Item> {
        let mut store = None;
        std::mem::swap(&mut self.0, &mut store);
        let result = match &mut store {
            None => {
                return None;
            }
            Some(n) => match n {
                Err(e) => return Some(Err((*e).to_string())),
                Ok(i) => match i.next() {
                    None => None,
                    Some(p) => Some(Ok(p)),
                },
            },
        };
        std::mem::swap(&mut store, &mut self.0);
        result
    }
}

pub fn pack<S, I, W>(i: I, w: &mut W) -> Result<()>
where
    S: Into<String> + Token,
    I: std::iter::Iterator<Item = S>,
    W: std::io::Write,
{
    let mut bw = std::io::BufWriter::new(w);
    for s in i {
        let buf: String = s.into();
        if let Err(e) = bw.write_all(buf.as_bytes()) {
            return Err(e.to_string());
        }
    }
    bw.flush().map_err(|e| e.to_string())?;
    Ok(())
}
