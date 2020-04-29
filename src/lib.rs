#![feature(associated_type_bounds)]

pub mod code;
pub mod coder;
pub mod encoding;
pub mod model;
pub mod tokens;

use anyhow::{anyhow, Result};
use code::Letter;
use tokens::bytes::{Byte, ByteIter, BytePacker};
use tokens::graphemes::{Grapheme, GraphemeIter, GraphemePacker};
use tokens::words::{Word, WordIter, WordPacker};
use tokens::{Token, TokenIter, TokenPacker};

use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, BufWriter, Cursor};

pub fn run(args: clap::ArgMatches) -> Result<()> {
    // Safe to use unwrap() because these args are `required`.
    let input_file = args.value_of("input_file").unwrap();
    let output_file = args.value_of("output_file").unwrap();
    let tokenizer_choice = args.value_of("tokenizer").unwrap();

    match args.subcommand_name() {
        Some("compress") => match tokenizer_choice {
            "byte" => {
                compress::<Byte, ByteIter<BufReader<File>>, BytePacker>(&input_file, &output_file)
            }
            "grapheme" => {
                compress::<Grapheme, GraphemeIter, GraphemePacker>(&input_file, &output_file)
            }
            "word" => compress::<Word, WordIter, WordPacker>(&input_file, &output_file),
            _ => Err(anyhow!("invalid tokenizer {}", tokenizer_choice)),
        },
        Some("decompress") => match tokenizer_choice {
            "byte" => {
                decompress::<Byte, ByteIter<BufReader<File>>, ByteIter<Cursor<Vec<u8>>>, BytePacker>(
                    &input_file,
                    &output_file,
                )
            }
            "grapheme" => decompress::<Grapheme, GraphemeIter, GraphemeIter, GraphemePacker>(
                &input_file,
                &output_file,
            ),
            "word" => decompress::<Word, WordIter, WordIter, WordPacker>(&input_file, &output_file),
            _ => Err(anyhow!("invalid tokenizer {}", tokenizer_choice)),
        },
        _ => Err(anyhow!("no sub-command selected")),
    }
}

/// Document me.
/// TODO: Convert to use AsRef<Path>
pub fn compress<T, TIter, TPacker>(input_file: &str, output_file: &str) -> Result<()>
where
    T: Token,
    TIter: TokenIter<BufReader<File>, T = T>,
    TPacker: TokenPacker<BufWriter<File>, T = T>,
{
    println!("Compressing...");
    let r = BufReader::new(File::open(input_file)?);
    let tokens = TIter::unpack(r).map(|r| r.unwrap());
    let encoding = crate::encoding::balanced_tree::new(model::from(tokens))?;

    let r = BufReader::new(File::open(input_file)?);
    let tokens = TIter::unpack(r).map(|r| r.unwrap());
    let code_text = coder::encode(encoding.map(), tokens).map(|r| r.unwrap());

    let mut w = BufWriter::new(File::create(output_file)?);
    crate::tokens::pack_all::<_, _, TPacker>(encoding.tokens(), &mut w)?;
    encoding.alphabet().pack(&mut w)?;
    crate::code::pack(code_text, &mut w)?;
    Ok(())
}

/// Document me.
/// TODO: Convert to use AsRef<Path>
pub fn decompress<T, TIter, TAllIter, TPacker>(input_file: &str, output_file: &str) -> Result<()>
where
    T: Token,
    TIter: TokenIter<BufReader<File>, T = T>,
    TAllIter: TokenIter<Cursor<Vec<u8>>, T = T>,
    TPacker: TokenPacker<BufWriter<File>, T = T>,
{
    println!("Decompressing...");
    let r = File::open(input_file)?;
    // Must use a cloned File because unpack_with_len() expects to take
    // ownership of the supplied `Read`er.
    let tokens = crate::tokens::unpack_all::<_, _, TAllIter>(BufReader::new(r.try_clone()?))?;

    let mut br = BufReader::new(r);
    let alphabet = crate::code::Alphabet::unpack(&mut br)?;
    let map = alphabet
        .letters()
        .iter()
        .cloned()
        .zip(tokens.into_iter())
        .collect::<HashMap<Letter, T>>();

    let coded_text = crate::code::parse(&alphabet, &mut br)?.map(|r| r.unwrap());
    let tokens = crate::coder::decode(&map, coded_text).map(|r| r.unwrap());

    let mut w = BufWriter::new(File::create(output_file)?);
    TPacker::pack(tokens, &mut w)?;
    Ok(())
}
