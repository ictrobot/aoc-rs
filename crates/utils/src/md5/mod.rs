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
    scalar::hash(buf, buf.len(), buf.len())[0]
}

multiversion! {
    use {crate::simd::*};

    /// [`multiversion!`] MD5 hash implementation.
    ///
    /// Hash the first `bytes` of each lane in the buffer.
    /// The buffer must be exactly `lane_width` * [`U32Vector::LANES`] long.
    #[must_use]
    pub fn hash(buf: &[u8], lane_width: usize, bytes: usize) -> [[u32; 4]; U32Vector::LANES] {
        assert_eq!(buf.len(), lane_width * U32Vector::LANES);
        assert!(bytes <= lane_width);

        let mut pos = 0;
        let mut state = [
            U32Vector::splat(0x6745_2301),
            U32Vector::splat(0xefcd_ab89),
            U32Vector::splat(0x98ba_dcfe),
            U32Vector::splat(0x1032_5476),
        ];
        let mut words = [U32Vector::splat(0); 16];

        let mut end_marker_written = false;
        let mut bit_count_written = false;
        while !bit_count_written {
            if pos + 64 <= bytes {
                for w in &mut words {
                    *w = gather(buf, lane_width, pos);
                    pos += 4;
                }
            } else if !end_marker_written {
                words = [U32Vector::splat(0); 16];
                let mut i = 0;
                while pos + 4 <= bytes {
                    words[i] = gather(buf, lane_width, pos);
                    pos += 4;
                    i += 1;
                }

                // 0x80 end marker after final byte
                words[i] = gather_remaining(buf, lane_width, pos, bytes - pos);
                end_marker_written = true;

                // If the remainder of the data and the end marker fit inside words[0..=13] then the
                // bit count goes in the same block
                bit_count_written = i <= 13;
            } else {
                // Block only contains the bit count
                words = [U32Vector::splat(0); 16];
                bit_count_written = true;
            }

            if bit_count_written {
                let bits = bytes as u64 * 8;
                words[14] = U32Vector::splat((bits & 0xFFFF_FFFF) as u32);
                words[15] = U32Vector::splat((bits >> 32) as u32);
            }

            state = md5_block(state, &words);
        }

        let state = state.map(|x| {
            let mut arr = [0; U32Vector::LANES];
            x.store(&mut arr);
            arr
        });

        array::from_fn(|i| {
            [
                state[0][i].swap_bytes(),
                state[1][i].swap_bytes(),
                state[2][i].swap_bytes(),
                state[3][i].swap_bytes(),
            ]
        })
    }

    fn gather(buf: &[u8], lane_width: usize, pos: usize) -> U32Vector {
        let mut values = [0u32; U32Vector::LANES];
        for (v, chunk) in values.iter_mut().zip(buf.chunks_exact(lane_width)) {
            *v = u32::from_le_bytes(chunk[pos..pos + 4].try_into().unwrap());
        }
        U32Vector::load(&values)
    }

    fn gather_remaining(buf: &[u8], lane_width: usize, pos: usize, count: usize) -> U32Vector {
        match count {
            3 => {
                let mut values = [0u32; U32Vector::LANES];
                for (v, chunk) in values.iter_mut().zip(buf.chunks_exact(lane_width)) {
                    *v = u32::from_le_bytes([chunk[pos], chunk[pos + 1], chunk[pos + 2], 0x80]);
                }
                U32Vector::load(&values)
            }
            2 => {
                let mut values = [0u32; U32Vector::LANES];
                for (v, chunk) in values.iter_mut().zip(buf.chunks_exact(lane_width)) {
                    *v = u32::from_le_bytes([chunk[pos], chunk[pos + 1], 0x80, 0]);
                }
                U32Vector::load(&values)
            }
            1 => {
                let mut values = [0u32; U32Vector::LANES];
                for (v, chunk) in values.iter_mut().zip(buf.chunks_exact(lane_width)) {
                    *v = u32::from_le_bytes([chunk[pos], 0x80, 0, 0]);
                }
                U32Vector::load(&values)
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
        for max_len in [4, 8, 16, 32] {
            for chunk in BENCH_STRING.chunks(max_len * U32Vector::LANES) {
                for len in 1..=max_len {
                    std::hint::black_box(hash(chunk, max_len, len));
                }
            }
        }
    }
}

const BENCH_STRING: [u8; 256] = *b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789abcdefABCDEF\x01\x02\x03\x04\x05\x06\x07\x08\x09\x0a\x0b\x0c\x0d\x0e\x0f\x10\x11\x12\x13\x14\x15\x16\x17\x18\x19\x1a\x1b\x1c\x1d\x1e\x1f\x20\x21\x22\x23\x24\x25\x26\x27\x28\x29\x2a\x2b\x2c\x2d\x2e\x2f\x30\x31\x32\x33\x34\x35\x36\x37\x38\x39\x3a\x3b\x3c\x3d\x3e\x3f\x40\x41\x42\x43\x44\x45\x46\x47\x48\x49\x4a\x4b\x4c\x4d\x4e\x4f\x50\x51\x52\x53\x54\x55\x56\x57\x58\x59\x5a\x5b\x5c\x5d\x5e\x5f\x60\x61\x62\x63\x64\x65\x66\x67\x68\x69\x6a\x6b\x6c\x6d\x6e\x6f\x70\x71\x72\x73\x74\x75\x76\x77\x78\x79\x7a\x7b\x7c\x7d\x7e\x7f\x80\x81\x82\x83\x84\x85\x86\x87\x88\x89\x8a\x8b\x8c\x8d\x8e\x8f\x90\x91\x92\x93\x94\x95\x96\x97\x98\x99\x9a\x9b\x9c\x9d\x9e\x9f\xa0\xa1\xa2\xa3\xa4\xa5\xa6\xa7\xa8\xa9\xaa\xab\xac\xad\xae\xaf\xb0\xb1\xb2\xb3\xb4\xb5\xb6";
