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

///! Contains Token implementations intended to help write unit tests.
use super::Token;
use std::fmt;

/// A `Token` that wraps i32 values.
///
/// Useful for unittests against the Token trait.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct I32Token(pub i32);

impl fmt::Display for I32Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl Token for I32Token {
    fn bit_count(&self) -> usize {
        4
    }
}
