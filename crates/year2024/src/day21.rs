use utils::geometry::Vec2;
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

#[cfg(feature = "const_lut")]
static PART1_MATRIX: [[u64; 11]; 11] = num_matrix(2);
#[cfg(feature = "const_lut")]
static PART2_MATRIX: [[u64; 11]; 11] = num_matrix(25);

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
        #[cfg(feature = "const_lut")]
        return self.complexity(&PART1_MATRIX);

        #[cfg(not(feature = "const_lut"))]
        return self.complexity(&num_matrix(2));
    }

    #[must_use]
    pub fn part2(&self) -> u64 {
        #[cfg(feature = "const_lut")]
        return self.complexity(&PART2_MATRIX);

        #[cfg(not(feature = "const_lut"))]
        return self.complexity(&num_matrix(25));
    }

    fn complexity(&self, matrix: &[[u64; 11]; 11]) -> u64 {
        self.codes
            .iter()
            .map(|&code| {
                let digits = [(code % 1000) / 100, (code % 100) / 10, code % 10];
                let length = matrix[NumericKeypad::Activate as usize][digits[0] as usize]
                    + matrix[digits[0] as usize][digits[1] as usize]
                    + matrix[digits[1] as usize][digits[2] as usize]
                    + matrix[digits[2] as usize][NumericKeypad::Activate as usize];
                length * code as u64
            })
            .sum()
    }
}

const fn num_matrix(robots: u32) -> [[u64; NumericKeypad::LEN]; NumericKeypad::LEN] {
    let mut dir_matrix = [[1; 5]; 5];
    let mut i = 0;
    while i < robots {
        dir_matrix = DirectionalKeypad::cost_matrix(dir_matrix);
        i += 1;
    }
    NumericKeypad::cost_matrix(dir_matrix)
}

// Use a macro for common functions as trait functions cannot be marked as const
macro_rules! cost_matrix_functions {
    () => {
        const fn cost_matrix(
            dir_matrix: [[u64; DirectionalKeypad::LEN]; DirectionalKeypad::LEN],
        ) -> [[u64; Self::LEN]; Self::LEN] {
            let mut result = [[u64::MAX; Self::LEN]; Self::LEN];
            let mut i = 0;
            while i < Self::LEN {
                result[i][i] = 1;
                Self::visit(
                    0,
                    Self::ALL[i],
                    DirectionalKeypad::Activate,
                    Self::ALL[i],
                    dir_matrix,
                    &mut result,
                );
                i += 1;
            }
            result
        }

        const fn visit(
            cost: u64,
            current: Self,
            parent: DirectionalKeypad,
            start: Self,
            dir_matrix: [[u64; DirectionalKeypad::LEN]; DirectionalKeypad::LEN],
            result: &mut [[u64; Self::LEN]; Self::LEN],
        ) {
            let cost_with_activate =
                cost + dir_matrix[parent as usize][DirectionalKeypad::Activate as usize];
            if cost_with_activate < result[start as usize][current as usize] {
                result[start as usize][current as usize] = cost_with_activate;
            }

            let start_coords = start.coords();
            let current_coords = current.coords();
            let current_distance = current_coords.x.abs_diff(start_coords.x)
                + current_coords.y.abs_diff(start_coords.y);

            let neighbours = current.neighbours();
            let mut i = 0;
            while i < neighbours.len() {
                let (next_parent, next) = neighbours[i];
                let next_point = next.coords();
                let next_distance =
                    next_point.x.abs_diff(start_coords.x) + next_point.y.abs_diff(start_coords.y);
                if next_distance > current_distance {
                    Self::visit(
                        cost + dir_matrix[parent as usize][next_parent as usize],
                        next,
                        next_parent,
                        start,
                        dir_matrix,
                        result,
                    );
                }
                i += 1;
            }
        }
    };
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

    const fn neighbours(self) -> &'static [(DirectionalKeypad, Self)] {
        use DirectionalKeypad::*;
        match self {
            Up => &[(Right, Activate), (Down, Down)],
            Activate => &[(Left, Up), (Down, Right)],
            Left => &[(Right, Down)],
            Down => &[(Up, Up), (Left, Left), (Right, Right)],
            Right => &[(Up, Activate), (Left, Down)],
        }
    }

    const fn coords(self) -> Vec2<u32> {
        use DirectionalKeypad::*;
        match self {
            Up => Vec2::new(1, 0),
            Activate => Vec2::new(2, 0),
            Left => Vec2::new(0, 1),
            Down => Vec2::new(1, 1),
            Right => Vec2::new(2, 1),
        }
    }

    cost_matrix_functions!();
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

    const fn neighbours(self) -> &'static [(DirectionalKeypad, Self)] {
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

    const fn coords(self) -> Vec2<u32> {
        use NumericKeypad::*;
        match self {
            Key7 => Vec2::new(0, 0),
            Key8 => Vec2::new(1, 0),
            Key9 => Vec2::new(2, 0),
            Key4 => Vec2::new(0, 1),
            Key5 => Vec2::new(1, 1),
            Key6 => Vec2::new(2, 1),
            Key1 => Vec2::new(0, 2),
            Key2 => Vec2::new(1, 2),
            Key3 => Vec2::new(2, 2),
            Key0 => Vec2::new(1, 3),
            Activate => Vec2::new(2, 3),
        }
    }

    cost_matrix_functions!();
}

examples!(Day21 -> (u64, u64) [
    {input: "029A\n980A\n179A\n456A\n379A", part1: 126384},
]);
