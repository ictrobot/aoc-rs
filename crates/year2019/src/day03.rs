use utils::geometry::{Direction, Vec2};
use utils::prelude::*;

/// Finding path intersections.
///
/// This solution assumes that intersections only occur between perpendicular segments.
#[derive(Clone, Debug)]
pub struct Day03 {
    part1: u32,
    part2: u32,
}

#[derive(Clone, Debug)]
struct Segment {
    min: Vec2<i32>,
    max: Vec2<i32>,
    start: Vec2<i32>,
    start_steps: u32,
}

impl Day03 {
    pub fn new(input: &str, _: InputType) -> Result<Self, InputError> {
        let [(mut wire1h, mut wire1v), (wire2h, wire2v)] = parser::byte_map!(
            b'U' => Direction::Up,
            b'R' => Direction::Right,
            b'D' => Direction::Down,
            b'L' => Direction::Left
        )
        .then(parser::u16())
        .repeat_fold(
            b',',
            1,
            (Vec::new(), Vec::new(), Vec2::ORIGIN, 0u32),
            |(mut horizontal, mut vertical, start, steps), (dir, dist)| {
                let end = start + Vec2::from(dir) * dist as i32;
                let segment = Segment {
                    min: start.component_min(end),
                    max: start.component_max(end),
                    start,
                    start_steps: steps,
                };
                if matches!(dir, Direction::Left | Direction::Right) {
                    horizontal.push(segment);
                } else {
                    vertical.push(segment);
                }
                (horizontal, vertical, end, steps + dist as u32)
            },
        )
        .map(|(horizontal, vertical, _, _)| (horizontal, vertical))
        .repeat_n(parser::eol())
        .parse_complete(input)?;

        wire1h.sort_unstable_by_key(|s| s.min.y);
        wire1v.sort_unstable_by_key(|s| s.min.x);

        let (mut part1, mut part2) = (u32::MAX, u32::MAX);
        let mut check = |horizontal: &Segment, vertical: &Segment| {
            let intersection = Vec2::new(vertical.min.x, horizontal.min.y);
            if intersection.x >= horizontal.min.x
                && intersection.x <= horizontal.max.x
                && intersection.y >= vertical.min.y
                && intersection.y <= vertical.max.y
                && intersection != Vec2::ORIGIN
            {
                part1 = part1.min(intersection.manhattan_distance());

                let h_steps = horizontal.start_steps + horizontal.start.x.abs_diff(intersection.x);
                let v_steps = vertical.start_steps + vertical.start.y.abs_diff(intersection.y);
                part2 = part2.min(h_steps + v_steps);
            }
        };

        for horizontal in wire2h.iter() {
            for vertical in wire1v
                .iter()
                .skip(wire1v.partition_point(|s| s.min.x < horizontal.min.x))
                .take_while(|s| s.min.x <= horizontal.max.x)
            {
                check(horizontal, vertical);
            }
        }
        for vertical in wire2v.iter() {
            for horizontal in wire1h
                .iter()
                .skip(wire1h.partition_point(|s| s.min.y < vertical.min.y))
                .take_while(|s| s.min.y <= vertical.max.y)
            {
                check(horizontal, vertical);
            }
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

examples!(Day03 -> (u32, u32) [
    {
        input: "R8,U5,L5,D3\nU7,R6,D4,L4",
        part1: 6,
        part2: 30,
    },
    {
        input: "R75,D30,R83,U83,L12,D49,R71,U7,L72\nU62,R66,U55,R34,D71,R55,D58,R83",
        part1: 159,
        part2: 610,
    },
    {
        input: "R98,U47,R26,D63,R33,U87,L62,D20,R33,U53,R51\nU98,R91,D20,R16,D67,R40,U7,R15,U6,R7",
        part1: 135,
        part2: 410,
    },
]);
