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

//! A library of some early compression algorithms based on replacement schemes.
//!
//! WARNING: This is a pet-project and does not intend to be a production-ready
//! library for data compression (e.g. no attempt is made to maintain at-rest
//! data format compatibility across library versions).
//!
//! This library implements the standard [Huffman coding] scheme, two
//! precursors to the Huffman scheme often called [Shannon-Fano coding], and
//! a simple fixed-width encoding that is easiest to understand, though not very
//! good at encoding information.
//!
//! [Huffman coding]: https://en.wikipedia.org/wiki/Huffman_coding
//! [Shannon-Fano coding]: https://en.wikipedia.org/wiki/Shannon%E2%80%93Fano_coding
//!
//! # Usage
//!
//! cshannon provides a binary that can be used for compression / decompression
//! at the command line and a library that can be integrated into other projects.
//!
//! Run `cshannon --help` to see the command-line options for the binary.
//!
//! The library exposes the same functionality via the `run` function:
//! ```
//! use cshannon::{Args, Command, CompressArgs, EncodingScheme, TokenizationScheme, run};
//! use std::path::Path;
//!
//! run(Args{
//!     command: Command::Compress(CompressArgs{
//!         tokenization_scheme: TokenizationScheme::Byte,
//!         encoding_scheme: EncodingScheme::Fano
//!     }),
//!     input_file: &Path::new("/path/to/input_file"),
//!     output_file: &Path::new("/path/to/output_file"),
//! });
//! ```
//!
//! # Abstraction operation description
//!
//! The abstract steps in compression are as follows:
//!
//! ```ascii-art
//! Input --> Tokens --> Model --> Encoding -+
//!   |                                      |
//!   +-----> Tokens ------------------------+--> Compressed
//!                                                 Output
//! ```
//!
//! The abstract steps for decompression are as follows:
//!
//! ```ascii-art
//! Compressed --> extract prefix --> Encoding
//!   Input                              |
//!    |                                 |
//!    +--> remaining data --------------+--> Output
//! ```
//!
//! Decompression is conceptually simpler because there are no choices (of
//! tokenizer and encoding). The encoding is included as a prefix in-band in the
//! compressed data.

// [internal documentation; not part of cargo docs]
//
// # Crate layout
//
// - The [tokens] module provides traits for tokenizing text. Three concrete
//   tokenization schemes are implemented: [tokens::bytes], [tokens::graphemes]
//   and [tokens::words].
// - The [model] module provides a way to compute a zeroeth order model from a
//   stream of tokens.
// - The [encoding] module provides traits for creating an encoding scheme from
//   a model. Four concrete encoding schemes are implemented:
//   [encoding::balanced_tree], [encoding::shannon], [encoding::fano] and
//   [encoding::huffman].
// - Finally, the [code] module provides methods to encode a token stream given
//   an encoding. The encoding itself is also included in the compressed
//   output.

mod code;
mod encoding;
mod model;
mod tokenization_scheme;
mod tokens;
mod util;

pub use crate::encoding::EncodingScheme;
pub use crate::tokenization_scheme::TokenizationScheme;

use anyhow::Result;
use std::path::Path;

/// The command to invoke via the `run` entry-point.
pub enum Command {
    /// Compress the data using one of the implemented algorithms.
    Compress(CompressArgs),
    /// Decompress data compressed previously using this library.
    ///
    /// This library does not maintain at-rest data format compatibility.
    /// Trying to decompress data compressed using a different version of the
    /// library is not guaranteed to work.
    Decompress(DecompressArgs),
}

/// Arguments specific to the compression operation.
pub struct CompressArgs {
    /// Choose how to split the input data into tokens that are individually
    /// compressed using one of the supported algorithms.
    pub tokenization_scheme: TokenizationScheme,
    /// Choose the compression algorithm used to compress the tokenized text.
    pub encoding_scheme: EncodingScheme,
}

/// Placeholder for (future) arguments specific to the decompression operation.
pub struct DecompressArgs {}

/// Arguments for the `run` entry-point of this library.
pub struct Args<'a> {
    pub command: Command,
    /// File to read the input text from.
    ///
    /// Some tokenization schemes make assumptions about the encoding of the
    /// source text. See documentation for `TokenizationScheme`.
    pub input_file: &'a Path,
    /// File to write the output (de)compressed text to.
    ///
    /// Data is utf-8 encoded before being written.
    pub output_file: &'a Path,
}

/// Invoke this library to compress or decompress data.
///
/// Example invocation:
/// ```
/// use cshannon::{Args, Command, CompressArgs, EncodingScheme, TokenizationScheme, run};
/// use std::path::Path;
///
/// run(Args{
///     command: Command::Compress(CompressArgs{
///         tokenization_scheme: TokenizationScheme::Byte,
///         encoding_scheme: EncodingScheme::Fano
///     }),
///     input_file: &Path::new("/path/to/input_file"),
///     output_file: &Path::new("/path/to/output_file"),
/// });
/// ```
///
/// See package documentation for an overview of the algorithms implemented in
/// this library.
pub fn run(args: Args) -> Result<()> {
    match args.command {
        Command::Compress(command_args) => internal::compress(
            args.input_file,
            args.output_file,
            command_args.encoding_scheme,
            command_args.tokenization_scheme,
        ),
        Command::Decompress(_) => internal::decompress(args.input_file, args.output_file),
    }
}

mod internal {

    use crate::code::Letter;
    use crate::encoding::{new_encoder, Encoding};
    use crate::model;
    use crate::tokenization_scheme::{pack_tokenization_scheme, unpack_tokenization_scheme};
    use crate::tokens::bytes::Byte;
    use crate::tokens::graphemes::Grapheme;
    use crate::tokens::words::Word;
    use crate::tokens::{Token, TokenPacker, Tokenizer};
    use crate::{EncodingScheme, TokenizationScheme};
    use anyhow::{anyhow, Result};
    use log::info;
    use std::collections::HashMap;
    use std::fs::File;
    use std::io::{BufReader, BufWriter};
    use std::path::Path;

    pub fn compress(
        input_file: &Path,
        output_file: &Path,
        encoding_scheme: EncodingScheme,
        tokenization_scheme: TokenizationScheme,
    ) -> Result<()> {
        info!("Compressing...");

        let mut w = BufWriter::new(File::create(output_file)?);
        pack_tokenization_scheme(tokenization_scheme, &mut w)?;

        match tokenization_scheme {
            TokenizationScheme::Byte => {
                compress_with_token::<Byte, _>(input_file, w, encoding_scheme)
            }
            TokenizationScheme::Grapheme => {
                compress_with_token::<Grapheme, _>(input_file, w, encoding_scheme)
            }
            TokenizationScheme::Word => {
                compress_with_token::<Word, _>(input_file, w, encoding_scheme)
            }
        }
    }

    pub fn decompress(input_file: &Path, output_file: &Path) -> Result<()> {
        info!("Decompressing...");
        let w = BufWriter::new(File::create(output_file)?);
        let mut r = BufReader::new(File::open(input_file)?);
        match unpack_tokenization_scheme(&mut r)? {
            TokenizationScheme::Byte => decompress_with_token::<Byte, _, _>(r, w),
            TokenizationScheme::Grapheme => decompress_with_token::<Grapheme, _, _>(r, w),
            TokenizationScheme::Word => decompress_with_token::<Word, _, _>(r, w),
        }
    }

    fn compress_with_token<T: Token, W: std::io::Write>(
        input_file: &Path,
        mut w: W,
        encoding_scheme: EncodingScheme,
    ) -> Result<()> {
        info!("Compressing...");
        let r = BufReader::new(File::open(input_file)?);
        let tokens = T::Tokenizer::tokenize(r).unwrap().map(|r| r.unwrap());
        let encoding = new_encoder(&&encoding_scheme, model::from(tokens))?;

        let r = BufReader::new(File::open(input_file)?);
        let tokens = T::Tokenizer::tokenize(r).unwrap().map(|r| r.unwrap());
        let code_text = encode(encoding.map(), tokens).map(|r| r.unwrap());

        encoding.pack(&mut w)?;
        crate::code::pack(code_text, &mut w)?;
        Ok(())
    }

    fn decompress_with_token<T: Token, R: std::io::Read, W: std::io::Write>(
        mut r: R,
        mut w: W,
    ) -> Result<()> {
        let encoding: Encoding<T> = Encoding::unpack(&mut r).unwrap();
        let map = encoding.reverse_map();
        let coded_text = crate::code::parse(&encoding.alphabet(), r)?.map(|r| r.unwrap());
        let decoded_text = decode(&map, coded_text).map(|r| r.unwrap());
        T::Packer::pack(decoded_text, &mut w)?;
        Ok(())
    }

    fn encode<'a, T, TS>(
        encoding: &'a HashMap<T, Letter>,
        input: TS,
    ) -> impl Iterator<Item = Result<&'a Letter>>
    where
        T: Token,
        TS: std::iter::Iterator<Item = T>,
    {
        input.map(move |t| match encoding.get(&t) {
            Some(l) => Ok(l),
            None => Err(anyhow!("Unknown token {}", t.to_string())),
        })
    }

    fn decode<'a, T, CS: 'a>(
        encoding: &'a HashMap<&'a Letter, &'a T>,
        input: CS,
    ) -> impl Iterator<Item = Result<T>> + 'a
    where
        T: Token,
        CS: std::iter::Iterator<Item = &'a Letter>,
    {
        input.map(move |l| match encoding.get(l) {
            Some(t) => Ok((*t).clone()),
            None => Err(anyhow!("no encoding for letter {}", l)),
        })
    }
}
