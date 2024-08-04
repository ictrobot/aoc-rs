//! Implementation of the MD5 hash function.
//!
//! **WARNING: Don't use MD5 for anything remotely security-sensitive!**
//! This implementation is meant to be used for Advent of Code puzzles only.
//!
//! The vectorized versions hash multiple inputs of the same length at once, which provides a
//! significant performance increase for the brute force puzzle solutions.
use crate::multiversion;
use crate::multiversion::Version;
use std::array;
use std::sync::LazyLock;

mod bruteforce;
pub use bruteforce::find_hash_with_appended_count;

#[cfg(test)]
mod tests;

/// Fastest supported implementation, for dynamic dispatch.
///
/// Determined using a small microbenchmark at runtime.
pub static FASTEST: LazyLock<Version> = multiversion! { fastest(microbenchmark()) };

/// Returns the MD5 hash of the input slice.
///
/// Wrapper around the [`scalar`] implementation.
///
/// # Examples
///
/// ```
/// # use utils::md5::hash;
/// assert_eq!(hash(b"").as_slice(), &[0xd41d8cd9, 0x8f00b204, 0xe9800998, 0xecf8427e]);
/// assert_eq!(hash(b"Hello World").as_slice(), &[0xb10a8db1, 0x64e07541, 0x05b7a99b, 0xe72e3fe5]);
/// ```
#[must_use]
pub fn hash(buf: &[u8]) -> [u32; 4] {
    scalar::hash(buf)[0]
}

multiversion! {
    use {crate::simd::*};

    // The length of 1/2/3/4 bytes for each lane
    const ONE_BYTE: usize = U32Vector::LANES;
    const TWO_BYTES: usize = 2 * U32Vector::LANES;
    const THREE_BYTES: usize = 3 * U32Vector::LANES;
    const FOUR_BYTES: usize = 4 * U32Vector::LANES;

    /// [`multiversion!`] MD5 hash implementation.
    ///
    /// The bytes for each lane must be interweaved, and each lane must be the same length.
    ///
    /// # Examples
    ///
    /// For [`array128`](crate::simd::array128) with four lanes:
    /// ```
    /// # use utils::md5::{self, array128};
    /// assert_eq!(
    ///     array128::hash(b"hwafeobglrchlldiodej"),
    ///     [
    ///         md5::hash(b"hello"),
    ///         md5::hash(b"world"),
    ///         md5::hash(b"abcde"),
    ///         md5::hash(b"fghij"),
    ///     ],
    /// );
    #[must_use]
    pub fn hash(mut buf: &[u8]) -> [[u32; 4]; U32Vector::LANES] {
        assert_eq!(buf.len() % U32Vector::LANES, 0);
        let bytes = buf.len() / U32Vector::LANES;

        let mut state = [
            U32Vector::splat(0x6745_2301),
            U32Vector::splat(0xefcd_ab89),
            U32Vector::splat(0x98ba_dcfe),
            U32Vector::splat(0x1032_5476),
        ];

        let mut end_marker_written = false;
        let mut bit_count_written = false;
        while !bit_count_written {
            let mut words = [U32Vector::splat(0); 16];

            let remaining = (buf.len() / FOUR_BYTES).min(16);
            for (w, chunk) in words.iter_mut().zip(buf.chunks_exact(FOUR_BYTES)) {
                *w = gather(chunk.try_into().unwrap());
            }
            buf = &buf[remaining * FOUR_BYTES..];

            if remaining < 16 {
                if !end_marker_written {
                    // 0x80 end marker after final byte
                    words[remaining] = gather_remaining(buf);
                    buf = &[];
                    end_marker_written = true;
                }

                if !bit_count_written && remaining <= 13 {
                    let bits = bytes as u64 * 8;
                    words[14] = U32Vector::splat((bits & 0xFFFF_FFFF) as u32);
                    words[15] = U32Vector::splat((bits >> 32) as u32);
                    bit_count_written = true;
                }
            }

            state = md5_block(state, &words);
        }

        // `state.map(|x| x.into());` doesn't always get vectorised
        let state: [[u32; U32Vector::LANES]; 4] = array::from_fn(|i| state[i].into());

        array::from_fn(|i| {
            [
                state[0][i].swap_bytes(),
                state[1][i].swap_bytes(),
                state[2][i].swap_bytes(),
                state[3][i].swap_bytes(),
            ]
        })
    }

    #[inline]
    #[allow(clippy::trivially_copy_pass_by_ref)]
    fn gather(buf: &[u8; FOUR_BYTES]) -> U32Vector {
        let mut values = [0u32; U32Vector::LANES];
        for (i, v) in values.iter_mut().enumerate() {
            *v = u32::from_le_bytes([
                buf[i], buf[ONE_BYTE + i], buf[TWO_BYTES + i], buf[THREE_BYTES + i]
            ]);
        }
        values.into()
    }

    #[inline]
    fn gather_remaining(buf: &[u8]) -> U32Vector {
        match buf.len() {
            THREE_BYTES => {
                let mut values = [0u32; U32Vector::LANES];
                for (i, v) in values.iter_mut().enumerate() {
                    *v = u32::from_le_bytes([buf[i], buf[ONE_BYTE + i], buf[TWO_BYTES + i], 0x80]);
                }
                values.into()
            }
            TWO_BYTES => {
                let mut values = [0u32; U32Vector::LANES];
                for (i, v) in values.iter_mut().enumerate() {
                    *v = u32::from_le_bytes([buf[i], buf[ONE_BYTE + i], 0x80, 0]);
                }
                values.into()
            }
            ONE_BYTE => {
                let mut values = [0u32; U32Vector::LANES];
                for (i, v) in values.iter_mut().enumerate() {
                    *v = u32::from_le_bytes([buf[i], 0x80, 0, 0]);
                }
                values.into()
            }
            0 => U32Vector::splat(0x80),
            _ => unreachable!("less than 4 bytes left"),
        }
    }

    #[allow(clippy::many_single_char_names)]
    fn md5_block([a0, b0, c0, d0]: [U32Vector; 4], m: &[U32Vector; 16]) -> [U32Vector; 4] {
        let (mut a, mut b, mut c, mut d) = (a0, b0, c0, d0);

        a = md5_round(md5_f(b, c, d), a, b, m[0], 7, 0xd76a_a478);
        d = md5_round(md5_f(a, b, c), d, a, m[1], 12, 0xe8c7_b756);
        c = md5_round(md5_f(d, a, b), c, d, m[2], 17, 0x2420_70db);
        b = md5_round(md5_f(c, d, a), b, c, m[3], 22, 0xc1bd_ceee);
        a = md5_round(md5_f(b, c, d), a, b, m[4], 7, 0xf57c_0faf);
        d = md5_round(md5_f(a, b, c), d, a, m[5], 12, 0x4787_c62a);
        c = md5_round(md5_f(d, a, b), c, d, m[6], 17, 0xa830_4613);
        b = md5_round(md5_f(c, d, a), b, c, m[7], 22, 0xfd46_9501);
        a = md5_round(md5_f(b, c, d), a, b, m[8], 7, 0x6980_98d8);
        d = md5_round(md5_f(a, b, c), d, a, m[9], 12, 0x8b44_f7af);
        c = md5_round(md5_f(d, a, b), c, d, m[10], 17, 0xffff_5bb1);
        b = md5_round(md5_f(c, d, a), b, c, m[11], 22, 0x895c_d7be);
        a = md5_round(md5_f(b, c, d), a, b, m[12], 7, 0x6b90_1122);
        d = md5_round(md5_f(a, b, c), d, a, m[13], 12, 0xfd98_7193);
        c = md5_round(md5_f(d, a, b), c, d, m[14], 17, 0xa679_438e);
        b = md5_round(md5_f(c, d, a), b, c, m[15], 22, 0x49b4_0821);

        a = md5_round(md5_g(b, c, d), a, b, m[1], 5, 0xf61e_2562);
        d = md5_round(md5_g(a, b, c), d, a, m[6], 9, 0xc040_b340);
        c = md5_round(md5_g(d, a, b), c, d, m[11], 14, 0x265e_5a51);
        b = md5_round(md5_g(c, d, a), b, c, m[0], 20, 0xe9b6_c7aa);
        a = md5_round(md5_g(b, c, d), a, b, m[5], 5, 0xd62f_105d);
        d = md5_round(md5_g(a, b, c), d, a, m[10], 9, 0x0244_1453);
        c = md5_round(md5_g(d, a, b), c, d, m[15], 14, 0xd8a1_e681);
        b = md5_round(md5_g(c, d, a), b, c, m[4], 20, 0xe7d3_fbc8);
        a = md5_round(md5_g(b, c, d), a, b, m[9], 5, 0x21e1_cde6);
        d = md5_round(md5_g(a, b, c), d, a, m[14], 9, 0xc337_07d6);
        c = md5_round(md5_g(d, a, b), c, d, m[3], 14, 0xf4d5_0d87);
        b = md5_round(md5_g(c, d, a), b, c, m[8], 20, 0x455a_14ed);
        a = md5_round(md5_g(b, c, d), a, b, m[13], 5, 0xa9e3_e905);
        d = md5_round(md5_g(a, b, c), d, a, m[2], 9, 0xfcef_a3f8);
        c = md5_round(md5_g(d, a, b), c, d, m[7], 14, 0x676f_02d9);
        b = md5_round(md5_g(c, d, a), b, c, m[12], 20, 0x8d2a_4c8a);

        a = md5_round(md5_h(b, c, d), a, b, m[5], 4, 0xfffa_3942);
        d = md5_round(md5_h(a, b, c), d, a, m[8], 11, 0x8771_f681);
        c = md5_round(md5_h(d, a, b), c, d, m[11], 16, 0x6d9d_6122);
        b = md5_round(md5_h(c, d, a), b, c, m[14], 23, 0xfde5_380c);
        a = md5_round(md5_h(b, c, d), a, b, m[1], 4, 0xa4be_ea44);
        d = md5_round(md5_h(a, b, c), d, a, m[4], 11, 0x4bde_cfa9);
        c = md5_round(md5_h(d, a, b), c, d, m[7], 16, 0xf6bb_4b60);
        b = md5_round(md5_h(c, d, a), b, c, m[10], 23, 0xbebf_bc70);
        a = md5_round(md5_h(b, c, d), a, b, m[13], 4, 0x289b_7ec6);
        d = md5_round(md5_h(a, b, c), d, a, m[0], 11, 0xeaa1_27fa);
        c = md5_round(md5_h(d, a, b), c, d, m[3], 16, 0xd4ef_3085);
        b = md5_round(md5_h(c, d, a), b, c, m[6], 23, 0x0488_1d05);
        a = md5_round(md5_h(b, c, d), a, b, m[9], 4, 0xd9d4_d039);
        d = md5_round(md5_h(a, b, c), d, a, m[12], 11, 0xe6db_99e5);
        c = md5_round(md5_h(d, a, b), c, d, m[15], 16, 0x1fa2_7cf8);
        b = md5_round(md5_h(c, d, a), b, c, m[2], 23, 0xc4ac_5665);

        a = md5_round(md5_i(b, c, d), a, b, m[0], 6, 0xf429_2244);
        d = md5_round(md5_i(a, b, c), d, a, m[7], 10, 0x432a_ff97);
        c = md5_round(md5_i(d, a, b), c, d, m[14], 15, 0xab94_23a7);
        b = md5_round(md5_i(c, d, a), b, c, m[5], 21, 0xfc93_a039);
        a = md5_round(md5_i(b, c, d), a, b, m[12], 6, 0x655b_59c3);
        d = md5_round(md5_i(a, b, c), d, a, m[3], 10, 0x8f0c_cc92);
        c = md5_round(md5_i(d, a, b), c, d, m[10], 15, 0xffef_f47d);
        b = md5_round(md5_i(c, d, a), b, c, m[1], 21, 0x8584_5dd1);
        a = md5_round(md5_i(b, c, d), a, b, m[8], 6, 0x6fa8_7e4f);
        d = md5_round(md5_i(a, b, c), d, a, m[15], 10, 0xfe2c_e6e0);
        c = md5_round(md5_i(d, a, b), c, d, m[6], 15, 0xa301_4314);
        b = md5_round(md5_i(c, d, a), b, c, m[13], 21, 0x4e08_11a1);
        a = md5_round(md5_i(b, c, d), a, b, m[4], 6, 0xf753_7e82);
        d = md5_round(md5_i(a, b, c), d, a, m[11], 10, 0xbd3a_f235);
        c = md5_round(md5_i(d, a, b), c, d, m[2], 15, 0x2ad7_d2bb);
        b = md5_round(md5_i(c, d, a), b, c, m[9], 21, 0xeb86_d391);

        [a0 + a, b0 + b, c0 + c, d0 + d]
    }

    #[inline]
    #[allow(clippy::many_single_char_names)]
    fn md5_round(f: U32Vector, a: U32Vector, b: U32Vector, m: U32Vector, s: u32, k: u32) -> U32Vector {
        (f + a + m + U32Vector::splat(k)).rotate_left(s) + b
    }

    #[inline]
    fn md5_f(b: U32Vector, c: U32Vector, d: U32Vector) -> U32Vector {
        (b & c) | (d & !b)
    }

    #[inline]
    fn md5_g(b: U32Vector, c: U32Vector, d: U32Vector) -> U32Vector {
        (d & b) | (c & !d)
    }

    #[inline]
    fn md5_h(b: U32Vector, c: U32Vector, d: U32Vector) -> U32Vector {
        b ^ c ^ d
    }

    #[inline]
    fn md5_i(b: U32Vector, c: U32Vector, d: U32Vector) -> U32Vector {
        c ^ (b | !d)
    }

    pub(super) fn microbenchmark() {
        let bench_string = BENCH_STRING.as_flattened();
        for chunk in bench_string.chunks(32 * U32Vector::LANES) {
            for len in 1..=32 {
                std::hint::black_box(hash(&chunk[..len * U32Vector::LANES]));
            }
        }
    }
}

const BENCH_STRING: [[u8; 32]; 128] = {
    let mut out = [*b"abcdefghijklmnopqrstuvwxyz012345"; 128];
    let mut i = 0;

    #[allow(clippy::cast_possible_truncation)]
    while i < out.len() {
        out[i][i % 32] = i as u8;
        i += 1;
    }

    out
};

/// Convert an MD5 hash to ASCII hex.
///
/// Implemented using bitwise operations on each [`u32`] to spread each nibble into separate bytes,
/// followed by adding either '0' or 'a' to each byte using a bitmask.
///
/// When using the vectorized AVX2 MD5 implementation, this makes
/// [2016 day 14](../../year2016/struct.Day14.html) roughly 4x faster compared to using a naive
/// [`format!`] implementation `format!("{a:08x}{b:08x}{c:08x}{d:08x}")`.
///
/// # Examples
///
/// ```
/// # use utils::md5::to_hex;
/// assert_eq!(
///     to_hex([0xd41d8cd9, 0x8f00b204, 0xe9800998, 0xecf8427e]),
///     *b"d41d8cd98f00b204e9800998ecf8427e",
/// );
/// ```
#[inline]
#[must_use]
pub fn to_hex([a, b, c, d]: [u32; 4]) -> [u8; 32] {
    let mut result = [0u8; 32];
    result[0..8].copy_from_slice(&u32_to_hex(a));
    result[8..16].copy_from_slice(&u32_to_hex(b));
    result[16..24].copy_from_slice(&u32_to_hex(c));
    result[24..32].copy_from_slice(&u32_to_hex(d));
    result
}

#[inline]
fn u32_to_hex(n: u32) -> [u8; 8] {
    const SPLAT: u64 = 0x0101_0101_0101_0101;

    let mut n = u64::from(n);
    // n = 0x0000_0000_1234_ABCD

    n = ((n & 0x0000_0000_FFFF_0000) << 16) | (n & 0x0000_0000_0000_FFFF);
    // n = 0x0000_1234_0000_ABCD

    n = ((n & 0x0000_FF00_0000_FF00) << 8) | (n & 0x0000_00FF_0000_00FF);
    // n = 0x0012_0034_00AB_00CD

    n = ((n & 0x00F0_00F0_00F0_00F0) << 4) | (n & 0x000F_000F_000F_000F);
    // n = 0x0102_0304_0A0B_0C0D

    let letter_positions = (n + ((128 - 10) * SPLAT)) & (128 * SPLAT);
    // letter_positions = 0x0000_0000_8080_8080

    let letter_mask = letter_positions - (letter_positions >> 7);
    // letter_mask = 0x0000_0000_7F7F_7F7F

    let hex = (n + u64::from(b'0') * SPLAT) + (letter_mask & (u64::from(b'a' - b'0' - 10) * SPLAT));
    // hex = 0x3132_3334_6162_6364

    hex.to_be_bytes()
}
