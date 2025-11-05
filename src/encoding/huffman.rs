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

//! Create a new [Huffman encoding].
//!
//! [Huffman encoding]: https://en.wikipedia.org/wiki/Huffman_coding

use super::Encoding;
use crate::code::Letter;
use crate::model::Model;
use crate::tokens::Token;
use anyhow::Result;
use std::cell::RefCell;
use std::cmp::{Eq, Ord, Ordering, PartialEq, PartialOrd, Reverse};
use std::collections::BinaryHeap;
use std::collections::HashMap;
use std::rc::Rc;

/// Create a new Huffman encoding.
///
/// See [package documentation] for details.
///
/// [package documentation]: index.html
pub fn new<T: Token>(m: Model<T>) -> Result<Encoding<T>> {
    if m.is_empty() {
        return Ok(Encoding::new(HashMap::new())?);
    }

    let leaves = build_tree(&m);
    Ok(Encoding::new(
        m.tokens_sorted()
            .into_iter()
            .map(|t| {
                let l = read_letter(leaves.get(&t).unwrap().clone());
                (t, l)
            })
            .collect(),
    )?)
}

fn read_letter(leaf: Node) -> Letter {
    let mut path = Vec::new();
    let mut node = leaf;
    loop {
        match node.parent() {
            None => {
                break;
            }
            Some(p) => {
                path.push(p.clone());
                node = p.into_node();
            }
        }
    }

    let mut letter = Letter::with_capacity(path.len() as u64);
    let mut saw_one = false;
    path.into_iter().rev().for_each(|p| match p {
        Parent::Zero(_) => letter.push0(),
        Parent::One(_) => {
            saw_one = true;
            letter.push1();
        }
    });
    if !saw_one {
        // An all-zeroes letter is not allowed.
        letter.push1();
    }
    letter
}

fn build_tree<T: Token>(m: &Model<T>) -> HashMap<T, Node> {
    let leaves = init_leaves(m);
    let mut pq = init_pq(&leaves);
    loop {
        let zero = match pq.pop() {
            None => panic!("priority queue should never be empty!"),
            Some(Reverse(n)) => n,
        };
        let one = match pq.pop() {
            None => {
                return leaves;
            }
            Some(Reverse(n)) => n,
        };
        pq.push(Reverse(Node::with_children(&zero, &one)));
    }
}

fn init_leaves<T: Token>(m: &Model<T>) -> HashMap<T, Node> {
    let mut leaves = HashMap::new();
    m.tokens_sorted().into_iter().for_each(|t| {
        let value = m.frequency(&t);
        leaves.insert(t, Node::new(value));
    });
    leaves
}

fn init_pq<T: Token>(leaves: &HashMap<T, Node>) -> BinaryHeap<Reverse<Node>> {
    let mut pq = BinaryHeap::new();
    leaves.values().for_each(|n| pq.push(Reverse(n.clone())));
    pq
}

#[derive(Clone, Ord, PartialOrd, Eq, PartialEq)]
struct Node(Rc<RefCell<ConcreteNode>>);

impl Node {
    pub fn new(value: u64) -> Self {
        Self(Rc::new(RefCell::new(ConcreteNode {
            parent: None,
            value,
            height: 0,
        })))
    }

    pub fn with_children(zero: &Node, one: &Node) -> Self {
        let node = Node::new(zero.value() + one.value());
        node.set0(&zero);
        node.set1(&one);
        node
    }

    pub fn parent(&self) -> Option<Parent> {
        self.0.borrow().parent.as_ref().cloned()
    }

    fn set0(&self, zero: &Node) {
        let mut inner = zero.0.borrow_mut();
        assert!(inner.parent.is_none());
        inner.parent = Some(Parent::Zero(Node(self.0.clone())));
        self.maybe_bump_height(inner.height)
    }

    fn set1(&self, one: &Node) {
        let mut inner = one.0.borrow_mut();
        assert!(inner.parent.is_none());
        inner.parent = Some(Parent::One(Node(self.0.clone())));
        self.maybe_bump_height(inner.height)
    }

    fn maybe_bump_height(&self, child_height: usize) {
        let mut inner = self.0.borrow_mut();
        if inner.height <= child_height {
            inner.height = child_height + 1;
        }
    }

    fn value(&self) -> u64 {
        self.0.borrow().value
    }
}

struct ConcreteNode {
    parent: Option<Parent>,
    value: u64,
    height: usize,
}

#[derive(Clone)]
enum Parent {
    Zero(Node),
    One(Node),
}

impl Parent {
    pub fn into_node(self) -> Node {
        match self {
            Parent::Zero(n) => n,
            Parent::One(n) => n,
        }
    }
}

impl PartialEq for ConcreteNode {
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value && self.height == other.height
    }
}

impl Eq for ConcreteNode {}

impl Ord for ConcreteNode {
    fn cmp(&self, other: &Self) -> Ordering {
        match self.value.cmp(&other.value) {
            // smaller value && smaller height is good. So the Ordering is
            // consistent between the two fields.
            Ordering::Equal => self.height.cmp(&other.height),
            o => o,
        }
    }
}

impl PartialOrd for ConcreteNode {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[cfg(test)]
mod tests {
    // Temporary
    #![allow(unused_imports)]

    use super::*;
    use crate::code::Letter;
    use crate::model;
    use crate::tokens::test_utils::I32Token;
    use crate::util::testing;
    use std::collections::HashMap;

    #[test]
    fn empty() {
        assert!(new(model::with_frequencies::<I32Token>(&[]))
            .unwrap()
            .alphabet()
            .is_empty());
    }

    #[test]
    fn two_tokens() {
        testing::init_logs_for_test();
        let m = model::with_frequencies(&[(I32Token(1), 4), (I32Token(2), 3)]);
        let t = new(m).unwrap();
        assert_eq!(t.alphabet().len(), 2);
        let want = crate::encoding::from_pairs(&[
            (I32Token(1), Letter::new(&[0b1000_0000], 1)),
            (I32Token(2), Letter::new(&[0b0100_0000], 2)),
        ])
        .unwrap();
        assert_eq!(t, want);
    }

    #[test]
    fn three_tokens_huge_first() {
        testing::init_logs_for_test();
        let m = model::with_frequencies(&[(I32Token(1), 40), (I32Token(2), 3), (I32Token(3), 2)]);
        let t = new(m).unwrap();
        assert_eq!(t.alphabet().len(), 3);
        let want = crate::encoding::from_pairs(&[
            (I32Token(1), Letter::new(&[0b1000_0000], 1)),
            (I32Token(2), Letter::new(&[0b0100_0000], 2)),
            (I32Token(3), Letter::new(&[0b0010_0000], 3)),
        ])
        .unwrap();
        assert_eq!(t, want);
    }

    #[test]
    fn three_tokens_similar() {
        testing::init_logs_for_test();
        let m = model::with_frequencies(&[(I32Token(1), 4), (I32Token(2), 3), (I32Token(3), 2)]);
        let t = new(m).unwrap();
        assert_eq!(t.alphabet().len(), 3);
        let want = crate::encoding::from_pairs(&[
            (I32Token(1), Letter::new(&[0b0100_0000], 2)),
            (I32Token(2), Letter::new(&[0b1100_0000], 2)),
            (I32Token(3), Letter::new(&[0b1000_0000], 2)),
        ])
        .unwrap();
        assert_eq!(t, want);
    }

    #[test]
    fn four_tokens_balanced_tree() {
        testing::init_logs_for_test();
        let m = model::with_frequencies(&[
            (I32Token(1), 5),
            (I32Token(2), 4),
            (I32Token(3), 3),
            (I32Token(4), 2),
        ]);
        let t = new(m).unwrap();
        assert_eq!(t.alphabet().len(), 4);
        let want = crate::encoding::from_pairs(&[
            (I32Token(1), Letter::new(&[0b1100_0000], 2)),
            (I32Token(2), Letter::new(&[0b1000_0000], 2)),
            (I32Token(3), Letter::new(&[0b0100_0000], 2)),
            (I32Token(4), Letter::new(&[0b0010_0000], 3)),
        ])
        .unwrap();
        assert_eq!(t, want);
    }
}
