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

use anyhow::Result;

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
    r.read_exact(&mut buf)?;
    Ok(u64::from_be_bytes(buf))
}
