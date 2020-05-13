#![no_main]
use anyhow::Result;
use cshannon::{run, Args};
use libfuzzer_sys::fuzz_target;
use std::fs;
use std::sync::Once;
use tempfile;

fuzz_target!(|data: &[u8]| {
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
        tokenizer: "byte",
        encoding: "balanced_tree",
    }));
    print_error_and_bail(run(Args {
        command: "decompress",
        input_file: compressed_file.to_str().unwrap(),
        output_file: decompressed_file.to_str().unwrap(),
        tokenizer: "byte",
        encoding: "balanced_tree",
    }));
    let decompressed = fs::read(&decompressed_file).unwrap();
    assert_eq!(data, &decompressed[..]);
});

fn print_error_and_bail<T>(r: Result<T>) {
    if let Err(e) = r {
        format!("Error: {}", e);
        format!("Backtrace: {}", e.backtrace());
        panic!("Error: {}", e);
    }
}

static LOG_INIT: Once = Once::new();

pub fn init_logs_for_test() {
    LOG_INIT.call_once(|| env_logger::init());
}
