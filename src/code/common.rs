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

pub fn unpack_u64(iter: &mut std::vec::IntoIter<u8>) -> Result<u64> {
    let mut buf: [u8; 8] = [0; 8];
    for i in 0..8 {
        match iter.next() {
            Some(u) => buf[i] = u,
            None => return Err("too few elements".to_owned()),
        }
    }
    Ok(u64::from_be_bytes(buf))
}
