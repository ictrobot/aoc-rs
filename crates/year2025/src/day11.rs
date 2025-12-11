use std::collections::HashMap;
use utils::array::ArrayVec;
use utils::prelude::*;

/// Counting paths through a directed acyclic graph.
#[derive(Clone, Debug)]
pub struct Day11 {
    connections: Vec<ArrayVec<u16, MAX_CONNECTIONS>>,
    out: u16,
    part1: Option<u16>,
    part2: Option<(u16, u16, u16)>,
}

#[derive(Clone, Copy, Debug, Default)]
enum State {
    #[default]
    Unvisited,
    CurrentlyVisiting,
    Visited(u64),
}

const MAX_CONNECTIONS: usize = 32;

impl Day11 {
    pub fn new(input: &str, _: InputType) -> Result<Self, InputError> {
        let label = parser::byte_range(b'a'..=b'z').repeat_n::<3, _>(parser::noop());

        let mut indexes = HashMap::new();
        for line in label
            .with_suffix(": ".then(parser::take_while1(|b| matches!(b, b' ' | b'a'..=b'z'))))
            .with_consumed()
            .with_eol()
            .parse_iterator(input)
        {
            let (lhs, line) = line?;
            if lhs == *b"out" {
                return Err(InputError::new(
                    input,
                    line,
                    "'out' should only appear on the rhs",
                ));
            }

            let next_num = indexes.len() as u16;
            if indexes.insert(lhs, next_num).is_some() {
                return Err(InputError::new(input, line, "duplicate device on lhs"));
            }
        }

        let out = indexes.len() as u16;
        indexes.insert(*b"out", out);

        let mapped_label = label.map_res(|b| {
            if let Some(&num) = indexes.get(&b) {
                Ok(num)
            } else {
                Err("device not found")
            }
        });

        Ok(Self {
            connections: mapped_label
                .repeat_arrayvec(b' ', 1)
                .with_prefix(label.with_suffix(": "))
                .parse_lines(input)?,
            out,
            part1: indexes.get(b"you").copied(),
            part2: if let Some(&svr) = indexes.get(b"svr")
                && let Some(&dac) = indexes.get(b"dac")
                && let Some(&fft) = indexes.get(b"fft")
            {
                Some((svr, dac, fft))
            } else {
                None
            },
        })
    }

    #[must_use]
    pub fn part1(&self) -> u64 {
        let you = self
            .part1
            .expect("input missing required devices for part 1");

        let mut visited = vec![State::Unvisited; self.connections.len() + 1];
        visited[self.out as usize] = State::Visited(1);

        self.visit(you, &mut visited)
    }

    #[must_use]
    pub fn part2(&self) -> u64 {
        let (svr, dac, fft) = self
            .part2
            .expect("input missing required devices for part 2");
        let out = self.out;

        let mut visited = vec![State::Unvisited; self.connections.len() + 1];
        let mut routes_from = |from: u16, to: u16, not_including: &[u16]| {
            visited.fill(State::Unvisited);
            visited[to as usize] = State::Visited(1);
            for &x in not_including.iter() {
                visited[x as usize] = State::Visited(0);
            }
            self.visit(from, &mut visited)
        };

        // svr -> fft -> dac -> out
        let svr_fft = routes_from(svr, fft, &[dac, out]);
        let fft_dac = routes_from(fft, dac, &[svr, out]);
        let dac_out = routes_from(dac, out, &[svr, fft]);

        // svr -> dac -> fft -> out
        let svr_dac = routes_from(svr, dac, &[fft, out]);
        let dac_fft = routes_from(dac, fft, &[svr, out]);
        let fft_out = routes_from(fft, out, &[svr, dac]);

        (svr_fft * fft_dac * dac_out) + (svr_dac * dac_fft * fft_out)
    }

    fn visit(&self, current: u16, visited: &mut [State]) -> u64 {
        match visited[current as usize] {
            State::Unvisited => {}
            State::CurrentlyVisiting => {
                panic!("cycle detected");
            }
            State::Visited(x) => return x,
        }
        visited[current as usize] = State::CurrentlyVisiting;

        let mut total = 0;
        for &next in self.connections[current as usize].iter() {
            total += self.visit(next, visited);
        }

        visited[current as usize] = State::Visited(total);
        total
    }
}

examples!(Day11 -> (u64, u64) [
    {file: "day11_example0.txt", part1: 5},
    {file: "day11_example1.txt", part2: 2},
]);
