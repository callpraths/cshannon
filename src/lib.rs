#![feature(associated_type_bounds)]

pub mod code;
pub mod coder;
pub mod encoding;
pub mod model;
pub mod tokens;

use tokens::bytes::{Byte, ByteIter, BytePacker};
use tokens::graphemes::{Grapheme, GraphemeIter, GraphemePacker};
use tokens::words::{Word, WordIter, WordPacker};
use tokens::{Token, TokenIter, TokenPacker};

use std::fs::{self, File};

pub fn run(args: clap::ArgMatches) -> Result<(), String> {
    // Safe to use unwrap() because these args are `required`.
    let input_file = args.value_of("input_file").unwrap();
    let output_file = args.value_of("output_file").unwrap();
    let tokenizer_choice = args.value_of("tokenizer").unwrap();

    match args.subcommand_name() {
        Some("compress") => match tokenizer_choice {
            "byte" => compress::<Byte, ByteIter<File>, BytePacker>(&input_file, &output_file),
            "grapheme" => {
                compress::<Grapheme, GraphemeIter, GraphemePacker>(&input_file, &output_file)
            }
            "word" => compress::<Word, WordIter, WordPacker>(&input_file, &output_file),
            _ => Err(format!("invalid tokenizer {}", tokenizer_choice)),
        },
        Some("decompress") => decompress(&input_file, &output_file),
        _ => Err("no sub-command selected".to_string()),
    }
}

fn compress<T, TIter, TPacker>(input_file: &str, output_file: &str) -> Result<(), String>
where
    T: Token,
    TIter: TokenIter<File, T = T>,
    TPacker: TokenPacker<File, T = T>,
{
    println!("Compressing...");
    let r = File::open(input_file).map_err(|e| e.to_string())?;
    let mut w = File::create(output_file).map_err(|e| e.to_string())?;
    pack::<_, _, TPacker>(unpack::<T, TIter>(r).map(|r| r.unwrap()), &mut w)
}

fn decompress(input_file: &str, output_file: &str) -> Result<(), String> {
    println!("Decompressing...");
    fs::write(
        output_file,
        fs::read_to_string(input_file).map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())
}

fn unpack<T, TIter>(r: File) -> impl TokenIter<File, T = T>
where
    T: Token,
    TIter: TokenIter<File, T = T>,
{
    TIter::unpack(r)
}

fn pack<T, I, TPacker>(i: I, w: &mut File) -> Result<(), String>
where
    T: Token,
    I: std::iter::Iterator<Item = T>,
    TPacker: TokenPacker<File, T = T>,
{
    TPacker::pack(i, w)
}
