use std::collections::{HashMap, HashSet, VecDeque};
use utils::bit::BitIterator;
use utils::prelude::*;

/// Minimizing steps to safely rearrange generators and microchips.
///
/// The key optimization is that states are equivalent if swapping the positions of
/// generator-microchip pairs would make them equal.
///
/// Additionally, work out which generators/microchips are safe to move, instead of wasting time
/// checking if states are valid.
#[derive(Clone, Debug)]
pub struct Day11 {
    floors: [Floor; 4],
    types: usize,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
struct State {
    floors: [Floor; 4],
    elevator: u8,
    steps: u16,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Hash)]
struct Floor {
    generators: u8,
    microchips: u8,
}

impl Day11 {
    pub fn new(input: &str, _: InputType) -> Result<Self, InputError> {
        if input.lines().count() != 4 {
            return Err(InputError::new(input, 0, "expected 4 floors"));
        }

        let mut floors = [Floor::default(); 4];
        let mut types = HashMap::new();
        for (
            line,
            Floor {
                generators,
                microchips,
            },
        ) in input.lines().zip(&mut floors)
        {
            for (matches, field) in [
                (line.match_indices(" generator"), generators),
                (line.match_indices("-compatible microchip"), microchips),
            ] {
                for (index, _) in matches {
                    let Some((_, type_str)) = line[..index].rsplit_once(' ') else {
                        return Err(InputError::new(input, index, "expected type"));
                    };

                    let types_len = types.len();
                    let type_index = *types.entry(type_str).or_insert(types_len);
                    if type_index >= 5 {
                        return Err(InputError::new(input, index, "too many types"));
                    }

                    *field |= 1 << type_index;
                }
            }
        }

        Ok(Self {
            floors,
            types: types.len(),
        })
    }

    #[must_use]
    pub fn part1(&self) -> u16 {
        Self::minimum_steps(self.floors, self.types)
    }

    #[must_use]
    pub fn part2(&self) -> u16 {
        let mut floors = self.floors;
        floors[0].generators |= 0b11 << self.types;
        floors[0].microchips |= 0b11 << self.types;
        Self::minimum_steps(floors, self.types + 2)
    }

    fn minimum_steps(floors: [Floor; 4], types: usize) -> u16 {
        // Ensure the current state is valid, as the code below assumes the current state is always valid
        if types > 7 {
            panic!("only 7 types supported"); // An eighth could be supported by updating to_unique
        }
        for f in floors {
            if f.generators != 0 && (f.microchips & !f.generators) != 0 {
                // Triggered running part 2 on the example input, as it starts with unpaired
                // microchips on the first floor
                panic!("invalid start state");
            }
        }

        let mut queue = VecDeque::with_capacity(1024);
        let mut visited = HashSet::with_capacity(10240);

        let start = State {
            floors,
            elevator: 0,
            steps: 0,
        };
        queue.push_back(start);
        visited.insert(start.to_unique());

        let all_types = !(u8::MAX << types);
        while let Some(state) = queue.pop_front() {
            if state.floors[3].microchips == all_types && state.floors[3].generators == all_types {
                return state.steps;
            }

            let src = state.floors[state.elevator as usize];
            let src_pairs = src.generators & src.microchips;
            let src_unpaired_generators = src.generators & !src.microchips;

            // If any generators are moved from the current floor, work out which can be moved and
            // which must be moved.
            let (src_gen_can_move, src_gen_must_move) = if src.generators == 0 {
                // No generators to move
                (0, 0)
            } else if src_pairs.count_ones() > 2
                || (src_pairs.count_ones() == 2 && src_unpaired_generators != 0)
                || (src_pairs.count_ones() == 1 && src_unpaired_generators.count_ones() >= 2)
            {
                // Only possible to move unpaired generators, as moving one of the paired generators
                // will leave a generator and an incompatible microchip behind
                (src_unpaired_generators, 0)
            } else if src_pairs.count_ones() == 2 && src_unpaired_generators == 0 {
                // Both paired generators must be moved, as leaving one behind will break
                // the incompatible microchip
                (src_pairs, src_pairs)
            } else if src_pairs.count_ones() == 1 && src_unpaired_generators.count_ones() <= 1 {
                // Unpaired generator must be moved (if present), paired generator can be moved
                (src.generators, src_unpaired_generators)
            } else {
                // All generators can be moved
                (src.generators, 0)
            };

            for elevator in [state.elevator + 1, state.elevator.saturating_sub(1)] {
                if elevator == state.elevator || elevator >= 4 {
                    continue;
                }

                // Don't go down to empty flows
                if elevator < state.elevator
                    && ((elevator == 0 && state.floors[0].empty())
                        || (elevator == 1 && state.floors[0].empty() && state.floors[1].empty()))
                {
                    continue;
                }

                let mut try_move = |generators: u8, microchips: u8| {
                    let mut next_state = state;
                    next_state.floors[state.elevator as usize].generators &= !generators;
                    next_state.floors[state.elevator as usize].microchips &= !microchips;

                    next_state.floors[elevator as usize].generators |= generators;
                    next_state.floors[elevator as usize].microchips |= microchips;

                    next_state.elevator = elevator;
                    next_state.steps += 1;

                    if visited.insert(next_state.to_unique()) {
                        queue.push_back(next_state);
                    }
                };

                let dst = state.floors[elevator as usize];
                let dst_unpaired_microchips = dst.microchips & !dst.generators;

                // Try moving a pair
                if src_pairs != 0 && dst_unpaired_microchips == 0 {
                    let pair = 1 << src_pairs.trailing_zeros();
                    try_move(pair, pair);
                }

                // Try moving generators
                if src_gen_can_move != 0 {
                    let can_move = src_gen_can_move;
                    let mut must_move = src_gen_must_move;
                    if dst_unpaired_microchips != 0 {
                        // Only safe to move generators if also moving generators required to make pairs
                        must_move |= dst_unpaired_microchips;
                    }

                    if must_move.count_ones() <= 2 && (must_move & !can_move) == 0 {
                        if elevator > state.elevator {
                            // Going up, move as many generators as possible
                            if must_move.count_ones() == 2
                                || (must_move.count_ones() == 1 && (can_move & !must_move) == 0)
                            {
                                // 2 must-move generators, or 1 must-move generator if there are no other movable generators
                                try_move(must_move, 0);
                            } else if must_move.count_ones() == 1 {
                                // Any combination of the 1 must-move generator + a can-move generator
                                for (_, g1) in BitIterator::ones(can_move & !must_move) {
                                    try_move(must_move | g1, 0);
                                }
                            } else if can_move.count_ones() == 1 {
                                // 1 can-move generator only
                                try_move(can_move, 0);
                            } else {
                                // Any combination of 2 can-move generators
                                for (_, g1) in BitIterator::ones(can_move) {
                                    for (_, g2) in BitIterator::ones(can_move & (g1 - 1)) {
                                        try_move(g1 | g2, 0);
                                    }
                                }
                            }
                        } else {
                            // Going down, move as few generators as possible
                            if must_move != 0 {
                                // Move the must-move generators
                                try_move(must_move, 0);
                            } else {
                                // Any of the can-move generators by itself
                                for (_, g1) in BitIterator::ones(can_move) {
                                    try_move(g1, 0);
                                }
                            }
                        }
                    }
                }

                // Try moving microchips
                if src.microchips != 0 {
                    let mut to_move = src.microchips;
                    if dst.generators != 0 {
                        // Only safe to move microchips to make pairs
                        to_move &= dst.generators;
                    }

                    if to_move != 0 {
                        if elevator > state.elevator {
                            // Going up, move as many microchips as possible
                            if to_move.count_ones() >= 2 {
                                // Any combination of 2 microchips
                                for (_, m1) in BitIterator::ones(to_move) {
                                    for (_, m2) in BitIterator::ones(to_move & (m1 - 1)) {
                                        try_move(0, m1 | m2);
                                    }
                                }
                            } else {
                                // Only 1 microchip to move
                                try_move(0, to_move);
                            }
                        } else {
                            // Going down, only move one microchip
                            for (_, m1) in BitIterator::ones(to_move) {
                                try_move(0, m1);
                            }
                        }
                    }
                }
            }
        }

        panic!("no solution found")
    }
}

impl State {
    #[inline]
    fn to_unique(self) -> u64 {
        // States are equivalent if swapping the positions of generator-microchip pairs would make
        // them equal. Store visited states by converting the position of each pair into a single
        // byte (top 4 bits for the microchip's floor, bottom 4 bits for the generator's floor),
        // then sorting the positions.
        let mut output = [0u8; 8];
        output[0] = self.elevator;

        for (i, out) in output[1..].iter_mut().enumerate() {
            for (f, floor) in self.floors.iter().enumerate() {
                *out |= if floor.generators & (1 << i) != 0 {
                    1 << f
                } else {
                    0
                };
                *out |= if floor.microchips & (1 << i) != 0 {
                    1 << (f + 4)
                } else {
                    0
                };
            }
        }
        output[1..].sort_unstable();

        // Hashing one u64 seems to be faster than hashing 8 or 9 bytes, but as one of the bytes is
        // needed to store the elevator position, only 7 types are supported.
        u64::from_ne_bytes(output)
    }
}

impl Floor {
    fn empty(self) -> bool {
        self.generators == 0 && self.microchips == 0
    }
}

examples!(Day11 -> (u16, u16) [
    {
        input: "The first floor contains a hydrogen-compatible microchip and a lithium-compatible microchip.\n\
            The second floor contains a hydrogen generator.\n\
            The third floor contains a lithium generator.\n\
            The fourth floor contains nothing relevant.",
        part1: 11
    },
]);
