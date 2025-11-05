use anyhow::{anyhow, Result};

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum TokenizationScheme {
    Byte,
    Grapheme,
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
