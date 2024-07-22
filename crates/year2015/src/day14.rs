use utils::prelude::*;

/// Reindeer speeds.
#[derive(Clone, Debug)]
pub struct Day14 {
    reindeer: Vec<(u32, u32, u32)>,
    duration: u32,
}

impl Day14 {
    pub fn new(input: &str, input_type: InputType) -> Result<Self, InputError> {
        Ok(Self {
            reindeer: parser::u32()
                .with_prefix(" can fly ")
                .with_prefix(parser::take_while(u8::is_ascii_alphabetic))
                .then(parser::u32().with_prefix(" km/s for "))
                .then(parser::u32().with_prefix(" seconds, but then must rest for "))
                .with_suffix(" seconds.")
                .parse_lines(input)?,
            duration: match input_type {
                InputType::Example => 1000,
                InputType::Real => 2503,
            },
        })
    }

    #[must_use]
    pub fn part1(&self) -> u32 {
        self.reindeer
            .iter()
            .map(|&r| Self::distance(self.duration, r))
            .max()
            .unwrap()
    }

    #[must_use]
    pub fn part2(&self) -> u32 {
        let mut scores = vec![0; self.reindeer.len()];
        let mut distances = vec![0; self.reindeer.len()];
        for t in 1..=self.duration {
            let mut max_dist = 0;
            for (&r, dist) in self.reindeer.iter().zip(distances.iter_mut()) {
                *dist = Self::distance(t, r);
                max_dist = max_dist.max(*dist);
            }

            for (score, dist) in scores.iter_mut().zip(distances.iter_mut()) {
                if *dist == max_dist {
                    *score += 1;
                }
            }
        }
        scores.into_iter().max().unwrap()
    }

    fn distance(duration: u32, (speed, fly_time, rest_time): (u32, u32, u32)) -> u32 {
        let cycle_time = fly_time + rest_time;
        let cycles = duration / cycle_time;
        speed * (cycles * fly_time + (duration - (cycles * cycle_time)).min(fly_time))
    }
}

examples!(Day14 -> (u32, u32) [
    {
        input: "Comet can fly 14 km/s for 10 seconds, but then must rest for 127 seconds.\n\
            Dancer can fly 16 km/s for 11 seconds, but then must rest for 162 seconds.",
        part1: 1120,
        part2: 689,
    },
]);
