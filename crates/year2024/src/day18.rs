use std::collections::VecDeque;
use std::ops::ControlFlow;
use utils::geometry::Vec2;
use utils::prelude::*;

/// Finding when the path through a grid is blocked.
#[derive(Clone, Debug)]
pub struct Day18 {
    blocked_at: Vec<u16>,
    size: usize,
    start_idx: usize,
    end_idx: usize,
    part1_fallen: u16,
    total_fallen: u16,
}

impl Day18 {
    pub fn new(input: &str, input_type: InputType) -> Result<Self, InputError> {
        let (max_coord, part1_fallen) = match input_type {
            InputType::Example => (6, 12),
            InputType::Real => (70, 1024),
        };
        // +1 as max_coord is inclusive, +2 for padding on each side
        let size = max_coord + 3;

        let mut blocked_at = vec![u16::MAX; size * size];
        for i in 0..size {
            blocked_at[i] = 0; // Top
            blocked_at[(size - 1) * size + i] = 0; // Bottom
            blocked_at[i * size] = 0; // Left
            blocked_at[i * size + (size - 1)] = 0; // Right
        }

        let mut fallen = 0;
        for item in parser::number_range(0..=max_coord)
            .then(parser::number_range(0..=max_coord).with_prefix(b','))
            .map(Vec2::from)
            .with_consumed()
            .with_suffix(parser::eol())
            .parse_iterator(input)
        {
            let (pos, line) = item?;
            if blocked_at[(pos.y + 1) * size + (pos.x + 1)] == u16::MAX {
                blocked_at[(pos.y + 1) * size + (pos.x + 1)] = fallen;
                fallen += 1;
            } else {
                return Err(InputError::new(input, line, "duplicate position in input"));
            }
        }

        Ok(Self {
            blocked_at,
            size,
            start_idx: size + 1,
            end_idx: (max_coord + 1) * size + (max_coord + 1),
            part1_fallen,
            total_fallen: fallen,
        })
    }

    #[must_use]
    pub fn part1(&self) -> u32 {
        let mut blocked_at = self.blocked_at.clone();
        let mut queue = VecDeque::new();
        queue.push_back((self.start_idx, 0));

        while let Some((pos, steps)) = queue.pop_front() {
            if pos == self.end_idx {
                return steps;
            }

            for next in [pos - self.size, pos + 1, pos + self.size, pos - 1] {
                if blocked_at[next] >= self.part1_fallen {
                    queue.push_back((next, steps + 1));
                    blocked_at[next] = 0;
                }
            }
        }

        panic!("no solution found")
    }

    #[must_use]
    pub fn part2(&self) -> String {
        let mut blocked_at = self.blocked_at.clone();
        let mut reachable = vec![None; self.total_fallen as usize];
        let mut fallen = self.total_fallen;
        let mut next = self.start_idx;
        loop {
            // Recursively flood fill the reachable grid spaces, tracking which grid spaces would
            // be reachable at a lower number of fallen bytes.
            if self
                .fill(next, fallen, &mut blocked_at, &mut reachable)
                .is_break()
            {
                if fallen == self.total_fallen {
                    panic!("path is never blocked");
                }
                return format!("{},{}", (next % self.size) - 1, (next / self.size) - 1);
            }

            // No path to the end. Decrease fallen to the value blocking the highest reachable grid
            // space and try again.
            fallen = reachable[..fallen as usize]
                .iter()
                .rposition(Option::is_some)
                .expect("no solution found") as u16;
            next = reachable[fallen as usize].unwrap();
        }
    }

    fn fill(
        &self,
        pos: usize,
        fallen: u16,
        blocked_at: &mut [u16],
        reachable: &mut [Option<usize>],
    ) -> ControlFlow<()> {
        if pos == self.end_idx {
            return ControlFlow::Break(());
        }

        for next in [pos - self.size, pos + 1, pos + self.size, pos - 1] {
            if blocked_at[next] >= fallen {
                blocked_at[next] = 0;
                self.fill(next, fallen, blocked_at, reachable)?;
            } else if blocked_at[next] > 0 {
                reachable[blocked_at[next] as usize] = Some(next);
            }
        }

        ControlFlow::Continue(())
    }
}

examples!(Day18 -> (u32, &'static str) [
    {file: "day18_example0.txt", part1: 22, part2: "6,1"},
]);
