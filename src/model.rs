
use crate::tokenizer::generic::{Token, Tokens};

use std::collections::HashMap;

pub struct Model<T: Token> {
    fs: HashMap<T, i64>,
    ps: HashMap<T, f64>,
}

impl<T: Token> Model<T> {
    pub fn f(&self, t: T) -> i64 {
        match self.fs.get(&t) {
            Some(&f) => f,
            None => 0,
        }
    }

    pub fn p(&self, t: T) -> f64 {
        match self.ps.get(&t) {
            Some(&p) => p,
            None => 0.0,
        }
    }
}

pub fn build_from<'a, T, TS>(ts: TS) -> Model<T>
where T: Token, TS: Tokens<'a, Item=T> {
    let mut m = Model::<T>{
        fs: HashMap::new(),
        ps: HashMap::new(),
    };
    let mut d: i64 = 0;
    for t in ts {
        match m.fs.get(&t) {
            Some(&f) => m.fs.insert(t, f + 1),
            None => m.fs.insert(t, 1),
        };
        d = d + 1;
    }
    for (t, f) in m.fs.iter() {
        m.ps.insert(*t, (*f as f64)/(d as f64));
    }
    m
}