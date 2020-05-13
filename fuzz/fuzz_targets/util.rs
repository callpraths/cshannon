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

use anyhow::Result;
use cshannon::{run, Args};
use std::fs;
use std::sync::Once;
use tempfile;

pub fn fuzz(tokenizer: &str, encoding: &str, data: &[u8]) {
    init_logs_for_test();
    let work_dir = tempfile::tempdir().unwrap();
    let input_file = work_dir.path().join("input.txt");
    let compressed_file = work_dir.path().join("compressed.txt");
    let decompressed_file = work_dir.path().join("decompressed.txt");

    fs::write(&input_file, data).unwrap();
    print_error_and_bail(run(Args {
        command: "compress",
        input_file: input_file.to_str().unwrap(),
        output_file: compressed_file.to_str().unwrap(),
        tokenizer: tokenizer,
        encoding: encoding,
    }));
    print_error_and_bail(run(Args {
        command: "decompress",
        input_file: compressed_file.to_str().unwrap(),
        output_file: decompressed_file.to_str().unwrap(),
        tokenizer: tokenizer,
        encoding: encoding,
    }));
    let decompressed = fs::read(&decompressed_file).unwrap();
    assert_eq!(data, &decompressed[..]);
}

fn print_error_and_bail<T>(r: Result<T>) {
    if let Err(e) = r {
        format!("Error: {}", e);
        format!("Backtrace: {}", e.backtrace());
        panic!("Error: {}", e);
    }
}

static LOG_INIT: Once = Once::new();

fn init_logs_for_test() {
    LOG_INIT.call_once(|| env_logger::init());
}
