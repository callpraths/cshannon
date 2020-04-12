#![feature(associated_type_bounds)]

pub mod code;
pub mod coder;
pub mod model;
pub mod tokens;

use tokens::bytes::Bytes;
use tokens::graphemes::Graphemes;
use tokens::words::Words;
use tokens::{Token, Tokens, TokensPacker, TokensUnpacker};

use std::fs::{self, File};

pub fn run(args: clap::ArgMatches) -> Result<(), String> {
    // Safe to use unwrap() because these args are `required`.
    let input_file = args.value_of("input_file").unwrap();
    let output_file = args.value_of("output_file").unwrap();
    let tokenizer_choice = args.value_of("tokenizer").unwrap();

    let input = fs::read_to_string(input_file).map_err(|e| e.to_string())?;
    let output = match args.subcommand_name() {
        Some("compress") => match tokenizer_choice {
            "byte" => compress::<Bytes>(&input),
            "grapheme" => compress::<Graphemes>(&input),
            "word" => compress::<Words>(&input),
            _ => Err(format!("invalid tokenizer {}", tokenizer_choice)),
        },
        Some("decompress") => decompress(&input),
        _ => Err("no sub-command selected".to_string()),
    }?;
    fs::write(output_file, output).map_err(|e| e.to_string())
}

fn compress<'a, T: Tokens<'a>>(input: &'a str) -> Result<String, String> {
    println!("Compressing...");
    T::from_text(input).to_text()
}

fn decompress(input: &str) -> Result<String, String> {
    println!("Decompressing...");
    Ok(input.to_string())
}

fn read_write<'a, T, S, D>(input_file: &'a mut File, output_file: &mut File) -> Result<(), String>
where
    T: Token,
    S: TokensPacker<T = T>,
    D: TokensUnpacker<'a, File, T = T>,
{
    let d = D::unpack(input_file);
    S::pack(
        d.map(|r| match r {
            Ok(t) => t,
            Err(e) => panic!(e),
        }),
        output_file,
    )?;
    Ok(())
}
