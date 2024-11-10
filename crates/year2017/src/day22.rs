use utils::grid;
use utils::prelude::*;

/// Simulating virus spread through a grid.
#[derive(Clone, Debug)]
pub struct Day22 {
    size: usize,
    grid: Vec<State>,
}

#[derive(Copy, Clone, Debug, PartialEq)]
enum State {
    // The enum discriminants are set such that:
    // - The next direction can be found by adding the current state and direction (mod 4)
    // - The next state in part 1 is found by adding 2 (mod 4), cycling between Clean and Infected
    // - The next state in part 2 is found by adding 1 (mod 4)
    // They must also match the From implementation
    Clean = 3,
    Weakened = 0,
    Infected = 1,
    Flagged = 2,
}

#[derive(Copy, Clone, Debug)]
enum Direction {
    // Discriminants must match the From implementation
    Up = 0,
    Right,
    Down,
    Left,
}

const PADDING: usize = 250;

impl Day22 {
    pub fn new(input: &str, _: InputType) -> Result<Self, InputError> {
        let (rows, cols, grid) = grid::from_str(input, |b| match b {
            b'.' => Some(State::Clean),
            b'#' => Some(State::Infected),
            _ => None,
        })?;
        if rows != cols || rows % 2 == 0 {
            return Err(InputError::new(input, 0, "expected odd size square grid"));
        }
        Ok(Self { size: rows, grid })
    }

    #[must_use]
    pub fn part1(&self) -> u32 {
        self.simulate(10_000, |state| State::from(state as usize + 2))
    }

    #[must_use]
    pub fn part2(&self) -> u32 {
        self.simulate(10_000_000, |state| State::from(state as usize + 1))
    }

    fn simulate(&self, bursts: u32, next_state: impl Fn(State) -> State) -> u32 {
        let size = self.size + (2 * PADDING);
        let mut grid = vec![State::Clean; size * size];

        for row in 0..self.size {
            let offset = ((PADDING + row) * size) + PADDING;
            grid[offset..offset + self.size]
                .copy_from_slice(&self.grid[row * self.size..(row + 1) * self.size]);
        }

        let direction_offsets = [-(size as isize), 1, size as isize, -1];
        let mut direction = Direction::Up;
        let mut index = grid.len() / 2;

        let mut infected_transitions = 0;
        for _ in 0..bursts {
            let state = grid[index];
            let next = next_state(state);

            direction = Direction::from(state as usize + direction as usize);
            grid[index] = next;
            index = index.wrapping_add_signed(direction_offsets[direction as usize]);

            infected_transitions += u32::from(next == State::Infected);
        }

        infected_transitions
    }
}

impl From<usize> for State {
    #[inline]
    fn from(value: usize) -> Self {
        match value % 4 {
            3 => State::Clean,
            0 => State::Weakened,
            1 => State::Infected,
            2 => State::Flagged,
            _ => unreachable!(),
        }
    }
}

impl From<usize> for Direction {
    #[inline]
    fn from(value: usize) -> Self {
        match value % 4 {
            0 => Direction::Up,
            1 => Direction::Right,
            2 => Direction::Down,
            3 => Direction::Left,
            _ => unreachable!(),
        }
    }
}

examples!(Day22 -> (u32, u32) [
    {input: "..#\n#..\n...", part1: 5587, part2: 2511944},
]);
