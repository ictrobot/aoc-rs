use utils::geometry::Vec2;
use utils::grid;
use utils::number::gcd;
use utils::prelude::*;

/// Ordering visible grid points in a rotational sweep.
#[derive(Clone, Debug)]
pub struct Day10 {
    asteroids: Vec<Vec2<i16>>,
    station: usize,
    part1: u32,
}

impl Day10 {
    pub fn new(input: &str, _: InputType) -> Result<Self, InputError> {
        let (rows, cols, grid) = grid::parse(
            input,
            0,
            false,
            |b| b == b'#',
            |b| matches!(b, b'.' | b'#'),
            |_, _| Err("expected '.' or '#'"),
        )?;
        if rows > i16::MAX as usize || cols > i16::MAX as usize {
            return Err(InputError::new(input, 0, "grid too large"));
        }

        let asteroids: Vec<Vec2<i16>> = grid
            .iter()
            .enumerate()
            .filter(|&(_, &cell)| cell)
            .map(|(index, _)| Vec2::new((index % cols) as i16, (index / cols) as i16))
            .collect();
        if asteroids.is_empty() {
            return Err(InputError::new(input, 0, "expected at least one asteroid"));
        }

        let delta_rows = 2 * rows - 1;
        let delta_cols = 2 * cols - 1;
        let delta_index = |delta: Vec2<i16>| {
            (delta.y as i32 + rows as i32 - 1) as usize * delta_cols
                + (delta.x as i32 + cols as i32 - 1) as usize
        };

        let mut reduced_cache = vec![u32::MAX; delta_rows * delta_cols];
        let mut seen = vec![0u32; reduced_cache.len()];
        let mut visible = vec![0u32; asteroids.len()];

        for i in 0..asteroids.len() {
            for j in i + 1..asteroids.len() {
                let delta = asteroids[j] - asteroids[i];
                let index = delta_index(delta);

                // reduced_cache[delta_index(delta)] caches the gcd reduced direction index for each
                // delta to avoid recalculating the same gcd multiple times.
                let reduced_index = if reduced_cache[index] == u32::MAX {
                    let reduced = delta / gcd(delta.x, delta.y).abs();
                    let reduced_index = delta_index(reduced);
                    reduced_cache[index] = reduced_index as u32;
                    reduced_index
                } else {
                    reduced_cache[index] as usize
                };

                // seen[reduced_index] is set to i + 1 each time a reduced direction is first
                // seen for asteroid i.
                // Iterating through asteroids > i in grid index order means that the first asteroid
                // j seen in each direction is the closest.
                // This allows just visiting each pair of asteroids once and incrementing the
                // visible count for both asteroids at once.
                if seen[reduced_index] <= i as u32 {
                    seen[reduced_index] = i as u32 + 1;
                    visible[i] += 1;
                    visible[j] += 1;
                }
            }
        }

        let (best_station, part1) = visible
            .into_iter()
            .enumerate()
            .max_by_key(|&(_, visible)| visible)
            .unwrap();

        Ok(Self {
            asteroids,
            station: best_station,
            part1,
        })
    }

    #[must_use]
    pub fn part1(&self) -> u32 {
        self.part1
    }

    #[must_use]
    pub fn part2(&self) -> u32 {
        struct Target {
            dir: Vec2<i16>,
            dir_multiple: i16,
            pos: Vec2<i16>,
        }

        assert!(
            self.asteroids.len() > 200,
            "expected there to be at least 201 asteroids"
        );

        let station_position = self.asteroids[self.station];
        let mut targets: Vec<Target> = self
            .asteroids
            .iter()
            .enumerate()
            .filter(|&(i, _)| i != self.station)
            .map(|(_, &position)| {
                let delta = position - station_position;
                let multiple = gcd(delta.x, delta.y).abs();
                Target {
                    dir: delta / multiple,
                    dir_multiple: multiple,
                    pos: position,
                }
            })
            .collect();

        // Sort the targets by (angle, distance)
        targets.sort_unstable_by(|a, b| {
            Self::quadrant(a.dir)
                .cmp(&Self::quadrant(b.dir))
                .then_with(|| {
                    // Within each quadrant, cross > 0 means the angle comes first clockwise
                    let cross = i32::from(a.dir.x) * i32::from(b.dir.y)
                        - i32::from(a.dir.y) * i32::from(b.dir.x);
                    0.cmp(&cross)
                })
                .then(a.dir_multiple.cmp(&b.dir_multiple))
        });

        // After sorting by (angle, distance), calculate the pass each target will be vaporized on
        // based on how many closer asteroids have the same angle.
        // Ordering by (pass, index sorted by angle) gives the final vaporization order.
        let mut target_keys = Vec::with_capacity(targets.len());
        let mut pass = 0u32;
        for (i, target) in targets.iter().enumerate() {
            if i > 0 && target.dir == targets[i - 1].dir {
                pass += 1;
            } else {
                pass = 0;
            }
            target_keys.push((pass, i));
        }

        let (_, &mut (_, index), _) = target_keys.select_nth_unstable(199);
        let position = targets[index].pos;
        (position.x as u32) * 100 + position.y as u32
    }

    fn quadrant(direction: Vec2<i16>) -> u8 {
        match (direction.x, direction.y) {
            (0.., ..0) => 0, // top-right, including up
            (1.., 0..) => 1, // bottom-right, including right
            (..1, 1..) => 2, // bottom-left, including down
            (..0, ..1) => 3, // top-left, including left
            (0, 0) => unreachable!("expected non-zero direction"),
        }
    }
}

examples!(Day10 -> (u32, u32) [
    {file: "day10_example0.txt", part1: 8},
    {file: "day10_example1.txt", part1: 33},
    {file: "day10_example2.txt", part1: 35},
    {file: "day10_example3.txt", part1: 41},
    {file: "day10_example4.txt", part1: 210, part2: 802},
]);
