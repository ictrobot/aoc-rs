use std::collections::VecDeque;
use utils::point::Point2D;
use utils::prelude::*;

/// Finding when a path through a grid is blocked.
#[derive(Clone, Debug)]
pub struct Day18 {
    positions: Vec<Point2D<usize>>,
}

impl Day18 {
    pub fn new(input: &str, _: InputType) -> Result<Self, InputError> {
        Ok(Self {
            positions: parser::number_range(0u32..=70)
                .repeat_n(",")
                .map(|[x, y]| Point2D {
                    x: x as usize,
                    y: y as usize,
                })
                .parse_lines(input)?,
        })
    }

    #[must_use]
    pub fn part1(&self) -> u32 {
        self.minimum_steps(1024).expect("no solution found")
    }

    #[must_use]
    pub fn part2(&self) -> String {
        let mut left = 0;
        let mut right = self.positions.len();

        while left < right {
            let mid = (left + right) / 2;
            match self.minimum_steps(mid + 1) {
                Some(_) => left = mid + 1,
                None => right = mid,
            }
        }

        if left >= self.positions.len() {
            panic!("no solution found");
        }

        format!("{},{}", self.positions[left].x, self.positions[left].y)
    }

    fn minimum_steps(&self, fallen: usize) -> Option<u32> {
        let mut blocked = [[false; 71]; 71];
        for pos in self.positions.iter().take(fallen) {
            blocked[pos.x][pos.y] = true;
        }

        let mut queue = VecDeque::new();
        queue.push_back((Point2D::new(0, 0), 0));

        while let Some((pos, steps)) = queue.pop_front() {
            if pos == Point2D::new(70, 70) {
                return Some(steps);
            }
            if pos.x > 0 && !blocked[pos.x - 1][pos.y] {
                queue.push_back((Point2D::new(pos.x - 1, pos.y), steps + 1));
                blocked[pos.x - 1][pos.y] = true;
            }
            if pos.x < 70 && !blocked[pos.x + 1][pos.y] {
                queue.push_back((Point2D::new(pos.x + 1, pos.y), steps + 1));
                blocked[pos.x + 1][pos.y] = true;
            }
            if pos.y > 0 && !blocked[pos.x][pos.y - 1] {
                queue.push_back((Point2D::new(pos.x, pos.y - 1), steps + 1));
                blocked[pos.x][pos.y - 1] = true;
            }
            if pos.y < 70 && !blocked[pos.x][pos.y + 1] {
                queue.push_back((Point2D::new(pos.x, pos.y + 1), steps + 1));
                blocked[pos.x][pos.y + 1] = true;
            }
        }

        None
    }
}

examples!(Day18 -> (u32, &'static str) []);
