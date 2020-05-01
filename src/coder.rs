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
