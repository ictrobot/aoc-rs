//! Knot Hash implementation.
//!
//! See [`Day10`](crate::Day10) and [`Day14`](crate::Day14).

use std::array;
use utils::md5;

#[inline]
pub(crate) fn knot_rounds(lengths: impl Iterator<Item = u8> + Clone, rounds: u32) -> [u8; 256] {
    let mut list = array::from_fn(|i| i as u8);
    let mut position = 0;
    let mut skip = 0;

    for _ in 0..rounds {
        for length in lengths.clone() {
            list[0..length as usize].reverse();
            list.rotate_left((length as usize + skip) % 256);
            position = (position + length as usize + skip) % 256;
            skip += 1;
        }
    }

    list.rotate_right(position);
    list
}

#[inline]
pub(crate) fn knot_hash(lengths: impl Iterator<Item = u8> + Clone) -> [u8; 16] {
    let sparse = knot_rounds(lengths.chain([17, 31, 73, 47, 23]), 64);

    array::from_fn(|i| {
        sparse[16 * i..16 * (i + 1)]
            .iter()
            .fold(0, |acc, x| acc ^ x)
    })
}

#[inline]
pub(crate) fn knot_hash_hex(lengths: impl Iterator<Item = u8> + Clone) -> [u8; 32] {
    let hash = knot_hash(lengths);

    md5::to_hex(array::from_fn(|i| {
        u32::from_be_bytes(hash[4 * i..4 * (i + 1)].try_into().unwrap())
    }))
}
