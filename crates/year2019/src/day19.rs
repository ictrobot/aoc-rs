use crate::intcode::Interpreter;
use crate::intcode::features::Day09Features;
use core::assert_matches;
use utils::geometry::Vec2;
use utils::prelude::*;

/// Interpreting machine code to scan a beam.
#[derive(Clone, Debug)]
pub struct Day19 {
    interpreter: Interpreter,
}

impl Day19 {
    pub fn new(input: &str, _: InputType) -> Result<Self, InputError> {
        Ok(Self {
            interpreter: Interpreter::parse(input, 1)?,
        })
    }

    #[must_use]
    pub fn part1(&self) -> u32 {
        let mut interpreter = Interpreter::new(Vec::new());
        let (mut min_x, mut max_x) = (49, 49);
        let mut affected = 0;
        for y in (0..50).rev() {
            let mut new_max_x = max_x;
            while new_max_x > 0 && !self.is_affected(&mut interpreter, new_max_x, y) {
                new_max_x -= 1;
            }
            if new_max_x == 0 && !self.is_affected(&mut interpreter, 0, y) {
                // Input has (0, 0) affected then a few blank lines before the beam resumes at y=6
                continue;
            }
            max_x = new_max_x;

            min_x = min_x.min(max_x);
            while min_x > 0 && self.is_affected(&mut interpreter, min_x - 1, y) {
                min_x -= 1;
            }

            affected += max_x - min_x + 1;
        }
        affected
    }

    #[must_use]
    pub fn part2(&self) -> u32 {
        let mut interpreter = Interpreter::new(Vec::new());
        let inside_x = (0..=50)
            .rev()
            .chain(51..)
            .find(|&x| self.is_affected(&mut interpreter, x, 50))
            .expect("no solution found");

        let initial = Vec2::new(inside_x.max(1), 50);
        let mut row_start = self.find_row_start(&mut interpreter, 0, 50, initial);
        let mut column_start =
            self.find_column_start(&mut interpreter, row_start.x + 99, 0, initial);

        loop {
            let next_row_start = self.find_row_start(
                &mut interpreter,
                row_start.x,
                column_start.y + 99,
                row_start,
            );
            let next_column_start = self.find_column_start(
                &mut interpreter,
                next_row_start.x + 99,
                column_start.y,
                column_start,
            );
            if next_column_start.y == column_start.y {
                return next_row_start.x * 10_000 + column_start.y;
            }
            (row_start, column_start) = (next_row_start, next_column_start);
        }
    }

    fn find_row_start(
        &self,
        interpreter: &mut Interpreter,
        mut x: u32,
        y: u32,
        last: Vec2<u32>,
    ) -> Vec2<u32> {
        let guess = x.max((last.x * y).div_ceil(last.y));
        if guess > x && self.is_affected(interpreter, guess, y) {
            x = guess;
            while x > 0 && self.is_affected(interpreter, x - 1, y) {
                x -= 1;
            }
        } else {
            while !self.is_affected(interpreter, x, y) {
                x += 1;
            }
        }
        Vec2::new(x, y)
    }

    fn find_column_start(
        &self,
        interpreter: &mut Interpreter,
        x: u32,
        mut y: u32,
        last: Vec2<u32>,
    ) -> Vec2<u32> {
        let guess = y.max((last.y * x).div_ceil(last.x));
        if guess > y && self.is_affected(interpreter, x, guess) {
            y = guess;
            while y > 0 && self.is_affected(interpreter, x, y - 1) {
                y -= 1;
            }
        } else {
            while !self.is_affected(interpreter, x, y) {
                y += 1;
            }
        }
        Vec2::new(x, y)
    }

    fn is_affected(&self, interpreter: &mut Interpreter, x: u32, y: u32) -> bool {
        interpreter.mem.clone_from(&self.interpreter.mem);
        interpreter.ip = 0;
        interpreter.relative_base = 0;
        interpreter.push_input(i64::from(x));
        interpreter.push_input(i64::from(y));

        let output = interpreter.expect_output::<Day09Features>();
        assert_matches!(output, 0 | 1, "expected output zero or one");
        output == 1
    }
}

examples!(Day19 -> (u32, u32) []);
