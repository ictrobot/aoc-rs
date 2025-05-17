use utils::bit::BitIterator;
use utils::prelude::*;

/// Finding the longest and strongest bridge.
#[derive(Clone, Debug)]
pub struct Day24 {
    part1: u32,
    part2: u32,
}

impl Day24 {
    pub fn new(input: &str, _: InputType) -> Result<Self, InputError> {
        let mut components: Vec<[u32; 2]> = parser::number_range(0..=63)
            .repeat_n(b'/')
            .parse_lines(input)?;
        if components.len() >= 64 {
            return Err(InputError::new(input, input.len(), "too many components"));
        }

        // Sort all the components with matching pin counts first
        components.sort_unstable_by_key(|&[a, b]| a != b);

        let mut bitsets = [0u64; 64];
        let mut sums = [0u32; 64];
        for (i, &[a, b]) in components.iter().enumerate() {
            bitsets[a as usize] |= 1 << i;
            bitsets[b as usize] |= 1 << i;
            sums[i] = a + b;
        }
        if bitsets[0] == 0 {
            return Err(InputError::new(input, 0, "no zero pin components"));
        }

        let mut out = [0u32; 64];
        Self::search(&bitsets, &sums, &mut out, 0, 0, 0, 0);

        Ok(Self {
            part1: out.iter().max().copied().unwrap(),
            part2: out.iter().rfind(|&&s| s > 0).copied().unwrap(),
        })
    }

    fn search(
        bitsets: &[u64],
        sum: &[u32],
        best: &mut [u32],
        pins: u32,
        used: u64,
        strength: u32,
        length: usize,
    ) {
        let remaining = bitsets[pins as usize] & !used;
        if remaining == 0 {
            best[length] = best[length].max(strength);
            return;
        }

        for (component_index, component_bit) in BitIterator::ones(remaining) {
            let component_sum = sum[component_index as usize];

            Self::search(
                bitsets,
                sum,
                best,
                component_sum - pins,
                used | component_bit,
                strength + component_sum,
                length + 1,
            );

            // It is always optimal to choose a component with matching pins if available
            if component_sum == pins * 2 {
                break;
            }
        }
    }

    #[must_use]
    pub fn part1(&self) -> u32 {
        self.part1
    }

    #[must_use]
    pub fn part2(&self) -> u32 {
        self.part2
    }
}

examples!(Day24 -> (u32, u32) [
    {input: "0/2\n2/2\n2/3\n3/4\n3/5\n0/1\n10/1\n9/10", part1: 31, part2: 19},
]);
