use std::collections::HashMap;

pub struct Model<T: Eq + std::hash::Hash>(HashMap<T, Stats>);

pub struct Stats {
    f: i64,
    p: f64,
}

impl<T: Eq + std::hash::Hash> Model<T> {
    pub fn f(&self, t: &T) -> i64 {
        match self.0.get(t) {
            Some(s) => s.f,
            None => 0,
        }
    }

    pub fn p(&self, t: &T) -> f64 {
        match self.0.get(t) {
            Some(s) => s.p,
            None => 0.0,
        }
    }
}

pub fn from<T, TS>(ts: TS) -> Model<T>
where
    T: Eq + std::hash::Hash,
    TS: std::iter::IntoIterator<Item = T>,
{
    let mut m = Model::<T>(HashMap::new());
    let mut d: i64 = 0;
    for t in ts {
        let s = m.0.entry(t).or_insert(Stats { f: 0, p: 0.0 });
        (*s).f += 1;
        d += 1;
    }
    for (_, s) in &mut m.0 {
        (*s).p = ((*s).f as f64) / (d as f64);
    }
    m
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic() {
        let tokens = vec![2, 3, 1, 2, 5, 11];
        let m = from(tokens);
        assert_eq!(m.f(&1), 1);
        assert_eq!(m.f(&2), 2);
        assert_eq!(m.f(&13), 0);

        // f64 equality is inexact.
        assert!(m.p(&5) > 0.166);
        assert!(m.p(&5) < 0.167);
    }
}
