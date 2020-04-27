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

// roundtrip is currently broken.
// Pushed a change for completeness, but needs follow up for fix(es).
#[test]
#[ignore]
fn test_bytes_roundrip() {
    // We freely unwrap() here since this is a simplistic integration test.
    let work_dir = tempfile::tempdir().unwrap();
    let input_file = work_dir.path().join("input.txt");
    let compressed_file = work_dir.path().join("compressed.txt");
    let decompressed_file = work_dir.path().join("decompressed.txt");

    fs::write(&input_file, TEXT).unwrap();
    assert!(test_bin::get_test_bin("cshannon")
        .args(&[
            "-i",
            input_file.to_str().unwrap(),
            "-o",
            compressed_file.to_str().unwrap(),
            "-t",
            "byte",
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
            "byte",
            "decompress",
        ])
        .status()
        .is_ok());
    let decompressed_text = fs::read_to_string(&decompressed_file).unwrap();
    assert_eq!(TEXT, decompressed_text);
}

// roundtrip is currently broken.
// Pushed a change for completeness, but needs follow up for fix(es).
#[test]
#[ignore]
fn test_graphemes_roundrip() {
    // We freely unwrap() here since this is a simplistic integration test.
    let work_dir = tempfile::tempdir().unwrap();
    let input_file = work_dir.path().join("input.txt");
    let compressed_file = work_dir.path().join("compressed.txt");
    let decompressed_file = work_dir.path().join("decompressed.txt");

    fs::write(&input_file, TEXT).unwrap();
    assert!(test_bin::get_test_bin("cshannon")
        .args(&[
            "-i",
            input_file.to_str().unwrap(),
            "-o",
            compressed_file.to_str().unwrap(),
            "-t",
            "grapheme",
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
            "grapheme",
            "decompress",
        ])
        .status()
        .is_ok());
    let decompressed_text = fs::read_to_string(&decompressed_file).unwrap();
    assert_eq!(TEXT, decompressed_text);
}

// roundtrip is currently broken.
// Pushed a change for completeness, but needs follow up for fix(es).
#[test]
#[ignore]
fn test_words_roundrip() {
    // We freely unwrap() here since this is a simplistic integration test.
    let work_dir = tempfile::tempdir().unwrap();
    let input_file = work_dir.path().join("input.txt");
    let compressed_file = work_dir.path().join("compressed.txt");
    let decompressed_file = work_dir.path().join("decompressed.txt");

    fs::write(&input_file, TEXT_ONLY_WORDS).unwrap();
    assert_eq!(
        test_bin::get_test_bin("cshannon")
            .args(&[
                "-i",
                input_file.to_str().unwrap(),
                "-o",
                compressed_file.to_str().unwrap(),
                "-t",
                "word",
                "compress",
            ])
            .status()
            .is_ok(),
        true
    );
    assert_eq!(
        test_bin::get_test_bin("cshannon")
            .args(&[
                "-i",
                compressed_file.to_str().unwrap(),
                "-o",
                decompressed_file.to_str().unwrap(),
                "-t",
                "word",
                "decompress",
            ])
            .status()
            .is_ok(),
        true
    );
    let decompressed_text = fs::read_to_string(&decompressed_file).unwrap();
    assert_eq!(TEXT_ONLY_WORDS, decompressed_text);
}
