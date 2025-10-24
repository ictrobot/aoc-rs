use utils::prelude::*;
use utils::queue::BucketQueue;

/// Searching a cave with different tools.
#[derive(Clone, Debug)]
pub struct Day22 {
    depth: u32,
    target_x: usize,
    target_y: usize,
}

impl Day22 {
    pub fn new(input: &str, _: InputType) -> Result<Self, InputError> {
        let (depth, target_x, target_y) = parser::u32()
            .with_prefix("depth: ")
            .with_eol()
            .then(parser::number_range(0..=31).with_prefix("target: "))
            .then(parser::number_range(0..=1021).with_prefix(b','))
            .parse_complete(input)?;

        Ok(Self {
            depth,
            target_x,
            target_y,
        })
    }

    #[must_use]
    pub fn part1(&self) -> u32 {
        self.scan::<0>(self.target_x + 1, self.target_y + 1)
            .iter()
            .sum()
    }

    #[must_use]
    pub fn part2(&self) -> u32 {
        let width = self.target_x + 75;
        let height = self.target_y + 75;

        // Pad the edges of the grid with [0; 3] to avoid needing bounds checks (as 0 is always less
        // than next_time)
        let mut grid = vec![[u32::MAX; 3]; width * height];
        for x in 0..width {
            grid[x] = [0; 3];
            grid[(height - 1) * width + x] = [0; 3];
        }
        for y in 0..height {
            grid[y * width] = [0; 3];
            grid[(y + 1) * width - 1] = [0; 3];
        }

        // Tool indices match the region type where they're invalid (torch = 1 = wet).
        // Set invalid tools for each region to 0.
        let scan = self.scan::<1>(width, height);
        for (times, &region_type) in grid.iter_mut().zip(scan.iter()) {
            times[region_type as usize] = 0;
        }

        let start_index = width + 1;
        let target_index = (self.target_y + 1) * width + self.target_x + 1;

        let mut queue = BucketQueue::<_, 8>::with_capacity(256);
        // Pack the tool into the bottom 2 bits in the queue entry
        queue.push(0, start_index << 2 | 1);
        grid[start_index][1] = 0;

        while let Some((time, packed)) = queue.pop_entry() {
            let (time, index, tool) = (time as u32, packed >> 2, packed & 3);

            if index == target_index && tool == 1 {
                return time;
            }

            if grid[index][tool] != time {
                continue;
            }

            let region_type = scan[index] as usize;
            let other_tool = 3 - tool - region_type;

            for (next_index, next_tool, next_time) in [
                (index - 1, tool, time + 1),
                (index + 1, tool, time + 1),
                (index - width, tool, time + 1),
                (index + width, tool, time + 1),
                (index, other_tool, time + 7),
            ] {
                if grid[next_index][next_tool] > next_time {
                    grid[next_index][next_tool] = next_time;
                    queue.push(next_time as usize, next_index << 2 | next_tool);
                }
            }
        }

        panic!("no solution found")
    }

    fn scan<const PADDING: usize>(&self, width: usize, height: usize) -> Vec<u32> {
        // Padding is used by part2 to avoid bounds checks in the main time grid. Also padding
        // the scan grid keeps indexes consistent between the two grids.
        let inner_width = width - 2 * PADDING;
        let inner_height = height - 2 * PADDING;
        let target_index = (self.target_y + PADDING) * width + self.target_x + PADDING;

        let mut erosion = vec![0u32; width * height];

        let base = PADDING * width + PADDING;
        for (x, e) in erosion[base..base + inner_width].iter_mut().enumerate() {
            *e = ((x as u32 * 16807) + self.depth) % 20183;
        }

        for y in 1..inner_height {
            let base = (y + PADDING) * width + PADDING;

            let mut prev = ((y as u32 * 48271) + self.depth) % 20183;
            erosion[base] = prev;

            for index in base + 1..base + inner_width {
                if index == target_index {
                    prev = self.depth % 20183;
                } else {
                    prev = ((prev * erosion[index - width]) + self.depth) % 20183;
                }
                erosion[index] = prev;
            }
        }

        erosion.into_iter().map(|x| x % 3).collect()
    }
}

examples!(Day22 -> (u32, u32) [
    {input: "depth: 510\ntarget: 10,10", part1: 114, part2: 45},
]);
