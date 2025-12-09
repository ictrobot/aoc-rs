use utils::geometry::Vec2;
use utils::prelude::*;

/// Finding the largest rectangle inside a polygon.
#[derive(Clone, Debug)]
pub struct Day09 {
    points: Vec<Vec2<u32>>,
}

impl Day09 {
    pub fn new(input: &str, _: InputType) -> Result<Self, InputError> {
        let points = parser::u32()
            .repeat_n(b',')
            .map(Vec2::from)
            .parse_lines(input)?;

        if points.len() < 4 {
            return Err(InputError::new(input, 0, "expected at least 4 points"));
        }

        for (p1, p2) in points
            .iter()
            .zip(points.iter().skip(1).chain(points.first()))
        {
            if p1.x != p2.x && p1.y != p2.y {
                return Err(InputError::new(
                    input,
                    0,
                    "expected points to form horizontal or vertical lines",
                ));
            }
        }

        Ok(Self { points })
    }

    #[must_use]
    pub fn part1(&self) -> u64 {
        let mut max = 0;
        for i in 0..self.points.len() {
            for j in i + 1..self.points.len() {
                let dx = self.points[i].x.abs_diff(self.points[j].x) + 1;
                let dy = self.points[i].y.abs_diff(self.points[j].y) + 1;
                max = max.max(dx as u64 * dy as u64)
            }
        }
        max
    }

    #[must_use]
    pub fn part2(&self) -> u64 {
        // Compress coordinates to reduce grid size
        let mut xs: Vec<u32> = self.points.iter().map(|p| p.x).collect();
        xs.sort_unstable();
        xs.dedup();
        let mut ys: Vec<u32> = self.points.iter().map(|p| p.y).collect();
        ys.sort_unstable();
        ys.dedup();

        // Compressed vertical edges, sorted by x coordinate
        let mut vertical_edges = self
            .points
            .iter()
            .zip(self.points.iter().skip(1).chain(self.points.first()))
            .filter(|(a, b)| a.x == b.x && a.y != b.y)
            .map(|(a, b)| {
                let x_compressed = xs.binary_search(&a.x).unwrap();
                let y1_compressed = ys.binary_search(&a.y.min(b.y)).unwrap();
                let y2_compressed = ys.binary_search(&a.y.max(b.y)).unwrap();
                (x_compressed, y1_compressed, y2_compressed)
            })
            .collect::<Vec<_>>();
        vertical_edges.sort_unstable_by_key(|&(x, _, _)| x);

        // Compressed point inside polygon grid, each cell (x, y) representing whether the region
        // (xs[x], ys[y]) to (xs[x+1], ys[y+1]) is inside the polygon.
        let inside_width = xs.len() - 1;
        let inside_height = ys.len() - 1;
        let mut inside_grid = vec![false; inside_width * inside_height];
        for y in 0..inside_height {
            let mut inside = false;
            let mut last_x = 0;
            for &(x, y1, y2) in vertical_edges.iter() {
                if y < y1 || y >= y2 {
                    continue;
                }

                if inside {
                    inside_grid[y * inside_width + last_x..y * inside_width + x].fill(inside);
                }

                inside = !inside;
                last_x = x;
            }
        }

        // Prefix sum grid to quickly count how many cells within a rectangle are inside the polygon
        let prefix_width = inside_width + 1;
        let prefix_height = inside_height + 1;
        let mut prefix_grid = vec![0u32; prefix_width * prefix_height];
        for y in 0..inside_height {
            let mut row_sum = 0u32;
            for x in 0..inside_width {
                row_sum += u32::from(inside_grid[y * inside_width + x]);
                prefix_grid[(y + 1) * prefix_width + (x + 1)] =
                    prefix_grid[y * prefix_width + (x + 1)] + row_sum;
            }
        }

        let mut max = 0;
        for i in 0..self.points.len() {
            for j in (i + 1)..self.points.len() {
                let x_min = self.points[i].x.min(self.points[j].x);
                let x_max = self.points[i].x.max(self.points[j].x);
                let y_min = self.points[i].y.min(self.points[j].y);
                let y_max = self.points[i].y.max(self.points[j].y);

                let dx = (x_max - x_min) as u64 + 1;
                let dy = (y_max - y_min) as u64 + 1;
                let area = dx * dy;
                if area <= max {
                    continue;
                }

                let x_min_compressed = xs.binary_search(&x_min).unwrap();
                let x_max_compressed = xs.binary_search(&x_max).unwrap();
                let y_min_compressed = ys.binary_search(&y_min).unwrap();
                let y_max_compressed = ys.binary_search(&y_max).unwrap();

                let inside_cells = prefix_grid[y_max_compressed * prefix_width + x_max_compressed]
                    + prefix_grid[y_min_compressed * prefix_width + x_min_compressed]
                    - prefix_grid[y_min_compressed * prefix_width + x_max_compressed]
                    - prefix_grid[y_max_compressed * prefix_width + x_min_compressed];

                let width_compressed = x_max_compressed - x_min_compressed;
                let height_compressed = y_max_compressed - y_min_compressed;
                let total_cells = (width_compressed * height_compressed) as u32;

                if inside_cells != total_cells {
                    continue;
                }

                max = area;
            }
        }
        max
    }
}

examples!(Day09 -> (u64, u64) [
    {input: "7,1\n11,1\n11,7\n9,7\n9,5\n2,5\n2,3\n7,3", part1: 50, part2: 24},
]);
