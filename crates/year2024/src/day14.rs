use utils::number::chinese_remainder;
use utils::point::Point2D;
use utils::prelude::*;

/// Finding when robots arrange themselves into a picture.
#[derive(Clone, Debug)]
pub struct Day14 {
    robots: Vec<Robot>,
}

#[derive(Copy, Clone, Debug)]
struct Robot {
    position: Point2D<i32>,
    velocity: Point2D<i32>,
}

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
        // The only time there are more than 30 robots in a single row and more than 30 robots in a
        // single column is when the robots are arranged into a Christmas Tree. Additionally, each
        // robot's x position repeats every 101 seconds, and y position repeats every 103 seconds.
        // This allows separately finding the time mod 101 where there are enough robots in a single
        // column, and then finding the time mod 103 where there are enough robots in a single row.
        // This then gives us two equations which can be solved using the Chinese reminder theorem.
        //      result % 101 == A
        //      result % 103 == B

        let a = (0..WIDTH)
            .find(|&t| {
                let mut columns = [0u8; WIDTH as usize];
                for r in &self.robots {
                    let col = (r.position.x + r.velocity.x * t).rem_euclid(WIDTH);
                    columns[col as usize] += 1;
                }
                columns.iter().any(|&b| b >= 30)
            })
            .expect("expected a time to have more than 30 robots in a column");

        let b = (0..HEIGHT)
            .find(|&t| {
                let mut rows = [0u8; HEIGHT as usize];
                for r in &self.robots {
                    let row = (r.position.y + r.velocity.y * t).rem_euclid(HEIGHT);
                    rows[row as usize] += 1;
                }
                rows.iter().any(|&b| b >= 30)
            })
            .expect("expected a time to have more than 30 robots in a row");

        chinese_remainder([a, b], [WIDTH, HEIGHT]).unwrap() as u32
    }
}

examples!(Day14 -> (u32, u32) []);
