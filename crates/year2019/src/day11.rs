use crate::intcode::features::Day09Features;
use crate::intcode::{Event, Interpreter};
use utils::geometry::{Direction, Vec2};
use utils::prelude::*;

/// Recognizing text painted by interpreting assembly.
#[derive(Clone, Debug)]
pub struct Day11 {
    base: Interpreter,
}

#[derive(Copy, Clone, Debug, PartialEq)]
enum Panel {
    Unpainted,
    White,
    Black,
}

const WIDTH: usize = 200;
const SIZE: usize = WIDTH * WIDTH;

impl Day11 {
    pub fn new(input: &str, _: InputType) -> Result<Self, InputError> {
        Ok(Self {
            base: Interpreter::parse(input, 1)?,
        })
    }

    #[must_use]
    pub fn part1(&self) -> usize {
        let mut grid = [Panel::Unpainted; SIZE];
        self.paint(&mut grid)
    }

    #[must_use]
    pub fn part2(&self) -> String {
        let mut grid = [Panel::Unpainted; SIZE];
        grid[(WIDTH / 2) * WIDTH + (WIDTH / 2)] = Panel::White;
        self.paint(&mut grid);

        let (mut min_x, mut min_y) = (WIDTH, WIDTH);
        for (y, row) in grid.chunks_exact(WIDTH).enumerate() {
            for (x, &panel) in row.iter().enumerate() {
                if panel == Panel::White {
                    min_x = min_x.min(x);
                    min_y = min_y.min(y);
                }
            }
        }

        let mut output = String::with_capacity(8);
        for x in (min_x..min_x + 40).step_by(5) {
            let mut letter = 0;
            for y in (min_y..min_y + 6).rev() {
                for dx in 0..5 {
                    letter = (letter << 1) | u32::from(grid[y * WIDTH + x + dx] == Panel::White);
                }
            }
            output.push(crate::Day08::ocr(letter));
        }
        output
    }

    fn paint(&self, grid: &mut [Panel; SIZE]) -> usize {
        let mut interpreter = self.base.clone();
        let mut pos = Vec2::new(WIDTH as i32 / 2, WIDTH as i32 / 2);
        let mut dir = Direction::Up;
        let mut painted = 0;

        loop {
            let index = pos.y as usize * WIDTH + pos.x as usize;
            interpreter.push_input(i64::from(grid[index] == Panel::White));

            let mut next_output = || match interpreter.run::<Day09Features>() {
                Event::Halt => None,
                Event::Input => panic!("no solution found: program required more input"),
                Event::Output(x @ 0..=1) => Some(x as u8),
                Event::Output(_) => panic!("no solution found: program returned invalid output"),
            };
            let (Some(color), Some(turn)) = (next_output(), next_output()) else {
                return painted;
            };

            if grid[index] == Panel::Unpainted {
                painted += 1;
            }
            grid[index] = if color == 0 {
                Panel::Black
            } else {
                Panel::White
            };

            if turn == 0 {
                dir = dir.turn_left();
            } else {
                dir = dir.turn_right();
            }
            pos += Vec2::from(dir);
            if pos.x < 0 || pos.x >= WIDTH as i32 || pos.y < 0 || pos.y >= WIDTH as i32 {
                panic!("robot left grid bounds");
            }
        }
    }
}

examples!(Day11 -> (usize, &'static str) []);
