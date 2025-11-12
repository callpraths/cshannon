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

use std::path::PathBuf;

use anyhow::Result;
use clap::{Parser, Subcommand};
use cshannon::{Command, CompressArgs, DecompressArgs, EncodingScheme, TokenizationScheme};
use env_logger::Env;

#[derive(Parser)]
#[command(version, about, long_about=None, author)]
#[command(propagate_version = true)]
struct Cli {
    /// Input file to (de)compress.
    #[arg(short, long)]
    input_file: PathBuf,
    /// Output file to (de)compress into.
    #[arg(short, long)]
    output_file: PathBuf,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Compress a file.
    Compress {
        /// Toknizer to use.
        #[arg(short, long)]
        tokenization: TokenizationSchemeArg,
        /// Encoding to use.
        #[arg(short, long)]
        encoding: EncodingSchemeArg,
    },
    /// Decompress a file.
    Decompress,
}

#[derive(clap::ValueEnum, Clone, Debug)]
enum TokenizationSchemeArg {
    Byte,
    Word,
    Grapheme,
}

#[derive(clap::ValueEnum, Clone, Debug)]
enum EncodingSchemeArg {
    BalancedTree,
    Fano,
    Shannon,
    Huffman,
}

fn main() -> Result<()> {
    env_logger::Builder::from_env(Env::default().default_filter_or("warn")).init();
    let cli = Cli::parse();
    let command = match &cli.command {
        Commands::Compress {
            encoding,
            tokenization,
        } => Command::Compress(CompressArgs {
            encoding_scheme: to_encoding_scheme(&encoding),
            tokenization_scheme: to_tokenization_scheme(tokenization),
        }),
        Commands::Decompress => Command::Decompress(DecompressArgs {}),
    };

    // Safe to use unwrap() because these args are `required`.
    cshannon::run(cshannon::Args {
        command,
        input_file: &cli.input_file,
        output_file: &cli.output_file,
    })?;
    println!("Success");
    Ok(())
}

// Migration kludge
fn to_encoding_scheme(encoding: &EncodingSchemeArg) -> EncodingScheme {
    match encoding {
        EncodingSchemeArg::BalancedTree => EncodingScheme::BalancedTree,
        EncodingSchemeArg::Fano => EncodingScheme::Fano,
        EncodingSchemeArg::Shannon => EncodingScheme::Shannon,
        EncodingSchemeArg::Huffman => EncodingScheme::Huffman,
    }
}

fn to_tokenization_scheme(tokenization: &TokenizationSchemeArg) -> TokenizationScheme {
    match tokenization {
        TokenizationSchemeArg::Byte => TokenizationScheme::Byte,
        TokenizationSchemeArg::Grapheme => TokenizationScheme::Grapheme,
        TokenizationSchemeArg::Word => TokenizationScheme::Word,
    }
}
