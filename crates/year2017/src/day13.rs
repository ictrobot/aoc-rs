use std::collections::BTreeMap;
use utils::number;
use utils::prelude::*;

/// Finding the gap in the firewall.
///
/// Similar to [2016 day 15](../year2016/struct.Day15.html), but instead of a system of linear
/// simultaneous congruences, it is a system of simultaneous modular inequalities.
#[derive(Clone, Debug)]
pub struct Day13 {
    layers: Vec<(u32, u32)>,
}

impl Day13 {
    pub fn new(input: &str, _: InputType) -> Result<Self, InputError> {
        Ok(Self {
            layers: parser::u32()
                .with_suffix(": ")
                .then(parser::u32())
                .parse_lines(input)?,
        })
    }

    #[must_use]
    pub fn part1(&self) -> u32 {
        self.layers
            .iter()
            .map(|&(depth, range)| {
                let period = (range - 1) * 2;
                if depth % period == 0 {
                    depth * range
                } else {
                    0
                }
            })
            .sum()
    }

    #[must_use]
    pub fn part2(&self) -> u32 {
        // Each key represents a modulus and maps to a list of values where
        // delay % modulus can't equal the value
        let mut constraints = BTreeMap::new();
        for &(depth, range) in &self.layers {
            let modulus = (range as i32 - 1) * 2;
            let disallowed_value = (-(depth as i32)).rem_euclid(modulus);

            constraints
                .entry(modulus)
                .or_insert_with(Vec::new)
                .push(disallowed_value);
        }

        // Find all the possible delays % lcm which meet the above constraints
        let mut lcm = 1;
        let mut possible_delays = vec![0];
        for (modulus, disallowed_values) in constraints {
            let new_lcm = number::lcm(modulus, lcm);
            possible_delays = possible_delays
                .into_iter()
                .flat_map(|delay| {
                    (delay..new_lcm)
                        .step_by(lcm as usize)
                        .filter(|&i| !disallowed_values.contains(&(i % modulus)))
                })
                .collect();
            lcm = new_lcm;
        }

        *possible_delays.iter().min().unwrap() as u32
    }
}

examples!(Day13 -> (u32, u32) [
    {input: "0: 3\n1: 2\n4: 4\n6: 4", part1: 24, part2: 10},
]);
