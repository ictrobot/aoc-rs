use utils::prelude::*;

/// Counting non-decreasing numbers containing pairs.
#[derive(Clone, Debug)]
pub struct Day04 {
    part1: u32,
    part2: u32,
}

impl Day04 {
    pub fn new(input: &str, _: InputType) -> Result<Self, InputError> {
        let [start, end] = parser::digit()
            .repeat_n::<6, _>(parser::noop())
            .map_res(|x| {
                if x[0] == 0 {
                    Err("expected six digit number not starting with zero")
                } else {
                    Ok(x)
                }
            })
            .repeat_n(b'-')
            .parse_complete(input)?;

        let (mut part1, mut part2) = (0, 0);
        for password in Self::packed_passwords(start, end) {
            // e.g. password  = 0x0000_0101_0202_0203

            // adjacent_xor   = 0x0000_0100_0300_0001
            let adjacent_xor = password ^ (password >> 8);
            // different_mask = 0x0000_1000_1000_0010
            let different_mask = (adjacent_xor + 0x0f0f_0f0f_0f0f) & 0x1010_1010_1010;

            // pair           = 0x0000_0010_0010_1000
            let pair = !different_mask & 0x1010_1010_1010;
            part1 += u32::from(pair != 0);

            // padded_mask    = 0x1010_0010_0000_1010
            let padded_mask = (different_mask << 8) | 0x1000_0000_0000_0010;
            // exact_pair     = 0x0000_1000_0000_0000
            let exact_pair = !padded_mask & (padded_mask << 8) & (padded_mask >> 8);
            part2 += u32::from(exact_pair != 0);
        }

        Ok(Self { part1, part2 })
    }

    #[must_use]
    pub fn part1(&self) -> u32 {
        self.part1
    }

    #[must_use]
    pub fn part2(&self) -> u32 {
        self.part2
    }

    fn packed_passwords(mut start: [u8; 6], end: [u8; 6]) -> impl Iterator<Item = u64> {
        // Ensure the start is non-decreasing
        if let Some(i) = start.windows(2).position(|w| w[0] > w[1]) {
            let digit = start[i];
            start[i + 1..].fill(digit);
        }

        let mut password = u64::from_be_bytes([
            0, 0, start[0], start[1], start[2], start[3], start[4], start[5],
        ]);
        let end = u64::from_be_bytes([0, 0, end[0], end[1], end[2], end[3], end[4], end[5]]);

        std::iter::from_fn(move || {
            if password > end {
                return None;
            }
            let current = password;

            // Advance the packed password to the next non-decreasing password
            // e.g. password              = 0x0000_0102_0303_0909

            // nine_mask                  = 0x0000_0000_0000_1010
            let nine_mask = (password + 0x0707_0707_0707) & 0x1010_1010_1010;
            // (nine_mask << 16 | 0xFFFF) = 0x0000_0000_1010_FFFF
            // leading_less_than_nine     = 4
            let leading_less_than_nine = (nine_mask << 16 | 0xFFFF).leading_zeros() / 8;

            if leading_less_than_nine == 0 {
                // All nines, return current then stop
                password = u64::MAX;
                return Some(current);
            }

            // trailing_nine_count        = 2
            let trailing_nine_count = 6 - leading_less_than_nine;
            // last_non_nine_digit        = 0x0000_0000_0000_0003
            let last_non_nine_digit = (password >> (trailing_nine_count * 8)) & 0xf;
            // next_digit                 = 0x0000_0000_0000_0004
            let next_digit = last_non_nine_digit + 1;
            // next_fill                  = 0x0000_0404_0404_0404
            let next_fill = 0x0101_0101_0101 * next_digit;

            // replace_count              = 3
            let replace_count = trailing_nine_count + 1;
            // keep_mask                  = 0xFFFF_FFFF_FF00_0000
            let keep_mask = u64::MAX << (replace_count * 8);

            // password                   = 0x0000_0102_0304_0404
            password = (password & keep_mask) | (next_fill & !keep_mask);

            Some(current)
        })
    }
}

examples!(Day04 -> (u32, u32) []);
