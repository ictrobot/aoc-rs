use utils::number::lcm;
use utils::prelude::*;

/// Transforming digits with a repeating block pattern.
///
/// This implementation is based on the following:
/// - [Advent of Code 2019 in 130 ms: day 16](https://emlun.se/advent-of-code-2019/2020/08/26/day-16.html)
/// - [/u/ephemient's post "Let's combinatorics"](https://www.reddit.com/r/adventofcode/comments/ebqgdu/2019_day_16_part_2_lets_combinatorics/)
/// - [Voltara's C++ implementation](https://github.com/Voltara/advent2019-fast/blob/5a35123d4113d63cf2bc0d59edb1e0a6d5d67d0c/src/day16.cpp)
///   and [README explanation](https://github.com/Voltara/advent2019-fast/blob/5a35123d4113d63cf2bc0d59edb1e0a6d5d67d0c/README.md#day-16)
#[derive(Clone, Debug)]
pub struct Day16 {
    digits: Vec<u8>,
    offset: usize,
}

const PHASES: usize = 100;
const PART2_REPEAT: usize = 10_000;

// After 100 suffix-sum phases, the kth digit after the offset is multiplied by C(99 + k, k) mod 10.
// Instead of calculating every value of C(99 + k, k), part 2 keeps only the positions where the
// coefficient is non-zero mod 2 or mod 5, combining the results to sum the matching input digits.
//
// Mod 2: by Lucas' theorem in base 2, C(n, k) is odd iff every 1 bit in k is also a 1 bit in n
// (k & !n == 0). Here n = 99 + k, so C(99 + k, k) is odd iff k & 99 == 0.
// Since 99 = 0b1100011, one mod 128 cycle leaves only k mod 128 in {0, 4, ..., 28}, which use
// only the bit positions where 99 has 0s.
// The second tuple element is the mod 10 value to add for the mod 2 component.
// 5 is used for all the terms as it is 1 mod 2 and 0 mod 5.
const BINOMIAL_MOD2_TERMS: [(usize, u32); 8] = [
    (0, 5),
    (4, 5),
    (8, 5),
    (12, 5),
    (16, 5),
    (20, 5),
    (24, 5),
    (28, 5),
];
const BINOMIAL_MOD2_PERIOD: usize = 128;

// Mod 5: by Lucas' theorem in base 5, C(n, k) is non-zero mod 5 iff every base 5 digit in k is at
// most the matching digit in n.
// Here n = 99 + k and 99 = 344 in base 5, so the 1s and 5s digits in k must be 0 and the 25s digit
// can be 0 or 1, giving k mod 125 = 0 or 25.
//
// The second tuple element is the mod 10 value to add for the mod 5 component.
// For k = 0, 6 is used as C(99, 0) = 1 mod 5, and 6 is 1 mod 5 and 0 mod 2.
// For k = 25, 4 is used as C(124, 25) = 4 mod 5, and 4 is already 0 mod 2.
const BINOMIAL_MOD5_TERMS: [(usize, u32); 2] = [(0, 6), (25, 4)];
const BINOMIAL_MOD5_PERIOD: usize = 125;

impl Day16 {
    pub fn new(input: &str, _: InputType) -> Result<Self, InputError> {
        if let Some(index) = input.bytes().position(|b| !b.is_ascii_digit()) {
            return Err(InputError::new(input, index, "expected digit"));
        }
        if input.len() < 8 {
            return Err(InputError::new(
                input,
                input.len(),
                "expected at least 8 digits",
            ));
        }

        let digits = input.bytes().map(|b| b - b'0').collect::<Vec<_>>();
        let offset = digits[..7]
            .iter()
            .fold(0usize, |acc, &digit| 10 * acc + digit as usize);

        Ok(Self { digits, offset })
    }

    #[must_use]
    pub fn part1(&self) -> u32 {
        let mut digits = self.digits.clone();
        let mut next = vec![0; digits.len()];
        let mut prefix = vec![0i32; digits.len() + 1];

        let len = digits.len();
        let quarter = len / 4;
        let third = len / 3;
        let half = len / 2;

        for _ in 0..PHASES {
            prefix[0] = 0;
            let mut total = 0;
            for (&digit, prefix) in digits.iter().zip(prefix.iter_mut().skip(1)) {
                total += i32::from(digit);
                *prefix = total;
            }

            let block_sum = |start: usize, width: usize| {
                prefix[(start + width).min(len)] - prefix[start.min(len)]
            };

            // Before len/4, rows can still contain multiple +1 and -1 blocks.
            for (index, out) in next.iter_mut().enumerate().take(quarter) {
                let width = index + 1;
                let mut total = 0i32;
                let mut start = index;

                while start < len {
                    total += block_sum(start, width);
                    start += 2 * width;

                    if start >= len {
                        break;
                    }

                    total -= block_sum(start, width);
                    start += 2 * width;
                }

                *out = (total.abs() % 10) as u8;
            }

            // Between len/4 and len/3, the row has exactly one +1 block and one -1 block.
            for (index, out) in next.iter_mut().enumerate().take(third).skip(quarter) {
                let width = index + 1;
                let total = block_sum(index, width) - block_sum(index + 2 * width, width);
                *out = (total.abs() % 10) as u8;
            }

            // Between len/3 and len/2, the row has exactly one +1 block.
            for (index, out) in next.iter_mut().enumerate().take(half).skip(third) {
                let width = index + 1;
                *out = (block_sum(index, width) % 10) as u8;
            }

            // From len/2 onward, the row is just a suffix sum.
            let mut suffix = 0u8;
            for i in (half..len).rev() {
                suffix += digits[i];
                suffix = suffix.wrapping_sub(10 * u8::from(suffix >= 10));
                next[i] = suffix;
            }

            (digits, next) = (next, digits);
        }

        digits
            .iter()
            .take(8)
            .fold(0, |acc, &digit| 10 * acc + u32::from(digit))
    }

    #[must_use]
    pub fn part2(&self) -> u32 {
        let len = self.digits.len();
        let total_len = len * PART2_REPEAT;
        assert!(
            self.offset + 8 <= total_len,
            "expected message offset within repeated signal"
        );
        assert!(
            self.offset >= total_len / 2,
            "expected message offset in second half of repeated signal"
        );

        // Coefficients repeat every 128 / 125 terms, and the repeated input repeats every `len`, so
        // the product repeats every `lcm(len, period)` terms.
        // For mod 2, each term adds 0 or 5 and each cycle sums to a multiple of 5, so 2 cycles sum
        // to 0 mod 10 and can be skipped.
        // For mod 5, each term adds 4*d or 6*d and one cycle sums to a multiple of 2, so 5 cycles
        // sum to 0 mod 10 and can be skipped.
        let mod2_skip_period = 2 * lcm(len, BINOMIAL_MOD2_PERIOD);
        let mod5_skip_period = 5 * lcm(len, BINOMIAL_MOD5_PERIOD);
        let mut result = 0;

        for i in 0..8 {
            let tail_len = total_len - self.offset - i;
            let start_index = (self.offset + i) % len;

            let sparse_sum = |step: usize, skip_period: usize, terms: &[(usize, u32)], mask: u8| {
                let mut total = 0;
                for &(residue, coefficient) in terms {
                    if residue >= tail_len {
                        break;
                    }

                    let remainder = tail_len - residue;
                    let skip = remainder - remainder % skip_period;
                    let mut position = residue + skip;
                    let mut index = (start_index + residue) % len;

                    while position < tail_len {
                        total += coefficient * u32::from(self.digits[index] & mask);
                        position += step;
                        index += step;
                        if index >= len {
                            index %= len;
                        }
                    }
                }
                total
            };

            let mod2_total = sparse_sum(
                BINOMIAL_MOD2_PERIOD,
                mod2_skip_period,
                &BINOMIAL_MOD2_TERMS,
                1,
            );
            let mod5_total = sparse_sum(
                BINOMIAL_MOD5_PERIOD,
                mod5_skip_period,
                &BINOMIAL_MOD5_TERMS,
                u8::MAX,
            );
            let total = mod2_total + mod5_total;
            result = 10 * result + total % 10;
        }

        result
    }
}

examples!(Day16 -> (u32, u32) [
    {input: "80871224585914546619083218645595", part1: 24176176},
    {input: "19617804207202209144916044189917", part1: 73745418},
    {input: "69317163492948606335995924319873", part1: 52432133},
    {input: "03036732577212944063491565474664", part2: 84462026},
    {input: "02935109699940807407585447034323", part2: 78725270},
    {input: "03081770884921959731165446850517", part2: 53553731},
]);
