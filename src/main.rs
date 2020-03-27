#![feature(associated_type_bounds)]

extern crate clap;

use clap::{App, Arg, SubCommand};

use std::fs;

pub mod model;
pub mod tokenizer;

use tokenizer::bytes::Bytes;
use tokenizer::graphemes::Graphemes;
use tokenizer::words::Words;
use tokenizer::Tokens;

fn main() {
    let args = App::new("Shannon Coder-Decoder")
        .version("0.1.0")
        .author("Prathmesh Prabhu (callpraths@gmail.com")
        .about("Compress / Decompress text a la Shannon")
        .arg(
            Arg::with_name("tokenizer")
                .long("tokenizer")
                .short("t")
                .required(true)
                .help("Tokenizer to use")
                .long_help(
                    "Tokenizer to use.
Must be one of:
    byte:     Parse into bytes.
    grapheme: Parse into unicode grapheme clusters.
    word:     Parse into unicode words. This tokenizer is lossy.",
                )
                .takes_value(true),
        )
        .arg(
            Arg::with_name("input_file")
                .long("input-file")
                .short("i")
                .required(true)
                .help("Input file to (de)compress")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("output_file")
                .long("output-file")
                .short("o")
                .required(true)
                .help("Output file to (de)compress to")
                .long_help(
                    "Output file to (de)compress to.
Must be different from input file.",
                )
                .takes_value(true),
        )
        .subcommand(SubCommand::with_name("compress").about("Compress a file"))
        .subcommand(SubCommand::with_name("decompress").about("Decompress a file"))
        .get_matches();

    match inner_main(args) {
        Ok(()) => println!("Success"),
        Err(err) => panic!("Error: {}", err),
    }
}

fn inner_main(args: clap::ArgMatches) -> Result<(), String> {
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
