use std::collections::HashMap;
use utils::prelude::*;

/// Analysing guard sleep schedules.
#[derive(Clone, Debug)]
pub struct Day04 {
    guards: Vec<Guard>,
}

#[derive(Clone, Debug)]
struct Guard {
    id: u32,
    nights: Vec<u64>,
    most_frequent_minute: u32,
    most_frequent_count: u32,
}

#[derive(Debug)]
enum Event {
    BeginsShift(u32),
    FallsAsleep,
    WakesUp,
}

impl Day04 {
    pub fn new(input: &str, _: InputType) -> Result<Self, InputError> {
        let mut lines: Vec<&str> = input.lines().collect();
        lines.sort_unstable();

        let mut current_guard_idx = None;
        let mut guard_indexes = HashMap::new();
        let mut guards = Vec::new();
        for line in lines {
            let (_, _, _, min, event) = parser::number_range(1u32..=12)
                .with_prefix("[1518-")
                .with_suffix(b'-')
                .then(parser::number_range(1u32..=31).with_suffix(b' '))
                .then(parser::number_range(0u32..=23).with_suffix(b':'))
                .then(parser::number_range(0u32..=59).with_suffix("] "))
                .then(parser::one_of((
                    parser::u32()
                        .with_prefix("Guard #")
                        .with_suffix(" begins shift")
                        .map(Event::BeginsShift),
                    "falls asleep".map(|_| Event::FallsAsleep),
                    "wakes up".map(|_| Event::WakesUp),
                )))
                .parse_complete(line)
                .map_err(|e| InputError::new(input, line, e.into_source()))?;

            if let Event::BeginsShift(id) = event {
                let guard_idx = *guard_indexes.entry(id).or_insert_with(|| {
                    let idx = guards.len();
                    guards.push(Guard {
                        id,
                        nights: Vec::with_capacity(16),
                        most_frequent_count: 0,
                        most_frequent_minute: 0,
                    });
                    idx
                });
                guards[guard_idx].nights.push(0);
                current_guard_idx = Some(guard_idx);
                continue;
            }

            let Some(idx) = current_guard_idx else {
                return Err(InputError::new(input, line, "invalid first event"));
            };

            let night = guards[idx].nights.last_mut().unwrap();
            let mask = (1u64 << (60 - min)) - 1;
            if let Event::FallsAsleep = event {
                *night |= mask;
            } else {
                *night &= !mask;
            }
        }

        if guards.is_empty() {
            return Err(InputError::new(input, 0, "expected at least one guard"));
        }

        for guard in &mut guards {
            (guard.most_frequent_minute, guard.most_frequent_count) = (0..60)
                .map(|m| {
                    (
                        m,
                        guard
                            .nights
                            .iter()
                            .filter(|&&b| b & (1u64 << (59 - m)) != 0)
                            .count() as u32,
                    )
                })
                .max_by_key(|&(_, c)| c)
                .unwrap();
        }

        Ok(Self { guards })
    }

    #[must_use]
    pub fn part1(&self) -> u32 {
        self.select_guard(|g| g.nights.iter().map(|b| b.count_ones()).sum())
    }

    #[must_use]
    pub fn part2(&self) -> u32 {
        self.select_guard(|g| g.most_frequent_count)
    }

    fn select_guard(&self, key_fn: impl Fn(&&Guard) -> u32) -> u32 {
        let guard = self.guards.iter().max_by_key(key_fn).unwrap();
        guard.id * guard.most_frequent_minute
    }
}

examples!(Day04 -> (u32, u32) [
    {file: "day04_example0.txt", part1: 240, part2: 4455},
]);
