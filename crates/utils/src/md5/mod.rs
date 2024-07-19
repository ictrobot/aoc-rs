//! Implementation of the MD5 hash function.
//!
//! **WARNING: Don't use MD5 for anything remotely security-sensitive!**
//! This implementation is meant to be used for Advent of Code puzzles only.

#[cfg(test)]
mod tests;

/// Returns the MD5 hash of the input slice.
///
/// # Examples
///
/// ```
/// # use utils::md5::hash;
/// assert_eq!(hash(b""), (0xd41d8cd9, 0x8f00b204, 0xe9800998, 0xecf8427e));
/// assert_eq!(hash(b"Hello World"), (0xb10a8db1, 0x64e07541, 0x05b7a99b, 0xe72e3fe5));
/// ```
#[must_use]
pub fn hash(buf: &[u8]) -> (u32, u32, u32, u32) {
    let mut pos = 0;
    let mut state = (0x6745_2301, 0xefcd_ab89, 0x98ba_dcfe, 0x1032_5476);
    let mut words = [0; 16];

    let mut end_marker_written = false;
    let mut bit_count_written = false;
    while !bit_count_written {
        if pos + 64 <= buf.len() {
            for w in &mut words {
                *w = u32::from_le_bytes(buf[pos..pos + 4].try_into().unwrap());
                pos += 4;
            }
        } else if !end_marker_written {
            words = [0; 16];
            let mut i = 0;
            while pos + 4 <= buf.len() {
                words[i] = u32::from_le_bytes(buf[pos..pos + 4].try_into().unwrap());
                pos += 4;
                i += 1;
            }

            // 0x80 end marker after final byte
            words[i] = match buf.len() - pos {
                3 => u32::from_le_bytes([buf[pos], buf[pos + 1], buf[pos + 2], 0x80]),
                2 => u32::from_le_bytes([buf[pos], buf[pos + 1], 0x80, 0]),
                1 => u32::from_le_bytes([buf[pos], 0x80, 0, 0]),
                0 => 0x80,
                _ => unreachable!("less than 4 bytes left"),
            };
            end_marker_written = true;

            // If the remainder of the data and the end marker fit inside words[0..=13] then the bit
            // count goes in the same block
            bit_count_written = i <= 13;
        } else {
            // Block only contains the bit count
            words = [0; 16];
            bit_count_written = true;
        }

        if bit_count_written {
            let bits = buf.len() as u64 * 8;
            words[14] = (bits & 0xFFFF_FFFF) as u32;
            words[15] = (bits >> 32) as u32;
        }

        state = md5_block(state, &words);
    }

    (
        state.0.swap_bytes(),
        state.1.swap_bytes(),
        state.2.swap_bytes(),
        state.3.swap_bytes(),
    )
}

#[allow(clippy::many_single_char_names)]
fn md5_block((a0, b0, c0, d0): (u32, u32, u32, u32), m: &[u32; 16]) -> (u32, u32, u32, u32) {
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

    (
        a0.wrapping_add(a),
        b0.wrapping_add(b),
        c0.wrapping_add(c),
        d0.wrapping_add(d),
    )
}

#[allow(clippy::many_single_char_names)]
fn md5_round(f: u32, a: u32, b: u32, m: u32, s: u32, k: u32) -> u32 {
    f.wrapping_add(a)
        .wrapping_add(m)
        .wrapping_add(k)
        .rotate_left(s)
        .wrapping_add(b)
}

fn md5_f(b: u32, c: u32, d: u32) -> u32 {
    (b & c) | (d & !b)
}

fn md5_g(b: u32, c: u32, d: u32) -> u32 {
    (d & b) | (c & !d)
}

fn md5_h(b: u32, c: u32, d: u32) -> u32 {
    b ^ c ^ d
}

fn md5_i(b: u32, c: u32, d: u32) -> u32 {
    c ^ (b | !d)
}