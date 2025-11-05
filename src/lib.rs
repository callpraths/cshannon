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
#![feature(test)]

//! A library of some early compression algorithms based on replacement schemes.
//!
//! This library implements the standard [Huffman coding] scheme and two
//! precursors to the Huffman scheme often called [Shannon-Fano coding].
//!
//! [Huffman coding]: https://en.wikipedia.org/wiki/Huffman_coding
//! [Shannon-Fano coding]: https://en.wikipedia.org/wiki/Shannon%E2%80%93Fano_coding
//!
//! # Usage
//!
//! cshannon provides a binary that can be used for compression / decompression
//! at the command line and a library that can be integrated into other projects.
//!
//! Run `cshannon --help` to see the command-line options for the binary.
//!
//! The easiest way to use cshannon library is:
//! ```
//! use cshannon::{Args, Command, CompressArgs, EncodingScheme, TokenizationScheme, run};
//! use std::path::Path;
//!
//! run(Args{
//!     command: Command::Compress(CompressArgs{
//!         encoding_scheme: EncodingScheme::Fano
//!     }),
//!     input_file: &Path::new("/path/to/input_file"),
//!     output_file: &Path::new("/path/to/output_file"),
//!     tokenization_scheme: TokenizationScheme::Byte,
//! });
//! ```
//!
//! # Crate layout
//!
//! The abstract steps in compression are as follows:
//!
//! ```ascii-art
//! Input --> Tokens --> Model --> Encoding -+
//!   |                                      |
//!   +-----> Tokens ------------------------+--> Compressed
//!                                                 Output
//! ```
//!
//! Different modules in the crate correspond to each of these steps.
//!
//! - The [tokens] module provides traits for tokenizing text. Three concrete
//!   tokenization schemes are implemented: [tokens::bytes], [tokens::graphemes]
//!   and [tokens::words].
//! - The [model] module provides a way to compute a zeroeth order model from a
//!   stream of tokens.
//! - The [encoding] module provides traits for creating an encoding scheme from
//!   a model. Four concrete encoding schemes are implemented:
//!   [encoding::balanced_tree], [encoding::shannon], [encoding::fano] and
//!   [encoding::huffman].
//! - Finally, the [code] module provides methods to encode a token stream given
//!   an encoding. The encoding itself is also included in the compressed
//!   output.
//!
//! The abstract steps for decompression are as follows:
//!
//! ```ascii-art
//! Compressed --> extract prefix --> Encoding
//!   Input                              |
//!    |                                 |
//!    +--> remaining data --------------+--> Output
//! ```
//!
//! Decompression is conceptually simpler because there are no choices (of
//! tokenizer and encoding). The encoding is included as a prefix in-band in the
//! compressed data. Most of the decompression logic resides in the [code]
//! module.

pub mod code;
pub mod model;
pub mod tokens;

mod encoding;
mod tokenization_scheme;
mod util;

pub use crate::encoding::EncodingScheme;
use crate::encoding::{new_encoder, Encoding};
pub use crate::tokenization_scheme::TokenizationScheme;
use crate::tokenization_scheme::{pack_tokenization_scheme, unpack_tokenization_scheme};

use code::Letter;
use tokens::bytes::Byte;
use tokens::graphemes::Grapheme;
use tokens::words::Word;
use tokens::{Token, TokenPacker, Tokenizer};

use anyhow::{anyhow, Result};
use log::info;
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, BufWriter};
use std::path::Path;

pub enum Command {
    Compress(CompressArgs),
    Decompress(DecompressArgs),
}

pub struct CompressArgs {
    pub encoding_scheme: EncodingScheme,
}

pub struct DecompressArgs {}

pub struct Args<'a> {
    pub command: Command,
    pub input_file: &'a Path,
    pub output_file: &'a Path,
    pub tokenization_scheme: TokenizationScheme,
}

pub fn run(args: Args) -> Result<()> {
    match args.command {
        Command::Compress(command_args) => compress(
            args.input_file,
            args.output_file,
            command_args.encoding_scheme,
            args.tokenization_scheme,
        ),
        Command::Decompress(_) => decompress(args.input_file, args.output_file),
    }
}

/// Document me.
/// TODO: Convert to use AsRef<Path>
pub fn compress(
    input_file: &Path,
    output_file: &Path,
    encoding_scheme: EncodingScheme,
    tokenization_scheme: TokenizationScheme,
) -> Result<()> {
    info!("Compressing...");

    let mut w = BufWriter::new(File::create(output_file)?);
    pack_tokenization_scheme(tokenization_scheme, &mut w)?;

    match tokenization_scheme {
        TokenizationScheme::Byte => compress_with_token::<Byte, _>(input_file, w, encoding_scheme),
        TokenizationScheme::Grapheme => {
            compress_with_token::<Grapheme, _>(input_file, w, encoding_scheme)
        }
        TokenizationScheme::Word => compress_with_token::<Word, _>(input_file, w, encoding_scheme),
    }
}

pub fn compress_with_token<T: Token, W: std::io::Write>(
    input_file: &Path,
    mut w: W,
    encoding_scheme: EncodingScheme,
) -> Result<()> {
    info!("Compressing...");
    let r = BufReader::new(File::open(input_file)?);
    let tokens = T::Tokenizer::tokenize(r).unwrap().map(|r| r.unwrap());
    let encoding = new_encoder(&&encoding_scheme, model::from(tokens))?;

    let r = BufReader::new(File::open(input_file)?);
    let tokens = T::Tokenizer::tokenize(r).unwrap().map(|r| r.unwrap());
    let code_text = encode(encoding.map(), tokens).map(|r| r.unwrap());

    encoding.pack(&mut w)?;
    crate::code::pack(code_text, &mut w)?;
    Ok(())
}

pub fn decompress(input_file: &Path, output_file: &Path) -> Result<()> {
    info!("Decompressing...");
    let w = BufWriter::new(File::create(output_file)?);
    let mut r = BufReader::new(File::open(input_file)?);
    match unpack_tokenization_scheme(&mut r)? {
        TokenizationScheme::Byte => decompress_with_token::<Byte, _, _>(r, w),
        TokenizationScheme::Grapheme => decompress_with_token::<Grapheme, _, _>(r, w),
        TokenizationScheme::Word => decompress_with_token::<Word, _, _>(r, w),
    }
}

fn decompress_with_token<T: Token, R: std::io::Read, W: std::io::Write>(
    mut r: R,
    mut w: W,
) -> Result<()> {
    let encoding: Encoding<T> = Encoding::unpack(&mut r).unwrap();
    let map = encoding.reverse_map();
    let coded_text = crate::code::parse(&encoding.alphabet(), r)?.map(|r| r.unwrap());
    let decoded_text = decode(&map, coded_text).map(|r| r.unwrap());
    T::Packer::pack(decoded_text, &mut w)?;
    Ok(())
}

fn encode<'a, T, TS>(
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

fn decode<'a, T, CS: 'a>(
    encoding: &'a HashMap<&'a Letter, &'a T>,
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

mod benchmarks {
    // Benchmarks don't get detected as uses correctly.
    #![allow(dead_code)]
    #![allow(unused_imports)]

    extern crate test;

    use super::*;
    use crate::util::testing;
    use anyhow::Result;
    use std::fs;
    use std::sync::Once;
    use test::Bencher;

    const TEXT: &str = "
Ah! well a-day! what evil looks
Had I from old and young!
Instead of the cross, the Albatross
About my neck was hung.
";

    #[bench]
    fn bytes_balanced_tree(b: &mut Bencher) {
        b.iter(|| roundtrip(TokenizationScheme::Byte, EncodingScheme::BalancedTree, TEXT));
    }

    #[bench]
    fn bytes_shannon(b: &mut Bencher) {
        // TODO(FIXME): Should be EncodingScheme::Shannon
        b.iter(|| roundtrip(TokenizationScheme::Byte, EncodingScheme::BalancedTree, TEXT));
    }

    #[bench]
    fn bytes_fano(b: &mut Bencher) {
        // TODO(FIXME): Should be EncodingScheme::Fano
        b.iter(|| roundtrip(TokenizationScheme::Byte, EncodingScheme::BalancedTree, TEXT));
    }

    #[bench]
    fn bytes_huffman(b: &mut Bencher) {
        // TODO(FIXME): Should be EncodingScheme::Huffman
        b.iter(|| roundtrip(TokenizationScheme::Byte, EncodingScheme::BalancedTree, TEXT));
    }

    fn roundtrip(
        tokenization_scheme: TokenizationScheme,
        encoding_scheme: EncodingScheme,
        data: &str,
    ) {
        testing::init_logs_for_test();
        let work_dir = tempfile::tempdir().unwrap();
        let input_file = work_dir.path().join("input.txt");
        let compressed_file = work_dir.path().join("compressed.txt");
        let decompressed_file = work_dir.path().join("decompressed.txt");

        fs::write(&input_file, data).unwrap();
        print_error_and_bail(run(Args {
            command: Command::Compress(CompressArgs { encoding_scheme }),
            input_file: &input_file.as_path(),
            output_file: &compressed_file.as_path(),
            tokenization_scheme,
        }));
        print_error_and_bail(run(Args {
            command: Command::Decompress(DecompressArgs {}),
            input_file: &compressed_file.as_path(),
            output_file: &decompressed_file.as_path(),
            tokenization_scheme,
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
}
