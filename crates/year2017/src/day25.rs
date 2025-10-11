use std::collections::VecDeque;
use utils::prelude::*;

/// Simulating a Turing machine.
#[derive(Clone, Debug)]
pub struct Day25 {
    start: State,
    steps: u32,
    rules: Vec<[Rule; 2]>,
}

#[derive(Copy, Clone, Debug)]
struct Rule {
    write_value: bool,
    move_dir: Direction,
    next_state: State,
}

parser::parsable_enum! {
    #[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
    enum Direction {
        "left" => Left = -1,
        "right" => Right = 1,
    }
}

parser::parsable_enum! {
    #[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
    enum State {
        "A" => A,
        "B" => B,
        "C" => C,
        "D" => D,
        "E" => E,
        "F" => F,
    }
}

impl Day25 {
    pub fn new(input: &str, _: InputType) -> Result<Self, InputError> {
        let rule = parser::byte_range(b'0'..=b'1')
            .map(|b| b == b'1')
            .with_prefix(parser::eol().then("    - Write the value "))
            .with_suffix(".".with_eol())
            .then(
                Direction::PARSER
                    .with_prefix("    - Move one slot to the ")
                    .with_suffix(".".with_eol()),
            )
            .then(
                State::PARSER
                    .with_consumed()
                    .with_prefix("    - Continue with state ")
                    .with_suffix(".".with_eol()),
            )
            .map(|(write_value, move_dir, (next_state, state_pos))| {
                (
                    Rule {
                        write_value,
                        move_dir,
                        next_state,
                    },
                    state_pos,
                )
            });

        let ((start, start_pos), steps, rules) = State::PARSER
            .with_consumed()
            .with_prefix("Begin in state ")
            .with_suffix(".".with_eol())
            .then(
                parser::u32()
                    .with_prefix("Perform a diagnostic checksum after ")
                    .with_suffix(" steps.".with_eol().with_eol()),
            )
            .then(
                State::PARSER
                    .with_consumed()
                    .with_prefix("In state ")
                    .with_suffix(":".with_eol())
                    .then(rule.with_prefix("  If the current value is 0:"))
                    .then(rule.with_prefix("  If the current value is 1:"))
                    .repeat(parser::eol(), 2),
            )
            .parse_complete(input)?;

        if let Some((_, &((_, pos), _, _))) = rules
            .iter()
            .enumerate()
            .find(|&(i, &((s, _), _, _))| s as usize != i)
        {
            return Err(InputError::new(input, pos, "rules are not in order"));
        }

        if start as usize >= rules.len() {
            return Err(InputError::new(input, start_pos, "invalid start state"));
        }

        if let Some(pos) = rules
            .iter()
            .map(|(_, r0, _)| r0)
            .chain(rules.iter().map(|(_, _, r1)| r1))
            .find_map(|&(r, pos)| (r.next_state as usize >= rules.len()).then_some(pos))
        {
            return Err(InputError::new(input, pos, "invalid next state"));
        }

        Ok(Self {
            start,
            steps,
            rules: rules
                .into_iter()
                .map(|(_, (r0, _), (r1, _))| [r0, r1])
                .collect(),
        })
    }

    #[must_use]
    pub fn part1(&self) -> u32 {
        // Increasing the element size means a cache hit skips more steps, but also reduces the
        // number of cache hits.
        type Element = u32;

        let mut tape: VecDeque<Element> = VecDeque::with_capacity(512);
        tape.push_back(0);
        let mut element_index = 0;
        let mut bit_index = 0;
        let mut state = self.start;

        #[derive(Debug)]
        struct StateTransition {
            from_state: State,
            from_element: Element,
            to_state: State,
            to_element: Element,
            steps: u32,
            element_index: usize,
        }
        // Index with cache[(state as usize << 1) | (bit_index > 0) as usize]
        let mut cache: [Vec<StateTransition>; 12] = Default::default();

        let mut step = 0;
        'outer: while step < self.steps {
            // Ensure the element index stays within the tape
            if element_index == 0 {
                tape.push_front(0);
                element_index += 1;
            } else if element_index == tape.len() {
                tape.push_back(0);
            }

            // Used the cached transition if this (state, starting bit index, from element) has been
            // seen previously
            for t in &cache[(state as usize * 2) | (bit_index > 0) as usize] {
                if t.from_element == tape[element_index] && t.steps + step <= self.steps {
                    while element_index > 0
                        && element_index < tape.len()
                        && t.from_state == state
                        && t.from_element == tape[element_index]
                        && t.steps + step <= self.steps
                    {
                        tape[element_index] = t.to_element;
                        element_index = element_index.wrapping_add(t.element_index);
                        state = t.to_state;
                        step += t.steps;
                    }

                    continue 'outer;
                }
            }

            let starting_element_index = element_index;
            let starting_bit_index = bit_index;
            let starting_element = tape[element_index];
            let starting_step = step;
            let starting_state = state;

            let mut element = starting_element;
            while step < self.steps && bit_index < Element::BITS {
                let rule = &self.rules[state as usize][((element >> bit_index) & 1) as usize];

                element =
                    (element & !(1 << bit_index)) | (Element::from(rule.write_value) << bit_index);
                bit_index = bit_index.wrapping_add_signed(rule.move_dir as i32);
                state = rule.next_state;
                step += 1;
            }
            tape[element_index] = element;

            // Calculate the correct bit index and new element index
            if step == self.steps {
                break;
            } else if bit_index == Element::BITS {
                bit_index = 0;
                element_index += 1;
            } else {
                bit_index = Element::BITS - 1;
                element_index -= 1;
            }

            // Cache the transition if the machine traversed the entire element
            if starting_bit_index == bit_index {
                cache[((starting_state as usize) << 1) | (bit_index > 0) as usize].push(
                    StateTransition {
                        from_state: starting_state,
                        from_element: starting_element,
                        to_state: state,
                        to_element: element,
                        steps: step - starting_step,
                        element_index: element_index.wrapping_sub(starting_element_index),
                    },
                );
            }
        }

        tape.iter().map(|&v| v.count_ones()).sum()
    }

    #[must_use]
    pub fn part2(&self) -> &'static str {
        "🎄"
    }
}

examples!(Day25 -> (u32, &'static str) [
    {file: "day25_example0.txt", part1: 3},
]);
