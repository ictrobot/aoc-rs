use utils::array::ArrayVec;
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
        let mut lines = input.lines().peekable();
        let Some(cols) = lines.peek().map(|s| s.len()) else {
            return Err(InputError::new(input, 0, "expected grid"));
        };
        if cols == 0 {
            return Err(InputError::new(input, 0, "expected at least one column"));
        }

        let mut antennas = std::array::from_fn(|_| ArrayVec::default());
        let mut index = 0;
        for line in lines {
            if line.len() != cols {
                return Err(InputError::new(
                    input,
                    &line[cols.min(line.len())..],
                    format!("expected {cols} columns"),
                ));
            }
            for b in line.bytes() {
                let freq = if b.is_ascii_lowercase() {
                    b - b'a'
                } else if b.is_ascii_uppercase() {
                    b - b'A' + 26
                } else if b.is_ascii_digit() {
                    b - b'0' + 52
                } else if b == b'.' {
                    index += 1;
                    continue;
                } else {
                    return Err(InputError::new(input, b as char, "expected . or frequency"));
                };

                if antennas[freq as usize].push(index).is_err() {
                    return Err(InputError::new(
                        input,
                        line,
                        format!("expected at most {MAX_ANTENNA} '{}' antennas", b as char),
                    ));
                }
                index += 1;
            }
        }

        Ok(Self {
            cols,
            len: index,
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
