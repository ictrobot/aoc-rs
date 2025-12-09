use utils::geometry::Vec2;
use utils::prelude::*;

/// Finding the largest rectangle inside a polygon.
#[derive(Clone, Debug)]
pub struct Day09 {
    part1: u64,
    part2: u64,
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

        for (p1, p2) in points.iter().zip(points.iter().cycle().skip(1)) {
            if p1.x != p2.x && p1.y != p2.y {
                return Err(InputError::new(
                    input,
                    0,
                    "expected points to form horizontal or vertical lines",
                ));
            }
        }

        // Compress coordinates to reduce grid size
        let mut xs: Vec<u32> = points.iter().map(|p| p.x).collect();
        xs.sort_unstable();
        xs.dedup();
        let mut ys: Vec<u32> = points.iter().map(|p| p.y).collect();
        ys.sort_unstable();
        ys.dedup();
        let points_compressed = points
            .iter()
            .map(|p| {
                Vec2::new(
                    xs.binary_search(&p.x).unwrap(),
                    ys.binary_search(&p.y).unwrap(),
                )
            })
            .collect::<Vec<_>>();

        // Compressed vertical edges, sorted by x coordinate
        let mut vertical_edges = points_compressed
            .iter()
            .zip(points_compressed.iter().cycle().skip(1))
            .filter(|(a, b)| a.x == b.x && a.y != b.y)
            .map(|(a, b)| (a.x, a.y.min(b.y), a.y.max(b.y)))
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

        let (mut part1, mut part2) = (0, 0);
        for i in 0..points.len() {
            let x1 = points[i].x;
            let x1_compressed = points_compressed[i].x;
            let y1 = points[i].y;
            let y1_compressed = points_compressed[i].y;

            for j in (i + 1)..points.len() {
                let dx = x1.abs_diff(points[j].x) as u64 + 1;
                let dy = y1.abs_diff(points[j].y) as u64 + 1;
                let area = dx * dy;
                part1 = part1.max(area);

                if area <= part2 {
                    continue;
                }

                let x2_compressed = points_compressed[j].x;
                let y2_compressed = points_compressed[j].y;

                let x_min_compressed = x1_compressed.min(x2_compressed);
                let x_max_compressed = x1_compressed.max(x2_compressed);
                let y_min_compressed = y1_compressed.min(y2_compressed);
                let y_max_compressed = y1_compressed.max(y2_compressed);

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

                part2 = area;
            }
        }

        Ok(Self { part1, part2 })
    }

    #[must_use]
    pub fn part1(&self) -> u64 {
        self.part1
    }

    #[must_use]
    pub fn part2(&self) -> u64 {
        self.part2
    }
}

examples!(Day09 -> (u64, u64) [
    {input: "7,1\n11,1\n11,7\n9,7\n9,5\n2,5\n2,3\n7,3", part1: 50, part2: 24},
]);
