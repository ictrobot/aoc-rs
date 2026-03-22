use crate::intcode::features::Day09Features;
use crate::intcode::{Event, Interpreter};
use std::collections::VecDeque;
use utils::prelude::*;

/// Interpreting machine code to find the shortest path in a maze.
#[derive(Clone, Debug)]
pub struct Day15 {
    part1: u32,
    part2: u32,
}

#[derive(Clone, Copy, Debug)]
struct Frame {
    index: usize,
    next_direction: usize,
    backtrack: i64,
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum MoveResult {
    HitWall,
    Moved,
    FoundOxygen,
}

#[derive(Clone, Copy, Debug, PartialEq)]
#[repr(u8)]
enum State {
    Unknown,
    Wall,
    Open,
}

const WIDTH: usize = 49;
const SIZE: usize = WIDTH * WIDTH;
const START_INDEX: usize = (WIDTH / 2) * WIDTH + (WIDTH / 2);
const DIRECTIONS: [(isize, i64, i64); 4] = [
    (-(WIDTH as isize), 1, 2),
    (WIDTH as isize, 2, 1),
    (-1, 3, 4),
    (1, 4, 3),
];

impl Day15 {
    pub fn new(input: &str, _: InputType) -> Result<Self, InputError> {
        let mut interpreter = Interpreter::parse(input, 1)?;
        let mut try_move = |movement_command: i64| {
            interpreter.push_input(movement_command);

            // Return error instead of panicking like Interpreter::expect_output
            match interpreter.run::<Day09Features>() {
                Event::Output(0) => Ok(MoveResult::HitWall),
                Event::Output(1) => Ok(MoveResult::Moved),
                Event::Output(2) => Ok(MoveResult::FoundOxygen),
                Event::Halt | Event::Input | Event::Output(_) => Err(InputError::new(
                    input,
                    0,
                    "expected program to output status",
                )),
            }
        };

        let mut grid = [State::Unknown; SIZE];
        grid[START_INDEX] = State::Open;
        let mut oxygen_index = None;

        let mut stack = Vec::with_capacity(SIZE);
        stack.push(Frame {
            index: START_INDEX,
            next_direction: 0,
            backtrack: 0,
        });

        while let Some(frame) = stack.last_mut() {
            if frame.next_direction == DIRECTIONS.len() {
                if frame.backtrack != 0
                    && let MoveResult::HitWall = try_move(frame.backtrack)?
                {
                    return Err(InputError::new(
                        input,
                        0,
                        "expected backtracking move to succeed",
                    ));
                }

                stack.pop();
                continue;
            }

            let (delta, command, backtrack) = DIRECTIONS[frame.next_direction];
            frame.next_direction += 1;

            let next = frame.index.wrapping_add_signed(delta);
            if !(WIDTH..SIZE - WIDTH).contains(&next)
                || next % WIDTH == 0
                || next % WIDTH == WIDTH - 1
            {
                return Err(InputError::new(input, 0, "grid too large"));
            }

            if grid[next] != State::Unknown {
                continue;
            }

            let result = try_move(command)?;
            if result == MoveResult::HitWall {
                grid[next] = State::Wall;
            } else {
                grid[next] = State::Open;
                stack.push(Frame {
                    index: next,
                    next_direction: 0,
                    backtrack,
                });

                if result == MoveResult::FoundOxygen {
                    if oxygen_index.is_some() {
                        return Err(InputError::new(input, 0, "duplicate oxygen systems"));
                    }
                    oxygen_index = Some(next);
                }
            }
        }

        let Some(oxygen_index) = oxygen_index else {
            return Err(InputError::new(input, 0, "no oxygen system found"));
        };

        let mut queue = VecDeque::new();
        queue.push_back(oxygen_index);
        grid[oxygen_index] = State::Wall;

        let mut part1 = 0;
        let mut minutes = 0;

        loop {
            for _ in 0..queue.len() {
                let index = queue.pop_front().unwrap();
                if index == START_INDEX {
                    part1 = minutes;
                }

                for offset in [-(WIDTH as isize), -1, 1, WIDTH as isize] {
                    let next = index.wrapping_add_signed(offset);
                    if grid[next] == State::Open {
                        grid[next] = State::Wall;
                        queue.push_back(next);
                    }
                }
            }

            if queue.is_empty() {
                break;
            }

            minutes += 1;
        }

        Ok(Self {
            part1,
            part2: minutes,
        })
    }

    #[must_use]
    pub fn part1(&self) -> u32 {
        self.part1
    }

    #[must_use]
    pub fn part2(&self) -> u32 {
        self.part2
    }
}

examples!(Day15 -> (u32, u32) []);
