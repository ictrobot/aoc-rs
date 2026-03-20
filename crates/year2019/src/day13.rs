use crate::intcode::features::Day09Features;
use crate::intcode::{Event, Interpreter};
use utils::prelude::*;

/// Interpreting machine code to play a brick breaker arcade game.
#[derive(Clone, Debug)]
pub struct Day13 {
    interpreter: Interpreter,
}

const EMPTY: i64 = 0;
const WALL: i64 = 1;
const BLOCK: i64 = 2;
const PADDLE: i64 = 3;
const BALL: i64 = 4;

impl Day13 {
    pub fn new(input: &str, _: InputType) -> Result<Self, InputError> {
        Ok(Self {
            interpreter: Interpreter::parse(input, 1)?,
        })
    }

    #[must_use]
    pub fn part1(&self) -> u32 {
        let mut interpreter = self.interpreter.clone();
        let mut blocks = 0;

        while interpreter.next_output::<Day09Features>().is_some() {
            let _ = interpreter.expect_output::<Day09Features>();
            match interpreter.expect_output::<Day09Features>() {
                EMPTY | WALL | PADDLE | BALL => {}
                BLOCK => blocks += 1,
                _ => panic!("no solution found: program returned invalid tile id"),
            }
        }

        blocks
    }

    #[must_use]
    pub fn part2(&self) -> i64 {
        let mut interpreter = self.interpreter.clone();
        interpreter.mem[0] = 2;

        let mut score = 0;
        let (mut ball_x, mut paddle_x) = (0i64, 0i64);

        loop {
            let x = match interpreter.run::<Day09Features>() {
                Event::Halt => break score,
                Event::Input => {
                    interpreter.push_input((ball_x - paddle_x).signum());
                    continue;
                }
                Event::Output(x) => x,
            };
            let y = interpreter.expect_output::<Day09Features>();
            let tile = interpreter.expect_output::<Day09Features>();

            if x == -1 && y == 0 {
                score = tile;
                continue;
            }

            match tile {
                EMPTY | WALL | BLOCK => {}
                PADDLE => paddle_x = x,
                BALL => ball_x = x,
                _ => panic!("no solution found: program returned invalid tile id"),
            }
        }
    }
}

examples!(Day13 -> (u32, i64) []);
