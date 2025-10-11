use utils::prelude::*;

/// Simulating 1D cellular automata.
#[derive(Clone, Debug)]
pub struct Day12 {
    initial: State,
    rules: u32,
}

#[derive(Copy, Clone, Debug)]
struct State {
    pots: [u64; WIDTH],
    start: i64,
    len: usize,
    sum: i64,
}

const WIDTH: usize = 4;
const PART1_GENERATIONS: i64 = 20;
const PART2_GENERATIONS: i64 = 50_000_000_000;

impl Day12 {
    pub fn new(input: &str, _: InputType) -> Result<Self, InputError> {
        let cell = parser::byte_map!(
            b'#' => true,
            b'.' => false,
        );

        let (initial, rules) = cell
            .repeat_arrayvec::<100, _>(parser::noop(), 1)
            .with_prefix("initial state: ")
            .with_eol()
            .with_eol()
            .then(
                cell.repeat_n::<5, _>(parser::noop())
                    .with_suffix(" => ")
                    .then(cell)
                    .repeat_arrayvec::<32, _>(parser::eol(), 1),
            )
            .parse_complete(input)?;

        let mut pots = [0; WIDTH];
        let mut sum = 0;
        for (i, &b) in initial.iter().enumerate() {
            pots[i / 64] |= (b as u64) << (i % 64);
            sum += i64::from(b) * i as i64;
        }

        let mut rules_mask = 0;
        for &(lhs, rhs) in rules.iter() {
            let index = lhs
                .iter()
                .enumerate()
                .fold(0, |acc, (i, &b)| acc | (b as u32) << i);
            rules_mask |= (rhs as u32) << index;
        }

        Ok(Self {
            initial: State {
                pots,
                start: 0,
                len: initial.len(),
                sum,
            },
            rules: rules_mask,
        })
    }

    #[must_use]
    pub fn part1(&self) -> i64 {
        let mut state = self.initial;
        for _ in 0..PART1_GENERATIONS {
            state = state.next(self.rules);
        }
        state.sum
    }

    #[must_use]
    pub fn part2(&self) -> i64 {
        let mut state = self.initial;
        for remaining in (PART2_GENERATIONS - 1000..PART2_GENERATIONS).rev() {
            let next = state.next(self.rules);

            // In the input the pattern eventually stabilizes and just moves to the right each
            // iteration
            if state.len == next.len && state.pots == next.pots {
                return next.sum + remaining * (next.sum - state.sum);
            }

            state = next;
        }
        panic!("no solution found: reached generation limit")
    }
}

impl State {
    fn next(&self, rules: u32) -> State {
        if self.len + 4 >= WIDTH * 64 {
            panic!("no solution found: reached width limit");
        }

        let (mut pots, mut start, mut len, mut sum) = ([0; WIDTH], self.start - 2, 0, 0);
        let (mut index, mut rule) = (0, 0);
        for (old_index, pot_num) in (0..self.len + 4).zip(start..) {
            rule >>= 1;
            rule |= ((self.pots[old_index / 64] >> (old_index % 64)) & 1) << 4;

            let value = (rules & (1 << rule)) != 0;
            if index == 0 && !value {
                // Skip over empty plant pots at the start
                start += 1;
                continue;
            }
            pots[index / 64] |= (value as u64) << (index % 64);

            index += 1;
            if value {
                sum += pot_num;

                // Don't include empty plant pots at the end in the length
                len = index;
            }
        }

        State {
            pots,
            start,
            len,
            sum,
        }
    }
}

examples!(Day12 -> (i64, i64) [
    {file: "day12_example0.txt", part1: 325},
    // Custom examples
    {
        input: "initial state: .#\n\n..#.. => #",
        part1: 1,
        part2: 1
    },
    {
        input: "initial state: #\n\n.#... => #",
        part1: PART1_GENERATIONS,
        part2: PART2_GENERATIONS
    },
    {
        input: "initial state: #\n\n....# => #",
        part1: -2 * PART1_GENERATIONS,
        part2: -2 * PART2_GENERATIONS
    },
    {
        input: "initial state: ##\n\n.##.. => #\n##... => #",
        part1: 2 * PART1_GENERATIONS + 1,
        part2: 2 * PART2_GENERATIONS + 1,
    },
    //             01234
    // 0:          ##.##
    // 1:         ##.#
    // 2:        ##...#
    // 3:       ##....##
    // 4:      ##....##
    // 5:    ##....##
    {
        input: "initial state: ##.##\n\n##.## => #\n..#.. => #\n.#... => #\n..##. => #\n...## => #",
        part1: -4 * PART1_GENERATIONS + 14,
        part2: -4 * PART2_GENERATIONS + 14,
    },
]);
