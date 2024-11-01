use utils::array::ArrayVec;
use utils::prelude::*;

/// Finding connected components in a graph.
#[derive(Clone, Debug)]
pub struct Day12 {
    part1: u32,
    part2: u32,
}

impl Day12 {
    pub fn new(input: &str, _: InputType) -> Result<Self, InputError> {
        let programs = parser::u32()
            .repeat_arrayvec(", ", 1)
            .with_prefix(parser::take_while1(u8::is_ascii_digit).with_suffix(" <-> "))
            .parse_lines(input)?;

        let mut groups = Vec::new();
        let mut visited = vec![false; programs.len()];
        for i in 0..programs.len() {
            if !visited[i] {
                groups.push(Self::connect(&programs, &mut visited, i));
            }
        }

        Ok(Self {
            part1: groups[0],
            part2: groups.len() as u32,
        })
    }

    fn connect(programs: &[ArrayVec<u32, 8>], visited: &mut [bool], program: usize) -> u32 {
        visited[program] = true;

        let mut group_len = 1;
        for &connected in &programs[program] {
            if !visited[connected as usize] {
                group_len += Self::connect(programs, visited, connected as usize);
            }
        }

        group_len
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

examples!(Day12 -> (u32, u32) [
    {file: "day12_example0.txt", part1: 6, part2: 2},
]);
