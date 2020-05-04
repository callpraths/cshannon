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
use log::trace;
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
    let tk = m.tokens_sorted();
    let fk = tk.iter().map(|t| m.probability(t));
    let lk = fk.map(l);
    let ck = CumulativeProbabilities::new(&m);
    let ek = ck.zip(lk).map(|(c, l)| e(c, l));
    let mut map: HashMap<T, Letter> = tk.iter().cloned().zip(ek).collect();

    // Need to fixup the code for the last token.
    // See comments in e_terminal() for details.
    if let Some(t_last) = tk.last() {
        if let Some(e_last) = map.get_mut(t_last) {
            *e_last = e_terminal(l(m.probability(t_last)));
        } else {
            panic!("did not find a code for the last token {}", t_last);
        }
    } else {
        panic!("failed to obtain last item from a vector");
    }

    Ok(Encoding::new(map)?)
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
    trace!("next letter: {}", letter);
    letter
}

// Shannon's encoding scheme has a discontinuity at the last token.
// Theoretically, the cumulative probability for the last token is 1.0
// The part before "1" is ignored when computing the code.
//
// This creates problem when paired with imprecise floating point arithmetic: A
// small change in the cumulative frequency at 1.0 leads to a huge difference in
// the code.
//
// e.g., code for 1.0 (with 4 bits) -> 0000
//       code for 0.9999 (with 4 bits) -> 1111
//
// The true Shannon code for this should be all 0s, but that is not allowed
// in our implementation, so we tack on a trailing 1.
// It is safe to add a trailing 1 because the initial 0s already ensure that
// the prefix condition is satisfied.
fn e_terminal(l: u64) -> Letter {
    let mut letter = Letter::with_capacity(l);
    for _ in 0..l {
        letter.push0();
    }
    letter.push1();
    trace!("terminal letter: {}", letter);
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
        // c: 0.4, 0.7, 0.9, 1.0
        // e: 01, 10, 111, 0000
        // e[corrected for all 0s]: 01, 10, 111, 0000_1
        let m = model::with_frequencies(
            [
                (I32Token(1), 4),
                (I32Token(2), 3),
                (I32Token(3), 2),
                (I32Token(4), 1),
            ]
            .iter()
            .cloned()
            .collect(),
        );
        let t = new(m).unwrap();
        assert_eq!(t.alphabet().len(), 4);
        let want: HashMap<I32Token, Letter> = [
            (I32Token(1), Letter::new(&[0b0100_0000], 2)),
            (I32Token(2), Letter::new(&[0b1000_0000], 2)),
            (I32Token(3), Letter::new(&[0b1110_0000], 3)),
            (I32Token(4), Letter::new(&[0b0000_1000], 5)),
        ]
        .iter()
        .cloned()
        .collect();
        assert_eq!(t.map(), &want);
    }
}
