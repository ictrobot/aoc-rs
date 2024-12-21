use std::cmp::Reverse;
use std::collections::BinaryHeap;
use utils::prelude::*;

/// Counting recursive keypad presses.
#[derive(Clone, Debug)]
pub struct Day21 {
    codes: Vec<u16>,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
#[rustfmt::skip]
enum NumericKeypad {
    Key7 = 7, Key8 = 8, Key9 = 9,
    Key4 = 4, Key5 = 5, Key6 = 6,
    Key1 = 1, Key2 = 2, Key3 = 3,
              Key0 = 0, Activate = 10,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
#[rustfmt::skip]
enum DirectionalKeypad {
              Up   = 0, Activate = 4,
    Left = 1, Down = 2, Right = 3,
}

impl Day21 {
    pub fn new(input: &str, _: InputType) -> Result<Self, InputError> {
        Ok(Self {
            codes: parser::number_range(0..=999)
                .with_suffix(b'A')
                .parse_lines(input)?,
        })
    }

    #[must_use]
    pub fn part1(&self) -> u64 {
        self.complexity(2)
    }

    #[must_use]
    pub fn part2(&self) -> u64 {
        self.complexity(25)
    }

    fn complexity(&self, robots: u32) -> u64 {
        let mut dir_matrix = [[1; 5]; 5];
        for _ in 0..robots {
            dir_matrix = Self::dir_cost(dir_matrix);
        }

        let num_matrix = Self::num_cost(dir_matrix);

        self.codes
            .iter()
            .map(|&code| {
                let digits = [code / 100, (code % 100) / 10, code % 10];
                let length = num_matrix[NumericKeypad::Activate as usize][digits[0] as usize]
                    + num_matrix[digits[0] as usize][digits[1] as usize]
                    + num_matrix[digits[1] as usize][digits[2] as usize]
                    + num_matrix[digits[2] as usize][NumericKeypad::Activate as usize];
                length * code as u64
            })
            .sum()
    }

    fn dir_cost(
        dir_matrix: [[u64; DirectionalKeypad::LEN]; DirectionalKeypad::LEN],
    ) -> [[u64; DirectionalKeypad::LEN]; DirectionalKeypad::LEN] {
        let mut result = [[u64::MAX; DirectionalKeypad::LEN]; DirectionalKeypad::LEN];
        let mut queue = BinaryHeap::new();
        for start in DirectionalKeypad::ALL {
            queue.push(Reverse((0, start, DirectionalKeypad::Activate)));
            result[start as usize][start as usize] = 1;
            while let Some(Reverse((cost, current, parent))) = queue.pop() {
                for &(next_parent, next) in current.neighbours() {
                    let next_cost = cost + dir_matrix[parent as usize][next_parent as usize];
                    let activate_cost = next_cost
                        + dir_matrix[next_parent as usize][DirectionalKeypad::Activate as usize];
                    if result[start as usize][next as usize] > activate_cost {
                        result[start as usize][next as usize] = activate_cost;
                        queue.push(Reverse((next_cost, next, next_parent)));
                    }
                }
            }
        }
        result
    }

    fn num_cost(
        dir_matrix: [[u64; DirectionalKeypad::LEN]; DirectionalKeypad::LEN],
    ) -> [[u64; NumericKeypad::LEN]; NumericKeypad::LEN] {
        let mut result = [[u64::MAX; NumericKeypad::LEN]; NumericKeypad::LEN];
        let mut queue = BinaryHeap::new();
        for start in NumericKeypad::ALL {
            queue.push(Reverse((0, start, DirectionalKeypad::Activate)));
            result[start as usize][start as usize] = 1;
            while let Some(Reverse((cost, current, parent))) = queue.pop() {
                for &(next_parent, next) in current.neighbours() {
                    let next_cost = cost + dir_matrix[parent as usize][next_parent as usize];
                    let activate_cost = next_cost
                        + dir_matrix[next_parent as usize][DirectionalKeypad::Activate as usize];
                    if result[start as usize][next as usize] > activate_cost {
                        result[start as usize][next as usize] = activate_cost;
                        queue.push(Reverse((next_cost, next, next_parent)));
                    }
                }
            }
        }
        result
    }
}

impl NumericKeypad {
    const ALL: [Self; 11] = [
        NumericKeypad::Key7,
        NumericKeypad::Key8,
        NumericKeypad::Key9,
        NumericKeypad::Key4,
        NumericKeypad::Key5,
        NumericKeypad::Key6,
        NumericKeypad::Key1,
        NumericKeypad::Key2,
        NumericKeypad::Key3,
        NumericKeypad::Key0,
        NumericKeypad::Activate,
    ];
    const LEN: usize = 11;

    fn neighbours(self) -> &'static [(DirectionalKeypad, Self)] {
        use DirectionalKeypad::{Down, Left, Right, Up};
        use NumericKeypad::*;
        match self {
            Key7 => &[(Right, Key8), (Down, Key4)],
            Key8 => &[(Left, Key7), (Right, Key9), (Down, Key5)],
            Key9 => &[(Left, Key8), (Down, Key6)],
            Key4 => &[(Up, Key7), (Right, Key5), (Down, Key1)],
            Key5 => &[(Up, Key8), (Left, Key4), (Right, Key6), (Down, Key2)],
            Key6 => &[(Up, Key9), (Left, Key5), (Down, Key3)],
            Key1 => &[(Up, Key4), (Right, Key2)],
            Key2 => &[(Up, Key5), (Left, Key1), (Right, Key3), (Down, Key0)],
            Key3 => &[(Up, Key6), (Left, Key2), (Down, Activate)],
            Key0 => &[(Up, Key2), (Right, Activate)],
            Activate => &[(Up, Key3), (Left, Key0)],
        }
    }
}

impl DirectionalKeypad {
    const ALL: [Self; 5] = [
        DirectionalKeypad::Up,
        DirectionalKeypad::Activate,
        DirectionalKeypad::Left,
        DirectionalKeypad::Down,
        DirectionalKeypad::Right,
    ];
    const LEN: usize = 5;

    fn neighbours(self) -> &'static [(DirectionalKeypad, Self)] {
        use DirectionalKeypad::*;
        match self {
            Up => &[(Right, Activate), (Down, Down)],
            Activate => &[(Left, Up), (Down, Right)],
            Left => &[(Right, Down)],
            Down => &[(Up, Up), (Left, Left), (Right, Right)],
            Right => &[(Up, Activate), (Left, Down)],
        }
    }
}

examples!(Day21 -> (u64, u64) [
    {input: "029A\n980A\n179A\n456A\n379A", part1: 126384},
]);
