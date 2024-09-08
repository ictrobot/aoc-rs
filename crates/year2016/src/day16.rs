use utils::prelude::*;

/// Calculating a dragon curve checksum.
///
/// The checksum is based on the parity of each chunk, and each chunk is the length of the largest
/// power of 2 that divides the checksum's length.
///
/// Using the dragon bit parity function from
/// [/u/askalski's post "How to tame your dragon in under a millisecond"](https://www.reddit.com/r/adventofcode/comments/5ititq/2016_day_16_c_how_to_tame_your_dragon_in_under_a/)
/// we can calculate the parity of any length of the sequence:
/// - Calculate how many full repeats of the pattern
///   \[Original data]\[Dragon bit]\[Reversed & inverted data]\[Dragon bit] there are, and how
///   far through the pattern the final truncated repeat was.
/// - Calculate the parity of the full repeats of the original and the reversed & inverted data.
/// - Calculate the parity of the original data in the truncated repeat, if any.
/// - Calculate the parity of the reversed & inverted data in the truncated repeat, if any.
/// - Calculate the parity of all the dragon bits.
/// - XOR the four calculated parity values together.
///
/// This allows computing the parity from the start of the sequence to the end of each chunk, which
/// then can be used to find each chunk's parity by XORing each parity with the previous one.
#[derive(Clone, Debug)]
pub struct Day16<'a> {
    input: &'a str,
    input_type: InputType,
}

impl<'a> Day16<'a> {
    pub fn new(input: &'a str, input_type: InputType) -> Result<Self, InputError> {
        if let Some(index) = input.find(|c| c != '0' && c != '1') {
            return Err(InputError::new(input, index, "expected 0 or 1"));
        }

        Ok(Self { input, input_type })
    }

    #[must_use]
    pub fn part1(&self) -> String {
        self.checksum(match self.input_type {
            InputType::Example => 20,
            InputType::Real => 272,
        })
    }

    #[must_use]
    pub fn part2(&self) -> String {
        self.checksum(35651584)
    }

    fn checksum(&self, length: u32) -> String {
        let chunk_length = 1 << length.trailing_zeros();
        let input_length = self.input.len() as u32;

        // Expanded data repeats the following pattern:
        // [Original data][Dragon bit][Reversed & inverted data][Dragon bit]
        let pattern_length = 2 * (input_length + 1);

        let mut previous = false;
        let mut output = String::with_capacity((length / chunk_length) as usize);
        for i in (chunk_length..=length).step_by(chunk_length as usize) {
            let mut parity = false;

            // Work out how many complete pattern repeats there are up until this point, and how
            // far through the pattern the final truncated repeat is
            let complete_repeats = i / pattern_length;
            let mut truncated = i % pattern_length;
            let mut dragon_bits = 2 * complete_repeats;

            // Calculate parity of all the repeated original + reversed & inverted sections
            parity ^= (complete_repeats * input_length) % 2 == 1;

            // Calculate parity of original data in the final truncated repeat
            if truncated > 0 {
                let original_length = truncated.min(input_length);
                let ones = self
                    .input
                    .bytes()
                    .take(original_length as usize)
                    .filter(|&b| b == b'1')
                    .count();
                parity ^= ones % 2 == 1;
                truncated -= original_length;
            }

            // Extra dragon bit in final truncated repeat
            if truncated > 0 {
                dragon_bits += 1;
                truncated -= 1;
            }

            // Calculate parity of the reversed & inverted data in the final truncated repeat
            if truncated > 0 {
                let mirrored_inverted_length = truncated.min(input_length);
                let ones = self
                    .input
                    .bytes()
                    .rev()
                    .take(mirrored_inverted_length as usize)
                    .filter(|&b| b == b'0')
                    .count();
                parity ^= ones % 2 == 1;
            }

            // Truncated repeat can't have a final dragon bit (otherwise it wouldn't be incomplete)

            // Calculate parity of all the dragon bits
            parity ^= Self::dragon_parity(dragon_bits);

            if parity ^ previous {
                output.push('0');
            } else {
                output.push('1');
            }
            previous = parity;
        }

        output
    }

    /// See https://www.reddit.com/r/adventofcode/comments/5ititq/2016_day_16_c_how_to_tame_your_dragon_in_under_a/
    fn dragon_parity(n: u32) -> bool {
        let gray = n ^ (n >> 1);
        ((n & gray).count_ones() ^ gray) & 1 != 0
    }
}

examples!(Day16<'_> -> (&'static str, &'static str) [
    {input: "10000", part1: "01100"},
]);
