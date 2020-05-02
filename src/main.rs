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
            Arg::with_name("encoding")
                .long("encoding")
                .short("e")
                .required(true)
                .help("Encoding to use")
                .long_help(
                    "Encoding to use.
Must be one of:
    balanced_tree: A fixed length encoding for all tokens.
    shannon:       Shannon's encoding scheme.",
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
