#![feature(associated_type_bounds)]

extern crate cshannon;

use anyhow::Result;
use clap::{App, Arg, SubCommand};

fn main() -> Result<()> {
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

    cshannon::run(args)?;
    println!("Success");
    Ok(())
}
