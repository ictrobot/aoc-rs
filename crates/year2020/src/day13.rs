use utils::number::{chinese_remainder, gcd};
use utils::prelude::*;

/// Finding when cycles first align.
///
/// Part 2 is similar to [2016 Day 15](../year2016/struct.Day15.html).
#[derive(Clone, Debug)]
pub struct Day13 {
    earliest: u64,
    buses: Vec<Bus>,
}

#[derive(Copy, Clone, Debug)]
struct Bus {
    id: u32,
    offset: i64,
}

impl Day13 {
    pub fn new(input: &str, _: InputType) -> Result<Self, InputError> {
        let (earliest, buses) = parser::u64()
            .with_eol()
            .then(
                parser::one_of((parser::nonzero_u32().map(Some), b'x'.map(|_| None)))
                    .repeat_fold(b',', 1, (Vec::new(), 0i64), |(mut buses, offset), id| {
                        if let Some(id) = id {
                            buses.push(Bus {
                                id: id.get(),
                                offset,
                            });
                        }
                        (buses, offset + 1)
                    })
                    .map(|(buses, _)| buses),
            )
            .parse_complete(input)?;

        if buses.is_empty() {
            return Err(InputError::new(input, 0, "expected at least one id"));
        }

        let mut period = 1i64;
        for bus in &buses {
            let id = i64::from(bus.id);
            if gcd(period, id) != 1 {
                return Err(InputError::new(input, 0, "ids must be pairwise coprime"));
            }
            period = period
                .checked_mul(id)
                .ok_or_else(|| InputError::new(input, 0, "combined period is too large"))?;
        }

        Ok(Self { earliest, buses })
    }

    #[must_use]
    pub fn part1(&self) -> u64 {
        let (id, wait) = self
            .buses
            .iter()
            .map(|bus| {
                let id = u64::from(bus.id);
                let wait = (id - self.earliest % id) % id;
                (id, wait)
            })
            .min_by_key(|&(_, wait)| wait)
            .unwrap();
        id * wait
    }

    #[must_use]
    pub fn part2(&self) -> u64 {
        chinese_remainder(
            self.buses.iter().map(|bus| -bus.offset),
            self.buses.iter().map(|bus| i64::from(bus.id)),
        )
        .expect("ids are pairwise coprime") as u64
    }
}

examples!(Day13 -> (u64, u64) [
    {input: "939\n7,13,x,x,59,x,31,19", part1: 295, part2: 1068781},
    {input: "0\n17,x,13,19", part2: 3417},
    {input: "0\n67,7,59,61", part2: 754018},
    {input: "0\n67,x,7,59,61", part2: 779210},
    {input: "0\n67,7,x,59,61", part2: 1261476},
    {input: "0\n1789,37,47,1889", part2: 1202161486},
]);
