use anyhow::{anyhow, Result};

/// Source text needs to be split into tokens that are then compressed using one
/// of the supported algorithms. This enum lists all the supported tokenization
/// schemes.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum TokenizationScheme {
    /// Split text byte-by-byte.
    ///
    /// This scheme makes no assumptions about the source text encoding.
    Byte,
    /// Split text by unicode [graphemes].
    ///
    /// This schemes assumes that source text is utf-8 encoded.
    ///
    /// [graphemes]: https://en.wikipedia.org/wiki/Grapheme
    Grapheme,
    /// Split text by unicode "words".
    ///
    /// This schemes assumes that source text is utf-8 encoded.
    /// This tokenization scheme (and hence the compression output) is lossy
    /// because punctuation etc. are lost after tokenization.
    ///
    /// [graphemes]: https://en.wikipedia.org/wiki/Grapheme
    ///
    /// WARNING: As of Nov 2025, this tokenization scheme is not yet implemented
    ///          properly. It is synonymous to `Grapheme`
    Word,
}

pub fn pack_tokenization_scheme<W: std::io::Write>(
    scheme: TokenizationScheme,
    w: &mut W,
) -> Result<()> {
    let marker = match scheme {
        TokenizationScheme::Byte => 1u8,
        TokenizationScheme::Grapheme => 2u8,
        TokenizationScheme::Word => 3u8,
    };
    w.write(&[marker])?;
    Ok(())
}

pub fn unpack_tokenization_scheme<R: std::io::Read>(r: &mut R) -> Result<TokenizationScheme> {
    let mut buf = [0u8];
    r.read_exact(&mut buf)?;
    let marker = buf[0];
    match marker {
        1u8 => Ok(TokenizationScheme::Byte),
        2u8 => Ok(TokenizationScheme::Grapheme),
        3u8 => Ok(TokenizationScheme::Word),
        _ => Err(anyhow!("Unknown tokenization scheme marker {}", marker)),
    }
}
