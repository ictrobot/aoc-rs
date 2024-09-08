use utils::prelude::*;

/// Finding numbers outside a list of ranges.
#[derive(Clone, Debug)]
pub struct Day20 {
    ranges: Vec<(u32, u32)>,
}

impl Day20 {
    pub fn new(input: &str, _: InputType) -> Result<Self, InputError> {
        let mut ranges = parser::u32()
            .then(parser::u32().with_prefix("-"))
            .map_res(|(min, max)| {
                if min <= max {
                    Ok((min, max))
                } else {
                    Err("end value cannot be less than start")
                }
            })
            .parse_lines(input)?;
        ranges.sort_unstable();
        Ok(Self { ranges })
    }

    #[must_use]
    pub fn part1(&self) -> u32 {
        let mut ip = 0;
        for &(start, end) in &self.ranges {
            if ip < start {
                break;
            }
            if end >= ip {
                ip = end.checked_add(1).expect("no allowed IPs");
            }
        }
        ip
    }

    #[must_use]
    pub fn part2(&self) -> u64 {
        let mut allowed = 0;
        let mut ip = 0;
        for &(start, end) in &self.ranges {
            if ip < start {
                allowed += u64::from(start - ip);
            }

            if end >= ip {
                let Some(next_ip) = end.checked_add(1) else {
                    // Range end is u32::MAX, no allowed IPs after final range to add
                    return allowed;
                };
                ip = next_ip;
            }
        }

        // Add on IPs after the end of the final range
        allowed + u64::from(u32::MAX - ip) + 1
    }
}

examples!(Day20 -> (u32, u64) [
    // Custom examples
    {
        input: "0-409354575\n\
            1005171202-1792978424\n\
            2396795992-4166223847\n\
            361276686-3509736453\n\
            2979588951-3460902073",
        part1: 4166223848,
        part2: 128743448
    },
    {
        input: "",
        part1: 0,
        part2: 4294967296,
    },
    {
        input: "0-0",
        part1: 1,
        part2: 4294967295,
    },
    {
        input: "4294967295-4294967295",
        part1: 0,
        part2: 4294967295,
    },
    {
        input: "0-4294967295",
        part2: 0,
    },
]);
