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

extern crate test_bin;

use std::fs;
use tempfile;

const TEXT: &str = "
Ah! well a-day! what evil looks
Had I from old and young!
Instead of the cross, the Albatross
About my neck was hung.
";

const TEXT_ONLY_WORDS: &str = "\
Ah well a day what evil looks \
Had I from old and young \
Instead of the cross the Albatross \
About my neck was hung\
";

fn roundtrip(text: &str, token: &str, encoding: &str) {
    // We freely unwrap() here since this is a simplistic integration test.
    let work_dir = tempfile::tempdir().unwrap();
    let input_file = work_dir.path().join("input.txt");
    let compressed_file = work_dir.path().join("compressed.txt");
    let decompressed_file = work_dir.path().join("decompressed.txt");

    fs::write(&input_file, text).unwrap();
    assert!(test_bin::get_test_bin("cshannon")
        .args(&[
            "-i",
            input_file.to_str().unwrap(),
            "-o",
            compressed_file.to_str().unwrap(),
            "-t",
            token,
            "-e",
            encoding,
            "compress",
        ])
        .status()
        .is_ok());
    assert!(test_bin::get_test_bin("cshannon")
        .args(&[
            "-i",
            compressed_file.to_str().unwrap(),
            "-o",
            decompressed_file.to_str().unwrap(),
            "-t",
            token,
            "-e",
            encoding,
            "decompress",
        ])
        .status()
        .is_ok());
    let decompressed_text = fs::read_to_string(&decompressed_file).unwrap();
    assert_eq!(text, decompressed_text);
}

// Pushed a change for completeness, but needs follow up for fix(es).
#[test]
fn bytes_balanced_tree() {
    roundtrip(TEXT, "byte", "balanced_tree");
}

// roundtrip is currently broken.
// Pushed a change for completeness, but needs follow up for fix(es).
#[test]
fn graphemes_balanced_tree() {
    roundtrip(TEXT, "grapheme", "balanced_tree");
}

// roundtrip is currently broken.
// Pushed a change for completeness, but needs follow up for fix(es).
#[test]
fn words_balanced_tree() {
    roundtrip(TEXT_ONLY_WORDS, "word", "balanced_tree");
}

// Pushed a change for completeness, but needs follow up for fix(es).
#[test]
#[ignore]
fn bytes_shannon() {
    roundtrip(TEXT, "byte", "shannon");
}

// roundtrip is currently broken.
// Pushed a change for completeness, but needs follow up for fix(es).
#[test]
#[ignore]
fn graphemes_shannon() {
    roundtrip(TEXT, "grapheme", "shannon");
}

// roundtrip is currently broken.
// Pushed a change for completeness, but needs follow up for fix(es).
#[test]
#[ignore]
fn words_shannon() {
    roundtrip(TEXT_ONLY_WORDS, "word", "shannon");
}
