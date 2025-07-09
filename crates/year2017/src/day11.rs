use utils::geometry::Vec2;
use utils::prelude::*;

/// Calculating distances in a hexagonal grid.
///
/// See <https://www.redblobgames.com/grids/hexagons/>, in particular:
/// - <https://www.redblobgames.com/grids/hexagons/#neighbors-axial>
/// - <https://www.redblobgames.com/grids/hexagons/#distances-axial>
#[derive(Clone, Debug)]
pub struct Day11 {
    part1: u32,
    part2: u32,
}

impl Day11 {
    pub fn new(input: &str, _: InputType) -> Result<Self, InputError> {
        let steps = parser::literal_map!(
            "ne" => Vec2::new(1, -1),
            "nw" => Vec2::new(-1, 0),
            "se" => Vec2::new(1, 0),
            "sw" => Vec2::new(-1, 1),
            "n" => Vec2::new(0, -1),
            "s" => Vec2::new(0, 1),
        )
        .with_suffix(b','.or(parser::eof()))
        .parse_iterator(input);

        let mut pos = Vec2::new(0, 0);
        let mut max = 0;
        for step in steps {
            pos += step?;
            max = max.max(Self::hex_dist_to_origin(pos));
        }

        Ok(Self {
            part1: Self::hex_dist_to_origin(pos),
            part2: max,
        })
    }

    fn hex_dist_to_origin(p: Vec2<i32>) -> u32 {
        (p.x.unsigned_abs() + (p.x + p.y).unsigned_abs() + p.y.unsigned_abs()) / 2
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

examples!(Day11 -> (u32, u32) [
    {input: "ne,ne,ne", part1: 3},
    {input: "ne,ne,sw,sw", part1: 0},
    {input: "ne,ne,s,s", part1: 2},
    {input: "se,sw,se,sw,sw", part1: 3},
]);
