//! The stream makes zero copies internally while iterating over the stream.

use crate::tokens::{Token, TokenIter, TokenPacker};
use unicode_segmentation::{self, UnicodeSegmentation};

use anyhow::{Error, Result};
use std::convert::{From, Into};
use std::marker::PhantomData;

pub struct StringPartsIter<S>(Option<Result<std::vec::IntoIter<S>>>)
where
    S: From<String> + Token;

impl<S> StringPartsIter<S>
where
    S: From<String> + Token,
{
    fn new<R>(mut r: R) -> Self
    where
        R: std::io::Read,
    {
        let mut data = Vec::<u8>::new();
        if let Err(e) = r.read_to_end(&mut data) {
            return Self(Some(Err(Error::new(e))));
        }
        match std::str::from_utf8(&data) {
            Err(e) => Self(Some(Err(Error::new(e)))),
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

impl<'a, S, R> TokenIter<R> for StringPartsIter<S>
where
    S: From<String> + Token,
    R: std::io::Read,
{
    type T = S;

    fn unpack(r: R) -> Self {
        Self::new(r)
    }
}

impl<S> std::iter::Iterator for StringPartsIter<S>
where
    S: From<String> + Token,
{
    type Item = Result<S>;

    fn next(&mut self) -> Option<Self::Item> {
        let mut store = None;
        std::mem::swap(&mut self.0, &mut store);
        let mut i = match store {
            None => {
                return None;
            }
            Some(n) => match n {
                Err(e) => return Some(Err(e)),
                Ok(i) => i,
            },
        };
        let result = i.next();
        std::mem::swap(&mut Some(Ok(i)), &mut self.0);
        result.map(Ok)
    }
}

pub struct StringPartsPacker<S>(PhantomData<S>)
where
    S: Into<String> + Token;

impl<S, W> TokenPacker<W> for StringPartsPacker<S>
where
    S: Into<String> + Token,
    W: std::io::Write,
{
    type T = S;

    fn pack<I>(i: I, w: &mut W) -> Result<()>
    where
        I: std::iter::Iterator<Item = Self::T>,
    {
        for s in i {
            let buf: String = s.into();
            if let Err(e) = w.write_all(buf.as_bytes()) {
                return Err(Error::new(e));
            }
        }
        w.flush()?;
        Ok(())
    }
}
