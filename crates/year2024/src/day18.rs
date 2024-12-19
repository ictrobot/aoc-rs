use std::collections::VecDeque;
use std::ops::ControlFlow;
use utils::point::Point2D;
use utils::prelude::*;

/// Finding when the path through a grid is blocked.
#[derive(Clone, Debug)]
pub struct Day18 {
    blocked_at: [[u16; WIDTH]; WIDTH],
    fallen: u16,
}

const MAX_COORD: usize = 70;
const WIDTH: usize = MAX_COORD + 1;

impl Day18 {
    pub fn new(input: &str, _: InputType) -> Result<Self, InputError> {
        let mut blocked_at = [[u16::MAX; WIDTH]; WIDTH];
        let mut fallen = 0;
        for item in parser::number_range(0..=MAX_COORD)
            .then(parser::number_range(0..=MAX_COORD).with_prefix(b','))
            .map(|(x, y)| Point2D { x, y })
            .with_suffix(parser::eol())
            .parse_iterator(input)
        {
            let pos = item?;
            if blocked_at[pos.x][pos.y] == u16::MAX {
                blocked_at[pos.x][pos.y] = fallen;
                fallen += 1;
            } else {
                return Err(InputError::new(input, 0, "duplicate position in input"));
            }
        }

        Ok(Self { blocked_at, fallen })
    }

    #[must_use]
    pub fn part1(&self) -> u32 {
        self.minimum_steps(1024).expect("no solution found")
    }

    fn minimum_steps(&self, fallen: u16) -> Option<u32> {
        let mut blocked_at = self.blocked_at;
        let mut queue = VecDeque::new();
        queue.push_back((Point2D::new(0, 0), 0));

        while let Some((pos, steps)) = queue.pop_front() {
            if pos == Point2D::new(MAX_COORD, MAX_COORD) {
                return Some(steps);
            }

            if pos.x > 0 && blocked_at[pos.x - 1][pos.y] >= fallen {
                queue.push_back((Point2D::new(pos.x - 1, pos.y), steps + 1));
                blocked_at[pos.x - 1][pos.y] = 0;
            }
            if pos.x < MAX_COORD && blocked_at[pos.x + 1][pos.y] >= fallen {
                queue.push_back((Point2D::new(pos.x + 1, pos.y), steps + 1));
                blocked_at[pos.x + 1][pos.y] = 0;
            }
            if pos.y > 0 && blocked_at[pos.x][pos.y - 1] >= fallen {
                queue.push_back((Point2D::new(pos.x, pos.y - 1), steps + 1));
                blocked_at[pos.x][pos.y - 1] = 0;
            }
            if pos.y < MAX_COORD && blocked_at[pos.x][pos.y + 1] >= fallen {
                queue.push_back((Point2D::new(pos.x, pos.y + 1), steps + 1));
                blocked_at[pos.x][pos.y + 1] = 0;
            }
        }

        None
    }

    #[must_use]
    pub fn part2(&self) -> String {
        let mut blocked_at = self.blocked_at;
        let mut reachable = vec![None; self.fallen as usize];
        let mut fallen = self.fallen;
        let mut next = Point2D::new(0, 0);
        loop {
            // Recursively flood fill the reachable grid spaces, tracking which grid spaces would
            // be reachable at a lower number of fallen bytes.
            if Self::fill(next, fallen, &mut blocked_at, &mut reachable).is_break() {
                if next == Point2D::new(0, 0) {
                    panic!("path is never blocked");
                }
                return format!("{},{}", next.x, next.y);
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
        pos: Point2D<usize>,
        fallen: u16,
        blocked_at: &mut [[u16; 71]; 71],
        reachable: &mut [Option<Point2D<usize>>],
    ) -> ControlFlow<()> {
        if pos == Point2D::new(MAX_COORD, MAX_COORD) {
            return ControlFlow::Break(());
        }

        for dir in [Point2D::UP, Point2D::RIGHT, Point2D::DOWN, Point2D::LEFT] {
            let next = pos.wrapping_add_signed(dir);
            if next.x > MAX_COORD || next.y > MAX_COORD {
                continue;
            }

            if blocked_at[next.x][next.y] >= fallen {
                blocked_at[next.x][next.y] = 0;
                Self::fill(next, fallen, blocked_at, reachable)?;
            } else if blocked_at[next.x][next.y] > 0 {
                reachable[blocked_at[next.x][next.y] as usize] = Some(next);
            }
        }

        ControlFlow::Continue(())
    }
}

examples!(Day18 -> (u32, &'static str) []);
