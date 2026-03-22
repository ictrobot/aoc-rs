use utils::prelude::*;
use utils::str::TinyStr8;

/// Calculating the maximum output from a process chain.
#[derive(Clone, Debug)]
pub struct Day14 {
    order: Vec<usize>,
    reactions: Vec<Reaction>,
}

#[derive(Clone, Debug)]
struct Reaction {
    output_amount: u32,
    inputs: Vec<Input>,
}

#[derive(Clone, Copy, Debug)]
struct Input {
    index: usize,
    amount: u32,
}

const ORE: usize = 0;
const FUEL: usize = 1;
const PART2_LIMIT: u64 = 1_000_000_000_000;

impl Day14 {
    pub fn new(input: &str, _: InputType) -> Result<Self, InputError> {
        let component = parser::number_range(1..=u32::MAX)
            .then(parser::tinystr8(u8::is_ascii_uppercase).with_prefix(b' '));
        let reaction = component
            .repeat(", ", 1)
            .then(component.with_prefix(" => "))
            .with_consumed()
            .with_eol();

        let mut names = vec![
            const { TinyStr8::from_const(b"ORE") },
            const { TinyStr8::from_const(b"FUEL") },
        ];
        let mut reactions = vec![
            Some(Reaction {
                output_amount: 1,
                inputs: Vec::new(),
            }),
            None,
        ];

        for item in reaction.parse_iterator(input) {
            let ((inputs, (output_amount, output_index)), line) = item?;

            // Using a linear scan over a small Vec instead of a HashMap is ~15% faster.
            let mut intern = |name| {
                if let Some(index) = names.iter().position(|&n| n == name) {
                    return index;
                }
                names.push(name);
                reactions.push(None);
                reactions.len() - 1
            };

            let output = intern(output_index);
            let inputs = inputs
                .into_iter()
                .map(|(amount, name)| Input {
                    index: intern(name),
                    amount,
                })
                .collect();

            if output == ORE {
                return Err(InputError::new(
                    input,
                    line,
                    "expected ORE to be an input only",
                ));
            }
            if reactions[output].is_some() {
                return Err(InputError::new(
                    input,
                    line,
                    "duplicate reaction for chemical",
                ));
            }

            reactions[output] = Some(Reaction {
                output_amount,
                inputs,
            });
        }

        let Some(reactions): Option<Vec<Reaction>> = reactions.into_iter().collect() else {
            return Err(InputError::new(
                input,
                0,
                "expected every chemical to have a reaction",
            ));
        };

        let Some(order) = Self::reaction_order(&reactions) else {
            return Err(InputError::new(input, 0, "expected acyclic reaction graph"));
        };

        Ok(Self { order, reactions })
    }

    fn reaction_order(reactions: &[Reaction]) -> Option<Vec<usize>> {
        #[derive(Clone, Copy, Debug)]
        enum State {
            Unvisited,
            Visiting,
            Done,
        }

        fn depth(
            reactions: &[Reaction],
            chemical: usize,
            state: &mut [State],
            depths: &mut [usize],
        ) -> Option<usize> {
            match state[chemical] {
                State::Done => Some(depths[chemical]),
                State::Visiting => None,
                State::Unvisited if chemical == ORE => Some(depths[chemical]),
                State::Unvisited => {
                    state[chemical] = State::Visiting;

                    let mut max_input_depth = 0;
                    for input in &reactions[chemical].inputs {
                        max_input_depth =
                            max_input_depth.max(depth(reactions, input.index, state, depths)?);
                    }
                    depths[chemical] = max_input_depth + 1;

                    state[chemical] = State::Done;
                    Some(depths[chemical])
                }
            }
        }

        let mut state = vec![State::Unvisited; reactions.len()];
        let mut depths = vec![0; reactions.len()];
        depth(reactions, FUEL, &mut state, &mut depths)?;

        let mut order = (1..reactions.len())
            .filter(|&i| matches!(state[i], State::Done))
            .collect::<Vec<_>>();
        order.sort_unstable_by_key(|&i| depths[i]);

        Some(order)
    }

    #[must_use]
    pub fn part1(&self) -> u64 {
        self.ore_needed(1)
    }

    #[must_use]
    pub fn part2(&self) -> u64 {
        // Using a binary search in a window around PART2_LIMIT instead of from 1 to PART2_LIMIT
        // reduces ore_needed calls from 40 (log2(PART2_LIMIT)) to ~25
        let mut lower = PART2_LIMIT / self.ore_needed(1);
        let mut upper = 2 * lower.max(1);

        while self.ore_needed(upper) <= PART2_LIMIT {
            lower = upper;
            upper *= 2;
        }

        while lower + 1 < upper {
            let middle = (lower + upper) / 2;
            if self.ore_needed(middle) <= PART2_LIMIT {
                lower = middle;
            } else {
                upper = middle;
            }
        }

        lower
    }

    #[inline]
    fn ore_needed(&self, fuel_amount: u64) -> u64 {
        let mut demand = vec![0u64; self.reactions.len()];
        demand[FUEL] = fuel_amount;

        for &chemical in self.order.iter().rev() {
            let needed = demand[chemical];
            let reaction = &self.reactions[chemical];
            let batches = needed.div_ceil(u64::from(reaction.output_amount));
            for input in &reaction.inputs {
                demand[input.index] += u64::from(input.amount) * batches;
            }
        }

        demand[ORE]
    }
}

examples!(Day14 -> (u64, u64) [
    {file: "day14_example0.txt", part1: 31},
    {file: "day14_example1.txt", part1: 165},
    {file: "day14_example2.txt", part1: 13312, part2: 82892753},
    {file: "day14_example3.txt", part1: 180697, part2: 5586022},
    {file: "day14_example4.txt", part1: 2210736, part2: 460664},
]);
