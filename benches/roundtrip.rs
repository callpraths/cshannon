// Benchmarks don't get detected as uses correctly.
#![allow(dead_code)]
#![allow(unused_imports)]

use anyhow::Result;
use criterion::{criterion_group, criterion_main, Criterion};
use cshannon::{
    run, Args, Command, CompressArgs, DecompressArgs, EncodingScheme, TokenizationScheme,
};
use env_logger;
use std::fs;
use std::sync::Once;

const TEXT: &str = "
Ah! well a-day! what evil looks
Had I from old and young!
Instead of the cross, the Albatross
About my neck was hung.
";

fn bytes_balanced_tree(c: &mut Criterion) {
    c.bench_function("bytes_balanced_tree", |b| {
        b.iter(|| roundtrip(TokenizationScheme::Byte, EncodingScheme::BalancedTree, TEXT))
    });
}

fn bytes_fano(c: &mut Criterion) {
    c.bench_function("bytes_fano", |b| {
        b.iter(|| roundtrip(TokenizationScheme::Byte, EncodingScheme::Fano, TEXT))
    });
}

fn bytes_shannon(c: &mut Criterion) {
    c.bench_function("bytes_shannon", |b| {
        b.iter(|| roundtrip(TokenizationScheme::Byte, EncodingScheme::Shannon, TEXT))
    });
}

fn bytes_huffman(c: &mut Criterion) {
    c.bench_function("bytes_huffman", |b| {
        b.iter(|| roundtrip(TokenizationScheme::Byte, EncodingScheme::Huffman, TEXT))
    });
}

criterion_group!(
    benches,
    bytes_balanced_tree,
    bytes_fano,
    bytes_shannon,
    bytes_huffman
);
criterion_main!(benches);

fn roundtrip(tokenization_scheme: TokenizationScheme, encoding_scheme: EncodingScheme, data: &str) {
    init_logs_for_test();
    let work_dir = tempfile::tempdir().unwrap();
    let input_file = work_dir.path().join("input.txt");
    let compressed_file = work_dir.path().join("compressed.txt");
    let decompressed_file = work_dir.path().join("decompressed.txt");

    fs::write(&input_file, data).unwrap();
    print_error_and_bail(run(Args {
        command: Command::Compress(CompressArgs {
            tokenization_scheme,
            encoding_scheme,
        }),
        input_file: &input_file.as_path(),
        output_file: &compressed_file.as_path(),
    }));
    print_error_and_bail(run(Args {
        command: Command::Decompress(DecompressArgs {}),
        input_file: &compressed_file.as_path(),
        output_file: &decompressed_file.as_path(),
    }));
    let decompressed = fs::read(&decompressed_file).unwrap();
    assert_eq!(data.as_bytes(), &decompressed[..]);
}

fn print_error_and_bail<T>(r: Result<T>) {
    if let Err(e) = r {
        eprintln!("Backtrace: {}", e.backtrace());
        panic!("Error: {}", e);
    }
}

static LOG_INIT: Once = Once::new();

pub fn init_logs_for_test() {
    LOG_INIT.call_once(|| env_logger::init());
}
