use std::collections::HashSet;
use utils::array::ArrayVec;
use utils::prelude::*;

/// Molecule string replacements.
///
/// Part 2 assumes there is only one possible number of steps but does not assume the `Rn` `Y` `Ar`
/// bracket structure or use the formula. Instead, it uses an optimized
/// [Earley parser](https://en.wikipedia.org/wiki/Earley_parser), which ensures the molecule can be
/// created from the provided rules.
#[derive(Clone, Debug)]
pub struct Day19 {
    rules: Vec<(Option<Atom>, ArrayVec<Atom, 8>)>,
    molecule: Vec<Atom>,
}

parser::parsable_enum! {
    #[derive(Copy, Clone, Debug, Default, PartialEq, Eq, Hash)]
    #[repr(u8)]
    enum Atom {
        #[default]
        "Al" => Al,
        "Ar" => Ar,
        "B" => B,
        "Ca" => Ca,
        "C" => C,
        "F" => F,
        "H" => H,
        "Mg" => Mg,
        "N" => N,
        "O" => O,
        "P" => P,
        "Rn" => Rn,
        "Si" => Si,
        "Th" => Th,
        "Ti" => Ti,
        "Y" => Y,
    }
}

const _: () = {
    assert!(Atom::ALL.len() <= 16);
};

impl Day19 {
    pub fn new(input: &str, _: InputType) -> Result<Self, InputError> {
        let Some((rules_str, molecule)) = input
            .rsplit_once("\n\n")
            .or_else(|| input.rsplit_once("\r\n\r\n"))
        else {
            return Err(InputError::new(
                input,
                0,
                "expected rules then a blank line then the molecule",
            ));
        };

        let rules = Atom::PARSER
            .map(Some)
            .or(b'e'.map(|_| None))
            .with_suffix(" => ")
            .then(Atom::PARSER.repeat_arrayvec(parser::noop(), 1))
            .parse_lines(rules_str)?;

        if rules.len() > 64 {
            return Err(InputError::new(input, rules_str.len(), "too many rules"));
        }

        Ok(Self {
            rules,
            molecule: Atom::PARSER.parse_all(molecule)?,
        })
    }

    #[must_use]
    pub fn part1(&self) -> usize {
        let mut set = HashSet::new();
        for (from, to) in &self.rules {
            let Some(from) = *from else { continue };
            let new_length = self.molecule.len() + to.len() - 1;
            for i in 0..self.molecule.len() {
                if self.molecule[i] == from {
                    let mut molecule = Vec::with_capacity(new_length);
                    molecule.extend_from_slice(&self.molecule[..i]);
                    molecule.extend_from_slice(to);
                    molecule.extend_from_slice(&self.molecule[i + 1..]);

                    // `.into_iter().map(|x| x as u8).collect::<Vec<_>>()` makes this function 2-3x
                    // faster as the std::hash::Hash implementation for u8 implements hash_slice
                    // efficiently using a single call to write, and the into_iter-map-collect chain
                    // is a no-op. It isn't possible to implement Hash::hash_slice for Atom so
                    // efficiently without unsafe code / transmute.
                    set.insert(molecule.into_iter().map(|x| x as u8).collect::<Vec<_>>());
                }
            }
        }
        set.len()
    }

    #[must_use]
    pub fn part2(&self) -> u32 {
        #[derive(Copy, Clone, Debug)]
        struct State {
            rule: usize,
            dot: usize,
            origin: usize,
        }

        // Store the chart as a list of state lists at each position, plus a bitset for the current
        // and next positions. This works well as only the current and next position sets are ever
        // updated, and the bitset makes duplicate checking fast. Previous sets are only ever
        // iterated over. The current list is also reused as a queue of states to process.
        let mut chart = vec![Vec::new(); self.molecule.len() + 1];

        // Indexed by bitset[origin][dot] & (1 << rule):
        // - 9 possible dot values (0-8 inclusive, enforced by ArrayVec N),
        // - 64 possible rules (checked in new).
        let mut current_bitset = vec![[0u64; 9]; self.molecule.len() + 1];
        let mut next_bitset = vec![[0u64; 9]; self.molecule.len() + 1];

        // Preprocess the rules into separate lists by the LHS, populating e rules into the initial
        // set.
        let mut rules_by_lhs = vec![Vec::new(); 16];
        for (i, (lhs, _)) in self.rules.iter().enumerate() {
            if let Some(lhs) = *lhs {
                rules_by_lhs[lhs as usize].push(i);
            } else {
                let state = State {
                    rule: i,
                    dot: 0,
                    origin: 0,
                };
                current_bitset[state.origin][state.dot] |= 1 << state.rule;
                chart[0].push((state, 1));
            }
        }

        // Optimization: Only do predictions once per atom per position.
        let mut predictions_done = 0u16;
        // Optimization: Only do completions once per (origin, atom) per position.
        let mut completions_done = vec![0u16; self.molecule.len() + 1];

        for pos in 0..chart.len() {
            let mut set_idx = 0;
            while let Some(&(state, steps)) = chart[pos].get(set_idx) {
                let (lhs, rhs) = &self.rules[state.rule];

                if state.dot < rhs.len() {
                    // Prediction
                    if predictions_done & (1 << rhs[state.dot] as usize) == 0 {
                        predictions_done |= 1 << rhs[state.dot] as usize;

                        for &i in &rules_by_lhs[rhs[state.dot] as usize] {
                            let new = State {
                                rule: i,
                                dot: 0,
                                origin: pos,
                            };
                            if current_bitset[new.origin][new.dot] & (1 << new.rule) == 0 {
                                current_bitset[new.origin][new.dot] |= 1 << new.rule;
                                chart[pos].push((new, 1));
                            }
                        }
                    }

                    // Scanning
                    if self.molecule.get(pos) == Some(&rhs[state.dot]) {
                        let new = State {
                            rule: state.rule,
                            dot: state.dot + 1,
                            origin: state.origin,
                        };
                        if next_bitset[new.origin][new.dot] & (1 << new.rule) == 0 {
                            next_bitset[new.origin][new.dot] |= 1 << new.rule;
                            chart[pos + 1].push((new, steps));
                        }
                    }
                } else if let Some(lhs) = *lhs {
                    // Completion
                    if completions_done[state.origin] & (1 << lhs as usize) == 0 {
                        completions_done[state.origin] |= 1 << lhs as usize;

                        let [current_chart, origin_chart] = chart
                            .get_disjoint_mut([pos, state.origin])
                            .expect("origin must be less than pos");

                        for (prev_state, prev_steps) in origin_chart.iter() {
                            let (_, prev_rhs) = &self.rules[prev_state.rule];
                            if prev_state.dot < prev_rhs.len() && prev_rhs[prev_state.dot] == lhs {
                                let new = State {
                                    rule: prev_state.rule,
                                    dot: prev_state.dot + 1,
                                    origin: prev_state.origin,
                                };
                                if current_bitset[new.origin][new.dot] & (1 << new.rule) == 0 {
                                    current_bitset[new.origin][new.dot] |= 1 << new.rule;
                                    current_chart.push((new, steps + prev_steps));
                                }
                            }
                        }
                    }
                } else if pos == self.molecule.len() {
                    // Completion of a start rule consuming the entire molecule
                    return steps;
                }

                set_idx += 1;
            }

            (current_bitset, next_bitset) = (next_bitset, current_bitset);
            next_bitset[..=pos].fill([0; 9]);

            // Reset optimization caches for the next position
            predictions_done = 0u16;
            completions_done[..pos].fill(0);
        }

        panic!("no solution found");
    }
}

examples!(Day19 -> (usize, u32) [
    {input: "H => HO\nH => OH\nO => HH\n\nHOH", part1: 4},
    {input: "e => H\ne => O\nH => HO\nH => OH\nO => HH\n\nHOH", part2: 3},
    {input: "e => H\ne => O\nH => HO\nH => OH\nO => HH\n\nHOHOHO", part2: 6},
]);
