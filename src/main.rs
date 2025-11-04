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

extern crate cshannon;

use anyhow::Result;
use clap::{Parser, Subcommand};
use cshannon::{encoding::EncodingScheme, Command, CompressArgs, DecompressArgs};
use env_logger::Env;

#[derive(Parser)]
#[command(version, about, long_about=None, author)]
#[command(propagate_version = true)]
struct Cli {
    /// Toknizer to use. Must be one of `byte`, `grapheme`, `word`.
    #[arg(short, long)]
    tokenizer: String,
    /// Encoding to use. Must be one of `balanced_tree`, `shannon`.
    #[arg(short, long)]
    encoding: String,
    /// Input file to (de)compress.
    #[arg(short, long)]
    input_file: String,
    /// Output file to (de)compress into.
    #[arg(short, long)]
    output_file: String,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Compress a file.
    Compress,
    /// Decompress a file.
    Decompress,
}

fn main() -> Result<()> {
    env_logger::Builder::from_env(Env::default().default_filter_or("warn")).init();
    let cli = Cli::parse();
    let command = match &cli.command {
        Commands::Compress => Command::Compress(CompressArgs {
            encoding_scheme: to_encoding_scheme(&cli.encoding),
        }),
        Commands::Decompress => Command::Decompress(DecompressArgs {}),
    };

    // Safe to use unwrap() because these args are `required`.
    cshannon::run(cshannon::Args {
        command,
        input_file: &cli.input_file,
        output_file: &cli.output_file,
        tokenizer: &cli.tokenizer,
    })?;
    println!("Success");
    Ok(())
}

// Migration kludge
fn to_encoding_scheme(encoding: &str) -> EncodingScheme {
    match encoding {
        "balanced_tree" => EncodingScheme::BalancedTree,
        "fano" => EncodingScheme::Fano,
        "shannon" => EncodingScheme::Shannon,
        "huffman" => EncodingScheme::Huffman,
        _ => panic!("Unsupported encoding scheme {}", encoding),
    }
}
