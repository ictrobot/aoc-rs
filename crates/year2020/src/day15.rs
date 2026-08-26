use utils::prelude::*;

/// Generating a sequence from previous occurrences.
#[derive(Clone, Debug)]
pub struct Day15 {
    starting: Vec<u32>,
}

const DENSE_THRESHOLD: u32 = 65_536;
const SPARSE_THRESHOLD: u32 = 16_777_216;
const SPARSE_BUCKET_SHIFT: u32 = 16;

impl Day15 {
    pub fn new(input: &str, _: InputType) -> Result<Self, InputError> {
        Ok(Self {
            starting: parser::number_range(0..=2019)
                .repeat(b',', 1)
                .parse_complete(input)?,
        })
    }

    #[must_use]
    pub fn part1(&self) -> u32 {
        self.number_at::<2020>()
    }

    #[must_use]
    pub fn part2(&self) -> u32 {
        self.number_at::<30_000_000>()
    }

    #[inline]
    fn number_at<const TARGET: u32>(&self) -> u32 {
        if TARGET as usize <= self.starting.len() {
            return self.starting[TARGET as usize - 1];
        }

        let dense_limit = TARGET.min(DENSE_THRESHOLD);
        let mut dense = vec![0u32; dense_limit as usize];
        let middle_limit = TARGET.min(SPARSE_THRESHOLD);
        let mut middle = vec![0u32; middle_limit.saturating_sub(DENSE_THRESHOLD) as usize];
        let sparse_buckets = TARGET
            .saturating_sub(SPARSE_THRESHOLD)
            .div_ceil(1 << SPARSE_BUCKET_SHIFT) as usize;
        let mut sparse = vec![Vec::new(); sparse_buckets];
        let mut seen = vec![0u64; TARGET.div_ceil(64) as usize];

        for (turn, &number) in self.starting[..self.starting.len() - 1].iter().enumerate() {
            dense[number as usize] = turn as u32 + 1;
        }

        let mut number = *self.starting.last().unwrap();
        let mut turn = self.starting.len() as u32;
        while turn < TARGET {
            let previous = if number < DENSE_THRESHOLD {
                std::mem::replace(&mut dense[number as usize], turn)
            } else if number < SPARSE_THRESHOLD {
                // Track seen numbers in a bitset to reduce random timestamp reads
                let base = number as usize / 64;
                let mask = 1 << (number % 64);
                if seen[base] & mask == 0 {
                    seen[base] |= mask;
                    middle[(number - DENSE_THRESHOLD) as usize] = turn;
                    0
                } else {
                    std::mem::replace(&mut middle[(number - DENSE_THRESHOLD) as usize], turn)
                }
            } else {
                // Store rarely repeated numbers sparsely to shrink the timestamp array
                std::hint::cold_path();
                let base = number as usize / 64;
                let mask = 1 << (number % 64);
                let bucket = ((number - SPARSE_THRESHOLD) >> SPARSE_BUCKET_SHIFT) as usize;
                if seen[base] & mask == 0 {
                    seen[base] |= mask;
                    sparse[bucket].push((number, turn));
                    0
                } else {
                    let (_, v) = sparse[bucket]
                        .iter_mut()
                        .find(|&&mut (n, _)| n == number)
                        .expect("expected previously seen number");
                    std::mem::replace(v, turn)
                }
            };

            if previous == 0 {
                turn += 1;

                // Process the known zero inline to avoid another pass through the range checks
                let previous = std::mem::replace(&mut dense[0], turn);
                number = if previous == 0 {
                    std::hint::cold_path();
                    0
                } else {
                    turn - previous
                };
                turn += 1;
            } else {
                number = turn - previous;
                turn += 1;
            }
        }

        if turn == TARGET {
            number
        } else if turn == TARGET + 1 {
            // Target was the zero processed as part of the double turn
            0
        } else {
            panic!("Reached turn {turn} but target is {TARGET}");
        }
    }
}

examples!(Day15 -> (u32, u32) [
    {input: "0,3,6", part1: 436, part2: 175594},
    {input: "1,3,2", part1: 1, part2: 2578},
    {input: "2,1,3", part1: 10, part2: 3544142},
    {input: "1,2,3", part1: 27, part2: 261214},
    {input: "2,3,1", part1: 78, part2: 6895259},
    {input: "3,2,1", part1: 438, part2: 18},
    {input: "3,1,2", part1: 1836, part2: 362},
]);
