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

use super::Encoding;
use crate::code::Letter;
use crate::model::Model;
use crate::tokens::Token;
use anyhow::Result;

/// Create a new Shannon `Encoding`.
///
/// The Shannon encoding scheme is defined thus:
///
/// Let the tokens, sorted in decreasing order of frequency be
///     t1, t2, t3 ...
/// Let the probability of occurrence of the Tokens be
///     f1, f2, f3 ...
/// Define the numbers
///     l1, l2, l3 ...
///     such that lk = ceil(log2(1/fk))
/// Let the (computed) cumulative proportions be
///     c1, c2, c3 ...
///
/// Then, the code is
///     e1, e2, e3 ...
///     such that ek = first lk bits of the binary expansion of Fk.
pub fn new<T>(m: Model<T>) -> Result<Encoding<T>>
where
    T: Token,
{
    let tk = m.tokens_sorted();
    let fk = tk.iter().map(|t| m.probability(t));
    let lk = fk.map(l);
    let ck = CumulativeProbabilities::new(&m);
    let ek = ck.zip(lk).map(|(c, l)| e(c, l));
    Ok(Encoding(tk.iter().cloned().zip(ek).collect()))
}

fn l(f: f64) -> u64 {
    (-f.log2()).ceil() as u64
}

fn e(mut c: f64, l: u64) -> Letter {
    let mut letter = Letter::with_capacity(l);
    for _ in 0..l {
        c *= 2.0;
        if c > 1.0 {
            letter.push1();
            c -= 1.0;
        } else {
            letter.push0();
        }
    }
    letter
}

struct CumulativeProbabilities<'a, T: Token> {
    m: &'a Model<T>,
    tokens: std::vec::IntoIter<T>,
    sum: f64,
}

impl<'a, T: Token> CumulativeProbabilities<'a, T> {
    pub fn new(m: &'a Model<T>) -> Self {
        Self {
            m,
            tokens: m.tokens_sorted().into_iter(),
            sum: 0.0,
        }
    }
}

impl<'a, T: Token> Iterator for CumulativeProbabilities<'a, T> {
    type Item = f64;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(t) = self.tokens.next() {
            self.sum += self.m.probability(&t);
            Some(self.sum)
        } else {
            None
        }
    }
}
