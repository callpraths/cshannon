use super::types::Result;

pub static BIT_HOLE_MASKS: [u8; 8] = [
    0b1000_0000,
    0b0100_0000,
    0b0010_0000,
    0b0001_0000,
    0b0000_1000,
    0b0000_0100,
    0b0000_0010,
    0b0000_0001,
];

pub fn pack_u64(s: u64) -> Vec<u8> {
    s.to_be_bytes().to_vec()
}

pub fn unpack_u64<R: std::io::Read>(mut r: R) -> Result<u64> {
    let mut buf: [u8; 8] = [0; 8];
    r.read_exact(&mut buf).map_err(|e| e.to_string())?;
    Ok(u64::from_be_bytes(buf))
}
