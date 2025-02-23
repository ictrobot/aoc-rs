use std::collections::VecDeque;
use utils::graph::explore_hamiltonian_paths;
use utils::grid;
use utils::prelude::*;

/// Finding the shortest path and cycle.
///
/// Very similar to [2015 Day 9](../year2015/struct.Day09.html) and
/// [2015 Day 13](../year2015/struct.Day13.html).
#[derive(Clone, Debug)]
pub struct Day24 {
    part1: u32,
    part2: u32,
}

impl Day24 {
    pub fn new(input: &str, _: InputType) -> Result<Self, InputError> {
        let mut digit_counts = [0usize; 10];

        // The actual input has a solid border of walls, but pad the input with an extra layer of
        // wall anyway to ensure index manipulation doesn't need checks for any input
        let (_, cols, grid) = grid::from_str_padded(input, 1, b'#', |b| match b {
            b'.' | b'#' => Some(b),
            b'0'..=b'9' => {
                digit_counts[(b - b'0') as usize] += 1;
                Some(b)
            }
            _ => None,
        })?;

        let digits = digit_counts.iter().position(|&c| c == 0).unwrap_or(10);
        if let Some(d) = digit_counts.iter().position(|&c| c > 1) {
            return Err(InputError::new(input, 0, format!("duplicate {d} digit")));
        }
        if digits == 0 {
            return Err(InputError::new(input, 0, "expected 0 in grid"));
        }
        if digit_counts[digits..].iter().any(|&c| c > 0) {
            return Err(InputError::new(input, 0, format!("missing {digits} digit")));
        }

        let mut digit_positions = vec![0; digits];
        for (i, &c) in grid.iter().enumerate() {
            if c.is_ascii_digit() {
                digit_positions[(c - b'0') as usize] = i;
            }
        }

        // Find the distance from each point of interest to every other one
        let mut dist_matrix = vec![u32::MAX; digits * digits];
        'digits: for (start_digit, &start_index) in digit_positions.iter().enumerate() {
            let mut visited = vec![false; grid.len()];
            visited[start_index] = true;

            let mut queue = VecDeque::new();
            queue.push_back((start_index, 0));

            while let Some((index, dist)) = queue.pop_front() {
                if grid[index].is_ascii_digit() {
                    let end_digit = (grid[index] - b'0') as usize;
                    dist_matrix[(start_digit * digits) + end_digit] = dist;
                    dist_matrix[(end_digit * digits) + start_digit] = dist;

                    // Stop BFS early if this row of the matrix is now complete
                    if dist_matrix[start_digit * digits..(start_digit + 1) * digits]
                        .iter()
                        .all(|&c| c != u32::MAX)
                    {
                        continue 'digits;
                    }
                }

                for next in [index - 1, index + 1, index - cols, index + cols] {
                    if grid[next] != b'#' && !visited[next] {
                        queue.push_back((next, dist + 1));
                        visited[next] = true;
                    }
                }
            }

            return Err(InputError::new(input, 0, "unreachable digit"));
        }

        // Find the shortest path and the shortest cycle
        let (mut part1, mut part2) = (u32::MAX, u32::MAX);
        explore_hamiltonian_paths(
            digits as u32,
            0,
            0,
            |a, b| dist_matrix[(a as usize * digits) + b as usize],
            |a, b| a + b,
            |total, loop_edge| {
                part1 = part1.min(total);
                part2 = part2.min(total + loop_edge);
            },
        );

        Ok(Self { part1, part2 })
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

examples!(Day24 -> (u32, u32) [
    {file: "day24_example0.txt", part1: 14, part2: 20},
]);
