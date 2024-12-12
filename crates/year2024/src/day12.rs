use utils::grid;
use utils::prelude::*;

/// Counting area, perimeter and sides of shapes in a grid.
#[derive(Clone, Debug)]
pub struct Day12 {
    grid: Vec<u8>,
    offsets: [isize; 4],
}

impl Day12 {
    pub fn new(input: &str, _: InputType) -> Result<Self, InputError> {
        let (_, cols, grid) =
            grid::from_str_padded(input, 2, 0, |b| b.is_ascii_uppercase().then_some(b))?;
        let offsets = [-(cols as isize), 1, cols as isize, -1];
        Ok(Self { grid, offsets })
    }

    #[must_use]
    pub fn part1(&self) -> u64 {
        let mut visited = vec![false; self.grid.len()];
        let mut total = 0;
        for i in 0..self.grid.len() {
            if self.grid[i] == 0 || visited[i] {
                continue;
            }

            let (area, perimeter) = self.flood_fill(i, &mut visited);
            total += area * perimeter;
        }
        total
    }

    fn flood_fill(&self, i: usize, visited: &mut [bool]) -> (u64, u64) {
        let plant = self.grid[i];
        visited[i] = true;

        let (mut area, mut perimeter) = (1, 0);
        for &offset in &self.offsets {
            let next = i.wrapping_add_signed(offset);
            if self.grid[next] == plant {
                if !visited[next] {
                    let (a, p) = self.flood_fill(next, visited);
                    area += a;
                    perimeter += p;
                }
            } else {
                perimeter += 1;
            }
        }

        (area, perimeter)
    }

    #[must_use]
    pub fn part2(&self) -> u64 {
        let mut visited = vec![false; self.grid.len()];
        let mut edges = vec![[false; 4]; self.grid.len()];
        let mut total = 0;
        for i in 0..self.grid.len() {
            if self.grid[i] == 0 || visited[i] {
                continue;
            }

            let (area, min_idx, max_idx) = self.edge_fill(i, &mut visited, &mut edges);

            let mut sides = 0;
            for dir in 0..4 {
                for j in min_idx..=max_idx {
                    if edges[j][dir] {
                        sides += 1;
                        self.edge_unset(j, dir, &mut edges);
                    }
                }
            }

            total += area * sides;
        }
        total
    }

    fn edge_fill(
        &self,
        i: usize,
        visited: &mut [bool],
        edges: &mut [[bool; 4]],
    ) -> (u64, usize, usize) {
        let plant = self.grid[i];
        visited[i] = true;

        let (mut area, mut min_idx, mut max_idx) = (1, usize::MAX, 0);
        for dir in 0..4 {
            let next = i.wrapping_add_signed(self.offsets[dir]);
            if self.grid[next] == plant {
                if !visited[next] {
                    let r = self.edge_fill(next, visited, edges);
                    area += r.0;
                    min_idx = min_idx.min(r.1);
                    max_idx = max_idx.max(r.2);
                }
            } else {
                edges[next][dir] = true;
                min_idx = min_idx.min(next);
                max_idx = max_idx.max(next);
            }
        }

        (area, min_idx, max_idx)
    }

    fn edge_unset(&self, i: usize, dir: usize, edges: &mut [[bool; 4]]) {
        edges[i][dir] = false;
        for offset in [self.offsets[(dir + 1) % 4], self.offsets[(dir + 3) % 4]] {
            let next = i.wrapping_add_signed(offset);
            if edges[next][dir] {
                self.edge_unset(next, dir, edges);
            }
        }
    }
}

examples!(Day12 -> (u64, u64) [
    {file: "day12_example0.txt", part1: 140, part2: 80},
    {file: "day12_example1.txt", part1: 772},
    {file: "day12_example2.txt", part1: 1930, part2: 1206},
    {file: "day12_example3.txt", part2: 236},
    {file: "day12_example4.txt", part2: 368},
]);
