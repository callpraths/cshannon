extern crate test_bin;

use std::fs;
use tempfile;

const TEXT: &str = "
Ah! well a-day! what evil looks
Had I from old and young!
Instead of the cross, the Albatross
About my neck was hung.
";

#[test]
fn test_roundrip() {
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
            "decompress",
        ])
        .status()
        .is_ok());
    let decompressed_text = fs::read_to_string(&decompressed_file).unwrap();
    assert_eq!(TEXT, decompressed_text);
}
