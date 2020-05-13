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
use log::{debug, log_enabled, Level};
use std::collections::HashMap;

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
    if m.is_empty() {
        return Encoding::new(HashMap::new());
    }

    let tk = m.tokens_sorted();
    let fk = tk.iter().map(|t| m.probability(t));
    let lk = fk.map(l);
    let ck = CumulativeProbabilities::new(&m);
    let ek = ck.zip(lk).map(|(c, l)| e(c, l));
    let map: HashMap<T, Letter> = tk.iter().cloned().zip(ek).collect();

    if log_enabled!(Level::Debug) {
        let tk = m.tokens_sorted();
        let ck = CumulativeProbabilities::new(&m);
        debug!("t \t f \t l \t c \t e");
        for (dt, dc) in tk.iter().zip(ck) {
            let df = m.probability(dt);
            let dl = l(df);
            let de = e(dc, dl);
            debug!("{}\t{}\t{}\t{}\t{}", dt, df, dl, dc, de);
        }
    }

    Ok(Encoding::new(map)?)
}

fn l(f: f64) -> u64 {
    (-f.log2()).ceil() as u64
}

fn e(c: f64, l: u64) -> Letter {
    let mut letter = Letter::with_capacity(l);
    let mut mut_c = c;
    for _ in 0..l {
        mut_c *= 2.0;
        if mut_c > 1.0 {
            letter.push1();
            mut_c -= 1.0;
        } else {
            letter.push0();
        }
    }
    // The correct Shannon encoding for this consists of all zeroes, but
    // `Letter` with all zeroes is disallowed in this implementation.
    // We simply tack on a "1" at the end. Adding a "1" at the end does not
    // break the "prefix property" of the encoding, so it is safe.
    //
    // It does make the encoding for the most frequently occurring `Token`
    // longer. Thus, it can have a large impact on the compression ratio.
    if c == 0.0 {
        letter.push1();
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
            let old_sum = self.sum;
            self.sum += self.m.probability(&t);
            Some(old_sum)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::code::Letter;
    use crate::model;
    use crate::tokens::test_utils::I32Token;
    use crate::util::testing;
    use std::collections::HashMap;
    #[test]
    fn basic() {
        testing::init_logs_for_test();
        // f: 0.4, 0.3, 0.2, 0.1
        // l: 2, 2, 3, 4
        // c: 0.0, 0.4, 0.7, 0.9
        // e: 00, 01, 101, 1110
        // e[corrected for all 0s]: 001, 01, 101, 1110
        let m = model::with_frequencies(&[
            (I32Token(1), 4),
            (I32Token(2), 3),
            (I32Token(3), 2),
            (I32Token(4), 1),
        ]);
        let t = new(m).unwrap();
        assert_eq!(t.alphabet().len(), 4);
        let want: HashMap<I32Token, Letter> = [
            (I32Token(1), Letter::new(&[0b0010_0000], 3)),
            (I32Token(2), Letter::new(&[0b0100_0000], 2)),
            (I32Token(3), Letter::new(&[0b1010_0000], 3)),
            (I32Token(4), Letter::new(&[0b1110_0000], 4)),
        ]
        .iter()
        .cloned()
        .collect();
        assert_eq!(t.map(), &want);
    }
}
