use utils::array::ArrayVec;
use utils::grid;
use utils::prelude::*;

/// Plotting lines between nodes of the same frequency.
#[derive(Clone, Debug)]
pub struct Day08 {
    cols: usize,
    len: usize,
    antennas: [ArrayVec<usize, MAX_ANTENNA>; FREQUENCY_COUNT],
}

const MAX_ANTENNA: usize = 4;
const FREQUENCY_COUNT: usize = 62;

impl Day08 {
    pub fn new(input: &str, _: InputType) -> Result<Self, InputError> {
        let mut antennas = std::array::from_fn(|_| ArrayVec::default());
        let (rows, cols) = grid::for_each_row(
            input,
            |b| b == b'.' || b.is_ascii_alphanumeric(),
            || "expected '.' or frequency",
            |row, cols, line| {
                for (col, &b) in line.iter().enumerate() {
                    let freq = match b {
                        b'a'..=b'z' => b - b'a',
                        b'A'..=b'Z' => b - b'A' + 26,
                        b'0'..=b'9' => b - b'0' + 52,
                        b'.' => continue,
                        _ => unreachable!("input already validated"),
                    };

                    if antennas[freq as usize].push(row * cols + col).is_err() {
                        return Err(InputError::new(
                            input,
                            line,
                            format!("expected at most {MAX_ANTENNA} '{}' antennas", b as char),
                        ));
                    }
                }
                Ok(())
            },
        )?;

        Ok(Self {
            cols,
            len: rows * cols,
            antennas,
        })
    }

    #[must_use]
    pub fn part1(&self) -> usize {
        self.count_antinode_locations(false)
    }

    #[must_use]
    pub fn part2(&self) -> usize {
        self.count_antinode_locations(true)
    }

    #[inline]
    fn count_antinode_locations(&self, part2: bool) -> usize {
        let mut antinodes = vec![false; self.len];
        for indexes in &self.antennas {
            for (i, &index1) in indexes.iter().enumerate() {
                for &index2 in &indexes[i + 1..] {
                    let offset = index2 - index1;
                    // Whether adding offset should move to the right/increase the column number.
                    // Used to prevent wrapping around onto the next/previous row when
                    // adding/subtracting the offset from the previous index.
                    let right = index1 % self.cols < index2 % self.cols;

                    let mut prev = index1;
                    while let Some(index) = prev.checked_sub(offset) {
                        if (index % self.cols < prev % self.cols) == right {
                            antinodes[index] = true;
                            prev = index;
                            if part2 {
                                continue;
                            }
                        }
                        break;
                    }

                    let mut prev = index2;
                    while let Some(index) = prev.checked_add(offset) {
                        if index < self.len && ((prev % self.cols < index % self.cols) == right) {
                            antinodes[index] = true;
                            prev = index;
                            if part2 {
                                continue;
                            }
                        }
                        break;
                    }
                }

                antinodes[index1] |= part2;
            }
        }
        antinodes.iter().filter(|&&x| x).count()
    }
}

examples!(Day08 -> (usize, usize) [
    {file: "day08_example0.txt", part1: 14, part2: 34},
    {file: "day08_example1.txt", part1: 2},
    {file: "day08_example2.txt", part1: 4},
    {file: "day08_example3.txt", part1: 4},
    {file: "day08_example4.txt", part2: 9},
]);
