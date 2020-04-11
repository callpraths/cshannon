use crate::code::{self, Letter};
use crate::tokens::Token;
use std::collections::HashMap;

pub fn encode<'a, T, TS>(
    encoding: &'a HashMap<T, Letter>,
    input: TS,
) -> impl Iterator<Item = &'a Letter>
where
    T: Token,
    TS: std::iter::Iterator<Item = T>,
{
    input.map(move |t| {
        if let Some(l) = encoding.get(&t) {
            l
        } else {
            panic!("Unknown token {}", t.to_string())
        }
    })
}

pub type Result<T> = std::result::Result<T, String>;

pub fn decode<'a, T, CS: 'a>(
    encoding: &'a HashMap<Letter, T>,
    input: CS,
) -> impl Iterator<Item = Result<&'a T>> + 'a
where
    T: Token,
    CS: std::iter::Iterator<Item = code::Result<&'a Letter>>,
{
    input.map(move |c| match c {
        Ok(l) => match encoding.get(l) {
            Some(t) => Ok(t),
            None => Err(format!("no encoding for letter {}", l)),
        },
        Err(e) => Err(e),
    })
}
