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

//! This is a minimal binary built using the exported cshannon library from this
//! crate.
//!
//! This binary will panic upon execution, because it does not properly setup
//! the necessary environment for the library (in partiular, no logger is
//! initialized). That is expected.
//!
//! The intent of this binary is to provide a minimal target to aid build size
//! optimization efforts for the library.

extern crate cshannon;

use std::path::Path;

fn main() {
    cshannon::run(cshannon::Args {
        command: cshannon::Command::Compress(cshannon::CompressArgs {
            encoding_scheme: cshannon::EncodingScheme::BalancedTree,
            tokenization_scheme: cshannon::TokenizationScheme::Grapheme,
        }),
        input_file: &Path::new("/tmp/non-existent-input-file"),
        output_file: &Path::new("/tmp/non-existent-output-file"),
    })
    .unwrap();
    cshannon::run(cshannon::Args {
        command: cshannon::Command::Decompress(cshannon::DecompressArgs {}),
        input_file: &Path::new("/tmp/non-existent-input-file"),
        output_file: &Path::new("/tmp/non-existent-output-file"),
    })
    .unwrap();
}
