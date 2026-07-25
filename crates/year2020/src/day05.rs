use utils::prelude::*;

/// Decoding binary numbers.
///
/// The key optimization is that `F`/`B` and `L`/`R` differ only in bit 2. Inverting that bit in
/// each of the first 8 bytes and gathering the results with one multiply decodes the row and the
/// first column bit together.
///
/// Validation compares each byte against the all-zeros and all-ones passes for its position.
/// lcm(11, 16) = 176 and lcm(12, 16) = 48, so repeating those two lines to fill a block gives
/// templates the compiler can check 16 bytes at a time. This is ~3x faster than parsing bytes one
/// at a time.
#[derive(Clone, Debug)]
pub struct Day05 {
    seats: [u64; 18],
}

const LETTER_BIT: u8 = 1 << 2;
const LOW_BITS: u64 = 0x0101_0101_0101_0101;
const PACK_MULTIPLIER: u64 = 0x8040_2010_0804_0201;
const LF_ZEROS: [u8; 176] = template(b"FFFFFFFLLL\n");
const LF_ONES: [u8; 176] = template(b"BBBBBBBRRR\n");
const CRLF_ZEROS: [u8; 48] = template(b"FFFFFFFLLL\r\n");
const CRLF_ONES: [u8; 48] = template(b"BBBBBBBRRR\r\n");

const fn template<const N: usize>(pass: &[u8]) -> [u8; N] {
    assert!(N.is_multiple_of(pass.len()) && N.is_multiple_of(16));
    let mut template = [0; N];
    let mut i = 0;
    while i < N {
        template[i] = pass[i % pass.len()];
        i += 1;
    }
    template
}

impl Day05 {
    pub fn new(input: &str, _: InputType) -> Result<Self, InputError> {
        if input.is_empty() {
            return Err(InputError::new(input, 0, "expected one of 'B', 'F'"));
        }

        let parsed = if input.as_bytes().get(10) == Some(&b'\r') {
            Self::parse::<12, _>(input.as_bytes(), &CRLF_ZEROS, &CRLF_ONES)
        } else {
            Self::parse::<11, _>(input.as_bytes(), &LF_ZEROS, &LF_ONES)
        };
        parsed.ok_or_else(|| Self::parse_error(input))
    }

    fn parse<const STRIDE: usize, const BLOCK: usize>(
        bytes: &[u8],
        zeros: &[u8; BLOCK],
        ones: &[u8; BLOCK],
    ) -> Option<Self> {
        // 1,024 seat bits, with one element of padding on each side
        let mut seats = [0u64; 18];
        let mut valid = true;

        let (blocks, tail) = bytes.as_chunks::<BLOCK>();
        for block in blocks {
            for ((&b, &zero), &one) in block.iter().zip(zeros).zip(ones) {
                valid &= b == zero || b == one;
            }
            for pass in block.as_chunks::<STRIDE>().0 {
                Self::decode_pass(pass, &mut seats);
            }
        }

        for ((&b, &zero), &one) in tail.iter().zip(zeros).zip(ones) {
            valid &= b == zero || b == one;
        }
        let (tail_passes, last) = tail.as_chunks::<STRIDE>();
        for pass in tail_passes {
            Self::decode_pass(pass, &mut seats);
        }

        if last.len() == 10 {
            Self::decode_pass(last, &mut seats);
        } else {
            valid &= last.is_empty();
        }

        let line_count = bytes.len() / STRIDE + usize::from(last.len() == 10);
        let seat_count = seats[1..17]
            .iter()
            .map(|bits| bits.count_ones() as usize)
            .sum::<usize>();
        valid &= line_count == seat_count;

        valid.then_some(Self { seats })
    }

    #[inline]
    fn decode_pass(pass: &[u8], seats: &mut [u64; 18]) {
        let first = u64::from_le_bytes(pass[..8].try_into().unwrap());

        // F and L have bit 2 set, so invert and pack 1 bit from each byte
        let bits = (!first >> 2) & LOW_BITS;
        let high = (bits.wrapping_mul(PACK_MULTIPLIER) >> 56) as usize;
        let bit1 = usize::from(pass[8] & LETTER_BIT == 0);
        let bit0 = usize::from(pass[9] & LETTER_BIT == 0);
        let seat = (high << 2) | (bit1 << 1) | bit0;

        seats[(seat / 64) + 1] |= 1 << (seat % 64);
    }

    #[cold]
    fn parse_error(input: &str) -> InputError {
        // Re-parse with the combinator parser to find the exact error position
        let row =
            parser::byte_map!(b'F' => 0, b'B' => 1)
                .repeat_fold(parser::noop(), 7, 0, |acc, b| (acc << 1) | b);
        let col =
            parser::byte_map!(b'L' => 0, b'R' => 1)
                .repeat_fold(parser::noop(), 3, 0, |acc, b| (acc << 1) | b);
        let parser = row
            .then(col)
            .map(|(row, col)| (row << 3) | col)
            .with_consumed()
            .with_eol();

        let mut seats = [false; 1024];
        for result in parser.parse_iterator(input) {
            let (seat, pass) = match result {
                Ok(pass) => pass,
                Err(error) => return error,
            };
            if pass.len() != 10 {
                return InputError::new(input, pass, "expected 10-character boarding pass");
            }
            if seats[seat as usize] {
                return InputError::new(input, pass, "duplicate boarding pass");
            }
            seats[seat as usize] = true;
        }

        // Input that fails the fast chunk parser but passes this parser must have mixed endings
        InputError::new(input, 0, "expected consistent line endings")
    }

    #[must_use]
    pub fn part1(&self) -> u16 {
        self.seats[1..17]
            .iter()
            .enumerate()
            .rev()
            .find_map(|(index, &bits)| {
                (bits != 0).then_some(index as u16 * 64 + 63 - bits.leading_zeros() as u16)
            })
            .expect("input contains at least one boarding pass")
    }

    #[must_use]
    pub fn part2(&self) -> u16 {
        self.seats
            .array_windows()
            .enumerate()
            .find_map(|(index, &[previous, current, next])| {
                let missing = !current
                    & ((current << 1) | (previous >> 63))
                    & ((current >> 1) | (next << 63));
                (missing != 0).then_some(index as u16 * 64 + missing.trailing_zeros() as u16)
            })
            .expect("no solution found")
    }
}

examples!(Day05 -> (u16, u16) [
    {input: "FBFBBFFRLR", part1: 357},
    {input: "BFFFBBFRRR", part1: 567},
    {input: "FFFBBBFRRR", part1: 119},
    {input: "BBFFBBFRLL", part1: 820},
    {input: "BFFFBBFRRR\nFFFBBBFRRR\nBBFFBBFRLL", part1: 820},
]);
