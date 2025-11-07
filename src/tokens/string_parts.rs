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

//! The stream makes zero copies internally while iterating over the stream.

use crate::tokens::{Token, TokenPacker};
use unicode_segmentation::{self, UnicodeSegmentation};

use anyhow::{Error, Result};
use log::{log_enabled, trace, Level};
use std::convert::{From, Into};
use std::marker::PhantomData;

#[derive(Clone, Debug)]
pub struct StringPartsIter<S>(Option<std::vec::IntoIter<S>>)
where
    S: From<String> + Token;

impl<S> StringPartsIter<S>
where
    S: From<String> + Token,
{
    pub fn new<R>(mut r: R) -> Result<Self>
    where
        R: std::io::Read,
    {
        let mut data = Vec::<u8>::new();
        r.read_to_end(&mut data)?;
        let s = std::str::from_utf8(&data)?;
        let mut parts = Vec::<S>::new();
        for g in s.graphemes(true) {
            parts.push(S::from(g.to_owned()));
        }
        Ok(Self(Some(parts.into_iter())))
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
        match store {
            None => {
                return None;
            }
            Some(mut i) => {
                let result = i.next();
                std::mem::swap(&mut Some(i), &mut self.0);

                if log_enabled!(Level::Trace) {
                    if let Some(v) = &result {
                        trace!("iter: |{}|", v);
                    }
                }
                result.map(Ok)
            }
        }
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct StringPartsPacker<S>(PhantomData<S>)
where
    S: Into<String> + Token;

impl<S> TokenPacker for StringPartsPacker<S>
where
    S: Into<String> + Token,
{
    type T = S;

    fn pack<I, W: std::io::Write>(i: I, mut w: W) -> Result<()>
    where
        I: std::iter::Iterator<Item = Self::T>,
    {
        for s in i {
            let buf: String = s.into();
            trace!("pack: |{}|", &buf);
            if let Err(e) = w.write_all(buf.as_bytes()) {
                return Err(Error::new(e));
            }
        }
        w.flush()?;
        Ok(())
    }
}
