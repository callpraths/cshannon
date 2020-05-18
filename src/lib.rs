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

#![feature(associated_type_bounds)]
#![feature(seek_convenience)]
#![feature(test)]

pub mod code;
pub mod coder;
pub mod encoding;
pub mod model;
pub mod tokens;
mod util;

use code::Letter;
use encoding::Encoding;
use encoding::{balanced_tree, fano, huffman, shannon};
use model::Model;
use tokens::bytes::{Byte, ByteIter, BytePacker};
use tokens::graphemes::{Grapheme, GraphemeIter, GraphemePacker};
use tokens::words::{Word, WordIter, WordPacker};
use tokens::{Token, TokenIter, TokenPacker};

use anyhow::{anyhow, Result};
use log::{debug, info, log_enabled, trace, Level};
use std::collections::HashMap;
use std::fs::File;
use std::io::Seek;
use std::io::{BufReader, BufWriter, Cursor};

pub struct Args<'a> {
    pub command: &'a str,
    pub input_file: &'a str,
    pub output_file: &'a str,
    pub encoding: &'a str,
    pub tokenizer: &'a str,
}

pub fn run(args: Args) -> Result<()> {
    match args.command {
        "compress" => match args.tokenizer {
            "byte" => compress::<Byte, ByteIter<BufReader<File>>, BytePacker>(
                args.input_file,
                args.output_file,
                encoder(args.encoding)?,
            ),
            "grapheme" => compress::<Grapheme, GraphemeIter, GraphemePacker>(
                args.input_file,
                args.output_file,
                encoder(args.encoding)?,
            ),
            "word" => compress::<Word, WordIter, WordPacker>(
                args.input_file,
                args.output_file,
                encoder(args.encoding)?,
            ),
            _ => Err(anyhow!("invalid tokenizer {}", args.tokenizer)),
        },
        "decompress" => match args.tokenizer {
            "byte" => {
                decompress::<Byte, ByteIter<BufReader<File>>, ByteIter<Cursor<Vec<u8>>>, BytePacker>(
                    args.input_file,
                    args.output_file,
                )
            }
            "grapheme" => decompress::<Grapheme, GraphemeIter, GraphemeIter, GraphemePacker>(
                args.input_file,
                args.output_file,
            ),
            "word" => decompress::<Word, WordIter, WordIter, WordPacker>(
                args.input_file,
                args.output_file,
            ),
            _ => Err(anyhow!("invalid tokenizer {}", args.tokenizer)),
        },
        name => Err(anyhow!("unsupported command {}", name)),
    }
}

type Encoder<T> = fn(Model<T>) -> Result<Encoding<T>>;

fn encoder<T: Token>(encoding: &str) -> Result<Encoder<T>> {
    match encoding {
        "balanced_tree" => Ok(balanced_tree::new::<T>),
        "shannon" => Ok(shannon::new::<T>),
        "fano" => Ok(fano::new::<T>),
        "huffman" => Ok(huffman::new::<T>),
        _ => Err(anyhow!("invalid encoding {}", encoding)),
    }
}

/// Document me.
/// TODO: Convert to use AsRef<Path>
pub fn compress<T, TIter, TPacker>(
    input_file: &str,
    output_file: &str,
    encoder: Encoder<T>,
) -> Result<()>
where
    T: Token,
    TIter: TokenIter<BufReader<File>, T = T>,
    TPacker: TokenPacker<BufWriter<File>, T = T>,
{
    info!("Compressing...");
    let r = BufReader::new(File::open(input_file)?);
    let tokens = TIter::unpack(r).map(|r| r.unwrap());
    let encoding = encoder(model::from(tokens))?;

    let r = BufReader::new(File::open(input_file)?);
    let tokens = TIter::unpack(r).map(|r| r.unwrap());
    let code_text = coder::encode(encoding.map(), tokens).map(|r| r.unwrap());

    let mut w = BufWriter::new(File::create(output_file)?);
    crate::tokens::pack_all::<_, _, TPacker>(encoding.tokens(), &mut w)?;
    encoding.alphabet().clone().pack(&mut w)?;
    crate::code::pack(code_text, &mut w)?;
    Ok(())
}

/// Document me.
/// TODO: Convert to use AsRef<Path>
pub fn decompress<T, TIter, TAllIter, TPacker>(input_file: &str, output_file: &str) -> Result<()>
where
    T: Token,
    TIter: TokenIter<BufReader<File>, T = T>,
    TAllIter: TokenIter<Cursor<Vec<u8>>, T = T>,
    TPacker: TokenPacker<BufWriter<File>, T = T>,
{
    info!("Decompressing...");
    let mut r = File::open(input_file)?;
    trace!("File position at the start: {:?}", r.stream_position());
    let mut br = BufReader::new(r);
    let tokens = crate::tokens::unpack_all::<_, _, TAllIter>(&mut br)?;
    trace!(
        "File position after unpacking token set: {:?}",
        br.stream_position()
    );

    let alphabet = crate::code::Alphabet::unpack(&mut br)?;
    trace!(
        "File position after unpacking alphabet set: {:?}",
        br.stream_position()
    );
    let letters = alphabet.letters();

    if letters.len() != tokens.len() {
        return Err(anyhow!(
            "Extracted letter count {} does not match token count {}",
            letters.len(),
            tokens.len(),
        ));
    }
    let map = letters
        .iter()
        .cloned()
        .zip(tokens.into_iter())
        .collect::<HashMap<Letter, T>>();
    log_decoder_ring(&map);

    let coded_text = crate::code::parse(&alphabet, br)?.map(|r| r.unwrap());
    let tokens = crate::coder::decode(&map, coded_text).map(|r| r.unwrap());

    let mut w = BufWriter::new(File::create(output_file)?);
    TPacker::pack(tokens, &mut w)?;
    Ok(())
}

fn log_decoder_ring<T: Token>(m: &HashMap<Letter, T>) {
    if !log_enabled!(Level::Debug) {
        return;
    }
    debug!("Decoder ring:");
    for (l, t) in m.iter() {
        debug!("  |{}|: |{}|", l, t);
    }
}

mod benchmarks {
    // Benchmarks don't get detected as uses correctly.
    #![allow(dead_code)]
    #![allow(unused_imports)]

    extern crate test;

    use super::{run, Args};
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
        b.iter(|| roundtrip("byte", "balanced_tree", TEXT));
    }

    #[bench]
    fn bytes_shannon(b: &mut Bencher) {
        b.iter(|| roundtrip("byte", "balanced_tree", TEXT));
    }

    #[bench]
    fn bytes_fano(b: &mut Bencher) {
        b.iter(|| roundtrip("byte", "balanced_tree", TEXT));
    }

    #[bench]
    fn bytes_huffman(b: &mut Bencher) {
        b.iter(|| roundtrip("byte", "balanced_tree", TEXT));
    }

    fn roundtrip(tokenizer: &str, encoding: &str, data: &str) {
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
        assert_eq!(data.as_bytes(), &decompressed[..]);
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
}
