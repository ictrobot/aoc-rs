use std::cmp::Ordering;
use std::collections::BinaryHeap;
use utils::geometry::Vec3;
use utils::prelude::*;

/// Finding the point in range of the most nodes.
#[derive(Clone, Debug)]
pub struct Day23 {
    bots: Vec<Bot>,
}

#[derive(Clone, Debug)]
struct Bot {
    pos: Vec3<i32>,
    r: u32,
}

// 2**29 seems to be the max range of the input, and 2**29 * 3 fits within i32 without overflow.
const MAX_POW2: u32 = 29;
const MIN: i32 = -(1 << MAX_POW2);
const MAX: i32 = (1 << MAX_POW2) - 1;

impl Day23 {
    pub fn new(input: &str, _: InputType) -> Result<Self, InputError> {
        Ok(Self {
            bots: parser::number_range(MIN..=MAX)
                .repeat_n(b',')
                .with_prefix("pos=<")
                .with_suffix(">, r=")
                .then(parser::u32())
                .map(|(pos, r)| Bot { pos: pos.into(), r })
                .parse_lines(input)?,
        })
    }

    #[must_use]
    pub fn part1(&self) -> usize {
        let strongest = self.bots.iter().max_by_key(|b| b.r).unwrap();

        self.bots
            .iter()
            .filter(|&b| b.pos.manhattan_distance_to(strongest.pos) <= strongest.r)
            .count()
    }

    #[must_use]
    pub fn part2(&self) -> u32 {
        let initial = Subcube::new(Vec3::splat(MIN), Vec3::splat(MAX), &self.bots);

        let mut heap = BinaryHeap::with_capacity(2048);
        heap.push(initial);

        while let Some(s) = heap.pop() {
            if s.size == 1 {
                return s.dist;
            }
            heap.extend(s.split(&self.bots));
        }

        unreachable!()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct Subcube {
    min: Vec3<i32>,
    max: Vec3<i32>,
    bot_count: usize,
    dist: u32,
    size: u32,
}

impl Subcube {
    #[inline]
    fn new(min: Vec3<i32>, max: Vec3<i32>, bots: &[Bot]) -> Self {
        Self {
            min,
            max,
            bot_count: bots
                .iter()
                .filter(|&bot| bot.pos.manhattan_distance_to_aabb(min, max) <= bot.r)
                .count(),
            dist: Vec3::ORIGIN.manhattan_distance_to_aabb(min, max),
            size: max.x.abs_diff(min.x) + 1,
        }
    }

    #[inline]
    fn split(&self, bots: &[Bot]) -> [Self; 8] {
        let half = (self.size / 2) as i32;
        let xs = [
            (self.min.x, self.min.x + half - 1),
            (self.min.x + half, self.max.x),
        ];
        let ys = [
            (self.min.y, self.min.y + half - 1),
            (self.min.y + half, self.max.y),
        ];
        let zs = [
            (self.min.z, self.min.z + half - 1),
            (self.min.z + half, self.max.z),
        ];

        std::array::from_fn(|i| {
            let (min_x, max_x) = xs[i >> 2];
            let (min_y, max_y) = ys[(i >> 1) & 1];
            let (min_z, max_z) = zs[i & 1];
            Self::new(
                Vec3::new(min_x, min_y, min_z),
                Vec3::new(max_x, max_y, max_z),
                bots,
            )
        })
    }
}

impl Ord for Subcube {
    #[inline]
    fn cmp(&self, other: &Self) -> Ordering {
        // Sort by bot count ascending, dist descending, size descending.
        // When used inside a MaxHeap this will order entries by the reverse (bot count descending,
        // dist ascending, size ascending), which ensures the first point visited is optimal.
        self.bot_count
            .cmp(&other.bot_count)
            .then(self.dist.cmp(&other.dist).reverse())
            .then(self.size.cmp(&other.size).reverse())
    }
}

impl PartialOrd for Subcube {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

examples!(Day23 -> (usize, u32) [
    {
        input: "pos=<0,0,0>, r=4\n\
            pos=<1,0,0>, r=1\n\
            pos=<4,0,0>, r=3\n\
            pos=<0,2,0>, r=1\n\
            pos=<0,5,0>, r=3\n\
            pos=<0,0,3>, r=1\n\
            pos=<1,1,1>, r=1\n\
            pos=<1,1,2>, r=1\n\
            pos=<1,3,1>, r=1",
        part1: 7,
    },
    {
        input: "pos=<10,12,12>, r=2\n\
            pos=<12,14,12>, r=2\n\
            pos=<16,12,12>, r=4\n\
            pos=<14,14,14>, r=6\n\
            pos=<50,50,50>, r=200\n\
            pos=<10,10,10>, r=5",
        part2: 36,
    },
]);
