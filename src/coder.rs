use crate::code::Letter;
use crate::tokens::Token;
use anyhow::{anyhow, Result};
use std::collections::HashMap;

pub fn encode<'a, T, TS>(
    encoding: &'a HashMap<T, Letter>,
    input: TS,
) -> impl Iterator<Item = Result<&'a Letter>>
where
    T: Token,
    TS: std::iter::Iterator<Item = T>,
{
    input.map(move |t| match encoding.get(&t) {
        Some(l) => Ok(l),
        None => Err(anyhow!("Unknown token {}", t.to_string())),
    })
}

pub fn decode<'a, T, CS: 'a>(
    encoding: &'a HashMap<Letter, T>,
    input: CS,
) -> impl Iterator<Item = Result<T>> + 'a
where
    T: Token,
    CS: std::iter::Iterator<Item = &'a Letter>,
{
    input.map(move |l| match encoding.get(l) {
        Some(t) => Ok((*t).clone()),
        None => Err(anyhow!("no encoding for letter {}", l)),
    })
}
