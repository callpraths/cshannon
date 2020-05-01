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
use cshannon::*;
use std::io::Cursor;

#[test]
fn roundtrip() {
    let l0 = code::Letter::new(&[0b1101_1000, 0b1000_0000], 9);
    let l1 = code::Letter::new(&[0b1100_0000], 4);
    let l2 = code::Letter::from_bytes(&[0b0101_1101]);

    let alphabet = code::Alphabet::new(vec![l0.clone(), l1.clone(), l2.clone()]);
    let text = vec![
        l1.clone(),
        l2.clone(),
        l0.clone(),
        l2.clone(),
        l1.clone(),
        l0.clone(),
        l1.clone(),
    ];

    let mut packed = Vec::new();
    assert!(code::pack(text.iter(), &mut packed).is_ok());
    let want_packed: Vec<u8> = vec![
        // l1__l2
        0b1100__0101,
        // l2__l0
        0b1101__1101,
        // l0__l2
        0b1000_1__010,
        // l2__l1
        0b1_1101__110,
        // l1__l0
        0b0__1101_100,
        // l0_l1_00
        0b0_1__1100_00,
    ];
    assert_eq!(packed, want_packed);
    let r: Result<Vec<&code::Letter>> = code::parse(&alphabet, Cursor::new(packed))
        .unwrap()
        .collect();
    let c = r.unwrap();
    assert_eq!(c, vec![&l1, &l2, &l0, &l2, &l1, &l0, &l1]);
}
