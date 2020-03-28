use bit_vec::BitVec;
use std::slice::Iter;

pub struct Letter(BitVec);

impl ToString for Letter {
    fn to_string(&self) -> String {
        let mut s = String::with_capacity(self.0.len());
        for v in self.0.iter() {
            if v {
                s.push_str("1")
            } else {
                s.push_str("0")
            }
        }
        s
    }
}

pub struct Text(Vec<Letter>);

impl<'a> IntoIterator for &'a Text {
    type Item = &'a Letter;
    type IntoIter = Iter<'a, Letter>;

    fn into_iter(self) -> Self::IntoIter {
        (&self.0).into_iter()
    }
}
