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

//! Provides facilities to read & write data encoded with a [prefix code].
//!
//! A [`Letter`] holds a single code-point of the prefix code. The (ordered) set
//! of all code-points is an [`Alphabet`].
//!
//! The two main functions exported from this module are [`pack()`] (to pack a
//! stream of code-points into a buffer) and [`parse()`] (to unpack a stream of
//! code-points from a buffer, given the [`Alphabet`] of code-points).
//!
//! [prefix code]: https://en.wikipedia.org/wiki/Prefix_code

mod alphabet;
mod common;
mod letter;
mod text;

pub use alphabet::Alphabet;
pub use letter::Letter;
pub use text::{pack, parse};
