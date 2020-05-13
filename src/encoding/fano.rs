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

pub fn new<T>(m: Model<T>) -> Result<Encoding<T>>
where
    T: Token,
{
    if m.is_empty() {
        return Encoding::new(HashMap::new());
    }
    let flattened: Vec<(T, f64)> = m
        .tokens_sorted()
        .iter()
        .map(|t| (t.clone(), m.probability(t)))
        .collect();
    let mut cumulative: Vec<(T, f64)> = Vec::with_capacity(flattened.len());
    let mut sum = 0.0;
    for (t, p) in flattened.into_iter() {
        sum += p;
        cumulative.push((t, sum))
    }
    Encoding::new(CodeIter::new(Window::new(&mut cumulative)).collect())
}

#[derive(Debug, PartialEq)]
struct Window<'a, T: Token>(&'a mut [(T, f64)]);

impl<'a, T: Token> std::fmt::Display for Window<'a, T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut first = true;
        write!(f, "[")?;
        for (t, c) in self.0.iter() {
            if !first {
                write!(f, ", ")?;
            }
            write!(f, "({}, {})", t, c)?;
            first = false;
        }
        write!(f, "]")
    }
}

#[derive(Debug, PartialEq)]
enum Refinement<'a, T: Token> {
    Split(Window<'a, T>, Window<'a, T>),
    Terminal(T),
}

impl<'a, T: Token> std::fmt::Display for Refinement<'a, T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Refinement::Split(left, right) => write!(f, "Split({}, {})", left, right),
            Refinement::Terminal(t) => write!(f, "Terminal({})", t),
        }
    }
}

const LINEAR_SEARCH_THRESHOLD: usize = 6;

impl<'a, T: Token> Window<'a, T> {
    pub fn new(data: &'a mut [(T, f64)]) -> Self {
        Self(data)
    }

    pub fn refine(self) -> Refinement<'a, T> {
        trace!("refine({})", &self);
        let ret = match self.0.len() {
            0 => panic!("Window must be at least length 1"),
            1 => Refinement::Terminal(self.0[0].0.clone()),
            _ => {
                let mid_value = self.0[self.0.len() - 1].1 / 2.0;
                let split = self.find_split_binary_search(mid_value);
                self.split_at(split)
            }
        };
        trace!("  --> {}", &ret);
        ret
    }

    fn find_split_binary_search(&self, mid_value: f64) -> usize {
        assert!(self.0.len() > 1);
        let mut left: usize = 0;
        let mut right = self.0.len();
        while right - left > LINEAR_SEARCH_THRESHOLD {
            let mid = (left + right) / 2;
            if self.0[mid].1 > mid_value {
                right = mid;
            } else {
                left = mid;
            }
        }
        self.find_split_linear(mid_value, left, right)
    }

    fn find_split_linear(&self, mid_value: f64, left: usize, right: usize) -> usize {
        assert!(right - left > 1);
        let mut diff = (self.0[left].1 - mid_value).abs();
        for i in left + 1..right {
            let ndiff = (self.0[i].1 - mid_value).abs();
            if ndiff > diff {
                return i;
            }
            diff = ndiff;
        }
        right
    }

    fn split_at(self, split: usize) -> Refinement<'a, T> {
        let len = self.0.len();
        assert!(split <= len);
        let ptr = self.0.as_mut_ptr();
        let (left, right) = unsafe {
            (
                std::slice::from_raw_parts_mut(ptr, split),
                std::slice::from_raw_parts_mut(ptr.offset(split as isize), len - split),
            )
        };
        Refinement::Split(Self(left), Self(right))
    }
}

enum Frame<'a, T: Token> {
    Left { residual: Window<'a, T> },
    Right,
}

impl<'a, T: Token> std::fmt::Display for Frame<'a, T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Frame::Left { residual } => write!(f, "Left({})", residual),
            Frame::Right => write!(f, "Right"),
        }
    }
}

struct CodeIter<'a, T: Token>(Vec<Frame<'a, T>>);

impl<'a, T: Token> std::fmt::Display for CodeIter<'a, T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[")?;
        let mut first = true;
        for s in self.0.iter() {
            if !first {
                write!(f, " ")?;
            }
            write!(f, "<{}>", s)?;
            first = false;
        }
        write!(f, "*")
    }
}

impl<'a, T: Token> Iterator for CodeIter<'a, T> {
    type Item = (T, Letter);

    fn next(&mut self) -> Option<Self::Item> {
        trace!("next({})", &self);
        let residual = match self.unroll() {
            None => return None,
            Some(residual) => residual,
        };
        self.0.push(Frame::Right);
        let ret = self.descend(residual);
        trace!(" --> ({}, {})", &ret.0, &ret.1);
        Some(ret)
    }
}

impl<'a, T: Token> CodeIter<'a, T> {
    pub fn new(window: Window<'a, T>) -> Self {
        Self(vec![Frame::Left { residual: window }])
    }

    fn descend(&mut self, mut node: Window<'a, T>) -> (T, Letter) {
        loop {
            match node.refine() {
                Refinement::Terminal(t) => return (t, self.read_letter()),
                Refinement::Split(left, right) => {
                    self.0.push(Frame::Left { residual: right });
                    node = left;
                }
            }
        }
    }

    fn unroll(&mut self) -> Option<Window<'a, T>> {
        loop {
            match self.0.pop() {
                None => return None,
                Some(Frame::Right) => continue,
                Some(Frame::Left { residual }) => return Some(residual),
            }
        }
    }

    fn read_letter(&self) -> Letter {
        // TODO: The very first branch decision is a lie, and should be ignored.
        // That does raise an interesting question about input that is of length
        // 1. Should the code be ""?
        let mut letter = Letter::with_capacity(self.0.len() as u64);
        for frame in self.0.iter() {
            match frame {
                Frame::Left { residual: _ } => letter.push0(),
                Frame::Right => letter.push1(),
            };
        }
        letter
    }
}

#[cfg(test)]
mod refinement_tests {
    use super::*;
    use crate::tokens::test_utils::I32Token;

    #[test]
    fn single_elem() {
        let mut data = vec![(I32Token(1), 1.0)];
        assert_eq!(
            Window::new(&mut data).refine(),
            Refinement::Terminal(I32Token(1))
        );
    }

    #[test]
    fn two_elems_to_terminals() {
        let mut data = vec![(I32Token(1), 0.3), (I32Token(2), 1.0)];
        let mut left = vec![(I32Token(1), 0.3)];
        let mut right = vec![(I32Token(2), 1.0)];
        let refinement = Window::new(&mut data).refine();
        assert_eq!(
            &refinement,
            &Refinement::Split(Window::new(&mut left), Window::new(&mut right))
        );
        if let Refinement::Split(left, right) = refinement {
            assert_eq!(left.refine(), Refinement::Terminal(I32Token(1)));
            assert_eq!(right.refine(), Refinement::Terminal(I32Token(2)));
        } else {
            panic!("but you just said that the refinement was a window!");
        }
    }

    #[test]
    fn two_elems_right_heavy() {
        let mut data = vec![(I32Token(1), 0.7), (I32Token(2), 1.0)];
        let mut left = vec![(I32Token(1), 0.7)];
        let mut right = vec![(I32Token(2), 1.0)];
        assert_eq!(
            Window::new(&mut data).refine(),
            Refinement::Split(Window::new(&mut left), Window::new(&mut right))
        );
    }

    #[test]
    fn three_elems_larger_left() {
        let mut data = vec![(I32Token(1), 0.3), (I32Token(2), 0.4), (I32Token(3), 1.0)];
        let mut left = vec![(I32Token(1), 0.3), (I32Token(2), 0.4)];
        let mut right = vec![(I32Token(3), 1.0)];
        assert_eq!(
            Window::new(&mut data).refine(),
            Refinement::Split(Window::new(&mut left), Window::new(&mut right))
        );
    }

    #[test]
    fn three_elems_split_at_value() {
        let mut data = vec![(I32Token(1), 0.3), (I32Token(2), 0.5), (I32Token(3), 1.0)];
        let mut left = vec![(I32Token(1), 0.3), (I32Token(2), 0.5)];
        let mut right = vec![(I32Token(3), 1.0)];
        assert_eq!(
            Window::new(&mut data).refine(),
            Refinement::Split(Window::new(&mut left), Window::new(&mut right))
        );
    }

    #[test]
    fn three_elems_larger_right() {
        let mut data = vec![(I32Token(1), 0.4), (I32Token(2), 0.7), (I32Token(3), 1.0)];
        let mut left = vec![(I32Token(1), 0.4)];
        let mut right = vec![(I32Token(2), 0.7), (I32Token(3), 1.0)];
        assert_eq!(
            Window::new(&mut data).refine(),
            Refinement::Split(Window::new(&mut left), Window::new(&mut right))
        );
    }

    #[test]
    fn four_elems_closer_right() {
        let mut data = vec![
            (I32Token(1), 0.31),
            (I32Token(2), 0.60),
            (I32Token(3), 0.85),
            (I32Token(4), 1.0),
        ];
        let mut left = vec![(I32Token(1), 0.31), (I32Token(2), 0.60)];
        let mut right = vec![(I32Token(3), 0.85), (I32Token(4), 1.0)];
        assert_eq!(
            Window::new(&mut data).refine(),
            Refinement::Split(Window::new(&mut left), Window::new(&mut right))
        );
    }

    #[test]
    fn large_balanced_input() {
        let mut data: Vec<(I32Token, f64)> = vec![
            (I32Token(1), 0.05),
            (I32Token(2), 0.10),
            (I32Token(3), 0.15),
            (I32Token(4), 0.20),
            (I32Token(5), 0.25),
            (I32Token(6), 0.30),
            (I32Token(7), 0.35),
            (I32Token(8), 0.40),
            (I32Token(9), 0.45),
            // Skip token close to the split because floating point arithmetic
            // is inexact.
            (I32Token(11), 0.55),
            (I32Token(12), 0.60),
            (I32Token(13), 0.65),
            (I32Token(14), 0.70),
            (I32Token(15), 0.75),
            (I32Token(16), 0.80),
            (I32Token(17), 0.85),
            (I32Token(18), 0.90),
            (I32Token(19), 0.95),
            (I32Token(20), 1.00),
        ];
        let mut left: Vec<(I32Token, f64)> = vec![
            (I32Token(1), 0.05),
            (I32Token(2), 0.10),
            (I32Token(3), 0.15),
            (I32Token(4), 0.20),
            (I32Token(5), 0.25),
            (I32Token(6), 0.30),
            (I32Token(7), 0.35),
            (I32Token(8), 0.40),
            (I32Token(9), 0.45),
        ];
        let mut right: Vec<(I32Token, f64)> = vec![
            (I32Token(11), 0.55),
            (I32Token(12), 0.60),
            (I32Token(13), 0.65),
            (I32Token(14), 0.70),
            (I32Token(15), 0.75),
            (I32Token(16), 0.80),
            (I32Token(17), 0.85),
            (I32Token(18), 0.90),
            (I32Token(19), 0.95),
            (I32Token(20), 1.00),
        ];
        assert_eq!(
            Window::new(&mut data).refine(),
            Refinement::Split(Window::new(&mut left), Window::new(&mut right))
        );
    }

    #[test]
    fn large_left_heavy_input() {
        let mut data: Vec<(I32Token, f64)> = vec![
            (I32Token(1), 0.3),
            (I32Token(2), 0.48),
            (I32Token(11), 0.55),
            (I32Token(12), 0.60),
            (I32Token(13), 0.65),
            (I32Token(14), 0.70),
            (I32Token(15), 0.75),
            (I32Token(16), 0.80),
            (I32Token(17), 0.85),
            (I32Token(18), 0.90),
            (I32Token(19), 0.95),
            (I32Token(20), 1.00),
        ];
        let mut left: Vec<(I32Token, f64)> = vec![(I32Token(1), 0.3), (I32Token(2), 0.48)];
        let mut right: Vec<(I32Token, f64)> = vec![
            (I32Token(11), 0.55),
            (I32Token(12), 0.60),
            (I32Token(13), 0.65),
            (I32Token(14), 0.70),
            (I32Token(15), 0.75),
            (I32Token(16), 0.80),
            (I32Token(17), 0.85),
            (I32Token(18), 0.90),
            (I32Token(19), 0.95),
            (I32Token(20), 1.00),
        ];
        assert_eq!(
            Window::new(&mut data).refine(),
            Refinement::Split(Window::new(&mut left), Window::new(&mut right))
        );
    }

    #[test]
    fn large_right_heavy_input() {
        let mut data: Vec<(I32Token, f64)> = vec![
            (I32Token(1), 0.05),
            (I32Token(2), 0.10),
            (I32Token(3), 0.15),
            (I32Token(4), 0.20),
            (I32Token(5), 0.25),
            (I32Token(6), 0.30),
            (I32Token(7), 0.35),
            (I32Token(8), 0.40),
            (I32Token(9), 0.45),
            (I32Token(18), 0.90),
            (I32Token(19), 0.95),
            (I32Token(20), 1.00),
        ];
        let mut left: Vec<(I32Token, f64)> = vec![
            (I32Token(1), 0.05),
            (I32Token(2), 0.10),
            (I32Token(3), 0.15),
            (I32Token(4), 0.20),
            (I32Token(5), 0.25),
            (I32Token(6), 0.30),
            (I32Token(7), 0.35),
            (I32Token(8), 0.40),
            (I32Token(9), 0.45),
        ];
        let mut right: Vec<(I32Token, f64)> = vec![
            (I32Token(18), 0.90),
            (I32Token(19), 0.95),
            (I32Token(20), 1.00),
        ];
        assert_eq!(
            Window::new(&mut data).refine(),
            Refinement::Split(Window::new(&mut left), Window::new(&mut right))
        );
    }

    #[test]
    fn huge_left_heavy_input() {
        let mut data: Vec<(I32Token, f64)> = vec![(I32Token(1), 0.45), (I32Token(2), 0.60)];
        for i in 1..40001 {
            data.push((I32Token(2 + i), 0.60 + (0.40 / 40000.0) * i as f64));
        }
        if let Refinement::Split(left, _) = Window::new(&mut data).refine() {
            assert_eq!(left.0.len(), 1);
        } else {
            panic!("refinement should be a Split");
        }
    }

    #[test]
    fn huge_right_heavy_input() {
        let mut data: Vec<(I32Token, f64)> = Vec::with_capacity(40005);
        for i in 1..40001 {
            data.push((I32Token(2 + i), (0.40 / 40000.0) * i as f64));
        }
        data.push((I32Token(1), 0.45));
        data.push((I32Token(2), 1.0));
        if let Refinement::Split(_, right) = Window::new(&mut data).refine() {
            assert_eq!(right.0.len(), 1);
        } else {
            panic!("refinement should be a Split");
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

    #[test]
    fn single_level() {
        testing::init_logs_for_test();

        let m = model::with_frequencies(&[(I32Token(1), 4), (I32Token(2), 3)]);
        let t = new(m).unwrap();
        assert_eq!(t.alphabet().len(), 2);
        let want = crate::encoding::from_pairs(&[
            (I32Token(1), Letter::new(&[0b1000_0000], 2)),
            (I32Token(2), Letter::new(&[0b1100_0000], 2)),
        ])
        .unwrap();
        assert_eq!(t, want);
    }

    #[test]
    fn two_level_balanced() {
        testing::init_logs_for_test();

        let m = model::with_frequencies(&[
            (I32Token(1), 31),
            (I32Token(2), 29),
            (I32Token(3), 25),
            (I32Token(4), 15),
        ]);
        let t = new(m).unwrap();
        assert_eq!(t.alphabet().len(), 4);
        let want = crate::encoding::from_pairs(&[
            (I32Token(1), Letter::new(&[0b1000_0000], 3)),
            (I32Token(2), Letter::new(&[0b1010_0000], 3)),
            (I32Token(3), Letter::new(&[0b1100_0000], 3)),
            (I32Token(4), Letter::new(&[0b1110_0000], 3)),
        ])
        .unwrap();
        assert_eq!(t, want);
    }
}
