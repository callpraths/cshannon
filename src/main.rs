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
