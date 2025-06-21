use utils::bit::BitIterator;
use utils::prelude::*;

/// Scheduling steps with dependencies.
#[derive(Clone, Debug)]
pub struct Day07 {
    steps: u32,
    requirements: [u32; 26],
    workers: usize,
    base_duration: u32,
}

impl Day07 {
    pub fn new(input: &str, input_type: InputType) -> Result<Self, InputError> {
        let step = parser::byte_range(b'A'..=b'Z').map(|x| x - b'A');

        let mut steps = 0u32;
        let mut requirements = [0u32; 26];
        for item in step
            .with_prefix("Step ")
            .with_suffix(" must be finished before step ")
            .then(step.with_suffix(" can begin."))
            .with_suffix(parser::eol())
            .parse_iterator(input)
        {
            let (first, second) = item?;
            steps |= (1 << first) | (1 << second);
            requirements[second as usize] |= 1 << first;
        }

        Ok(Self {
            steps,
            requirements,
            workers: match input_type {
                InputType::Example => 2,
                InputType::Real => 5,
            },
            base_duration: match input_type {
                InputType::Example => 1,
                InputType::Real => 61,
            },
        })
    }

    #[must_use]
    pub fn part1(&self) -> String {
        let mut sequence = String::with_capacity(26);
        let mut todo = self.steps;
        while todo != 0 {
            let (s, bit) = BitIterator::ones(todo)
                .find(|&(r, _)| self.requirements[r as usize] & todo == 0)
                .expect("no valid sequence");
            todo &= !bit;
            sequence.push((b'A' + s as u8) as char);
        }
        sequence
    }

    #[must_use]
    pub fn part2(&self) -> u32 {
        let mut pending = self.steps;
        let mut incomplete = self.steps;
        let mut second = 0;
        let mut in_progress = Vec::with_capacity(self.workers);

        while incomplete != 0 {
            for (s, bit) in BitIterator::ones(pending)
                .filter(|&(s, _)| self.requirements[s as usize] & incomplete == 0)
                .take(self.workers - in_progress.len())
            {
                pending &= !bit;
                in_progress.push((second + self.base_duration + s, bit));
            }

            (second, _) = *in_progress.iter().min().expect("no valid sequence");
            in_progress.retain(|&(completed_at, bit)| {
                if completed_at == second {
                    incomplete &= !bit;
                    false
                } else {
                    true
                }
            });
        }

        second
    }
}

examples!(Day07 -> (&'static str, u32) [
    {file: "day07_example0.txt", part1: "CABDFE", part2: 15},
]);
