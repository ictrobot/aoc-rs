use utils::grid;
use utils::prelude::*;

/// Counting area, perimeter and sides of shapes in a grid.
#[derive(Clone, Debug)]
pub struct Day12 {
    part1: u32,
    part2: u32,
}

impl Day12 {
    pub fn new(input: &str, _: InputType) -> Result<Self, InputError> {
        let (_, cols, grid) = grid::parse(
            input,
            1,
            0,
            |b| b,
            |b| b.is_ascii_uppercase(),
            |_, _| Err("expected uppercase letter"),
        )?;
        let offsets = [-(cols as isize), 1, cols as isize, -1];

        let mut visited = vec![false; grid.len()];
        let (mut part1, mut part2) = (0, 0);
        for i in 0..grid.len() {
            if grid[i] == 0 || visited[i] {
                continue;
            }

            let (area, perimeter, corners) = FloodFill::fill_shape(&grid, offsets, &mut visited, i);
            part1 += area * perimeter;
            part2 += area * corners;
        }

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

struct FloodFill<'a> {
    grid: &'a [u8],
    offsets: [isize; 4],
    visited: &'a mut [bool],
    area: u32,
    perimeter: u32,
    corners: u32,
}

impl<'a> FloodFill<'a> {
    fn fill_shape(
        grid: &'a [u8],
        offsets: [isize; 4],
        visited: &'a mut [bool],
        i: usize,
    ) -> (u32, u32, u32) {
        let mut instance = Self {
            grid,
            offsets,
            visited,
            area: 0,
            perimeter: 0,
            corners: 0,
        };
        instance.visit(i);
        (instance.area, instance.perimeter, instance.corners)
    }

    fn visit(&mut self, i: usize) {
        let plant = self.grid[i];
        self.visited[i] = true;
        self.area += 1;

        for d in 0..4 {
            let neighbour1 = i.wrapping_add_signed(self.offsets[d]);
            if self.grid[neighbour1] == plant {
                if !self.visited[neighbour1] {
                    self.visit(neighbour1);
                }
            } else {
                self.perimeter += 1;
            }

            let neighbour2 = i.wrapping_add_signed(self.offsets[(d + 1) % 4]);
            let between = i.wrapping_add_signed(self.offsets[d] + self.offsets[(d + 1) % 4]);
            if ((self.grid[neighbour1] == plant) == (self.grid[neighbour2] == plant))
                && (self.grid[neighbour1] != plant || self.grid[between] != plant)
            {
                self.corners += 1;
            }
        }
    }
}

examples!(Day12 -> (u32, u32) [
    {file: "day12_example0.txt", part1: 140, part2: 80},
    {file: "day12_example1.txt", part1: 772},
    {file: "day12_example2.txt", part1: 1930, part2: 1206},
    {file: "day12_example3.txt", part2: 236},
    {file: "day12_example4.txt", part2: 368},
]);
