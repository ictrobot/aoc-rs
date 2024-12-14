use utils::point::Point2D;
use utils::prelude::*;

/// Finding when robots arrange themselves into a picture.
///
/// Assumes that the picture of the Christmas tree will involve a horizontal line of at least 10
/// robots, and that doesn't happen in any prior iterations.
#[derive(Clone, Debug)]
pub struct Day14 {
    robots: Vec<Robot>,
}

#[derive(Copy, Clone, Debug)]
struct Robot {
    position: Point2D<i32>,
    velocity: Point2D<i32>,
}

// WIDTH must be less than 128
const WIDTH: i32 = 101;
const HEIGHT: i32 = 103;

impl Day14 {
    pub fn new(input: &str, _: InputType) -> Result<Self, InputError> {
        Ok(Self {
            robots: parser::number_range(0..=WIDTH - 1)
                .with_prefix("p=")
                .then(parser::number_range(0..=HEIGHT - 1).with_prefix(","))
                .then(parser::i32().with_prefix(" v="))
                .then(parser::i32().with_prefix(","))
                .map(|(px, py, vx, vy)| Robot {
                    position: Point2D::new(px, py),
                    velocity: Point2D::new(vx, vy),
                })
                .parse_lines(input)?,
        })
    }

    #[must_use]
    pub fn part1(&self) -> u64 {
        let mut counts = [0; 4];
        for &(mut r) in self.robots.iter() {
            r.position += r.velocity * 100;
            r.position.x = r.position.x.rem_euclid(WIDTH);
            r.position.y = r.position.y.rem_euclid(HEIGHT);

            if r.position.x == WIDTH / 2 || r.position.y == HEIGHT / 2 {
                continue;
            }

            let mut quadrant = 0;
            if r.position.x > WIDTH / 2 {
                quadrant += 2;
            }
            if r.position.y > HEIGHT / 2 {
                quadrant += 1;
            }
            counts[quadrant] += 1;
        }
        counts.iter().product()
    }

    #[must_use]
    pub fn part2(&self) -> u32 {
        let mut robots = self.robots.clone();
        for i in 1..=u32::MAX {
            let mut grid = [0u128; HEIGHT as usize];
            for r in robots.iter_mut() {
                r.position += r.velocity;
                r.position.x = r.position.x.rem_euclid(WIDTH);
                r.position.y = r.position.y.rem_euclid(HEIGHT);
                grid[r.position.y as usize] |= 1 << r.position.x;
            }

            if grid.iter().any(|&b| Self::has_ten_consecutive_bits(b)) {
                return i;
            }
        }
        unreachable!("no solution found")
    }

    fn has_ten_consecutive_bits(b: u128) -> bool {
        b & (b << 1)
            & (b << 2)
            & (b << 3)
            & (b << 4)
            & (b << 5)
            & (b << 6)
            & (b << 7)
            & (b << 8)
            & (b << 9)
            != 0
    }
}

examples!(Day14 -> (u32, u32) []);
