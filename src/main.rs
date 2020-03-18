extern crate clap;

use clap::{App, Arg, SubCommand};

use std::fs;

fn main() {
    let args = App::new("Shannon Coder-Decoder")
        .version("0.1.0")
        .author("Prathmesh Prabhu (callpraths@gmail.com")
        .about("Compress / Decompress text a la Shannon")
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
        Err(err) => println!("Error: {}", err),
    }
}

fn inner_main(args: clap::ArgMatches) -> Result<(), String> {
    // Safe to use unwrap() because these args are `required`.
    let input_file = args.value_of("input_file").unwrap();
    let output_file = args.value_of("output_file").unwrap();

    let input = fs::read_to_string(input_file).map_err(|e| e.to_string())?;
    let output = match args.subcommand_name() {
        Some("compress") => compress(&input),
        Some("decompress") => decompress(&input),
        _ => Err("no sub-command selected".to_string()),
    }?;
    fs::write(output_file, output).map_err(|e| e.to_string())
}

fn compress(input: &str) -> Result<String, String> {
    println!("Compressing...");
    Ok(input.to_string())
}

fn decompress(input: &str) -> Result<String, String> {
    println!("Decompressing...");
    Ok(input.to_string())
}

pub mod tokenizer {
    pub mod generic {

        // Tokens must be usable as keys in std::collections::HashMap
        pub trait Token: ToString + Eq + std::hash::Hash {
            // The number of bits of source text contained in this Token.
            fn bit_count() -> usize;
        }

        // We'd like to be able to define a single fn pointer type that returns
        // an Iterator over some type that satisfies Token, but this is not
        // possible yet.
        // https://github.com/rust-lang/rfcs/blob/master/text/1522-conservative-impl-trait.md
        pub type Tokenizer<'a, I>
        where
            I: 'a,
            I: std::iter::Iterator,
            I::Item: Token,
        = fn(&'a str) -> I;
    }

    pub mod byte_stream {
        use std::fmt;
        use std::hash::Hash;

        use crate::tokenizer::generic::{Token, Tokenizer};

        #[derive(Debug, PartialEq, Eq, Hash)]
        pub struct Byte {}

        impl std::fmt::Display for Byte {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                Ok(())
            }
        }

        impl Token for Byte {
            fn bit_count() -> usize {
                0
            }
        }

        pub struct ByteStream<'a> {
            text: &'a str,
        }

        impl std::iter::Iterator for ByteStream<'_> {
            type Item = Byte;

            fn next(&mut self) -> Option<Self::Item> {
                None
            }
        }

        pub fn byte_tokenizer<'a>(text: &'a str) -> ByteStream<'a> {
            ByteStream { text }
        }

        // validate type.
        static TOKENIZER_FN: Tokenizer<ByteStream> = byte_tokenizer;
    }
}
