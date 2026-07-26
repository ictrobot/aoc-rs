use utils::geometry::{Direction, Vec2};
use utils::prelude::*;

/// Moving a point with translations and rotations.
#[derive(Clone, Debug)]
pub struct Day12 {
    part1: u32,
    part2: u32,
}

#[derive(Copy, Clone, Debug)]
enum Instruction {
    Move(Direction, u16),
    Forward(u16),
    Rotate90,
    Rotate180,
    Rotate270,
}

impl Day12 {
    pub fn new(input: &str, _: InputType) -> Result<Self, InputError> {
        let instruction = parser::parse_tree!(
            (b'N', v @ parser::u16()) => Instruction::Move(Direction::Up, v),
            (b'S', v @ parser::u16()) => Instruction::Move(Direction::Down, v),
            (b'E', v @ parser::u16()) => Instruction::Move(Direction::Right, v),
            (b'W', v @ parser::u16()) => Instruction::Move(Direction::Left, v),
            (b'F', v @ parser::u16()) => Instruction::Forward(v),
            (b'L') =>> {
                // Rotations are normalised to clockwise
                ("90") => Instruction::Rotate270,
                ("180") => Instruction::Rotate180,
                ("270") => Instruction::Rotate90,
            },
            (b'R') =>> {
                ("90") => Instruction::Rotate90,
                ("180") => Instruction::Rotate180,
                ("270") => Instruction::Rotate270,
            },
        )
        .with_eol();

        let (mut ship1, mut direction) = (Vec2::ORIGIN, Vec2::RIGHT);
        let (mut ship2, mut waypoint) = (Vec2::ORIGIN, Vec2::new(10, 1));
        for item in instruction.parse_iterator(input) {
            match item? {
                Instruction::Move(dir, v) => {
                    let movement = Vec2::from(dir) * i32::from(v);
                    ship1 += movement;
                    waypoint += movement;
                }
                Instruction::Forward(v) => {
                    ship1 += direction * i32::from(v);
                    ship2 += waypoint * i32::from(v);
                }
                Instruction::Rotate90 => {
                    (direction, waypoint) = (direction.turn_right(), waypoint.turn_right());
                }
                Instruction::Rotate180 => (direction, waypoint) = (-direction, -waypoint),
                Instruction::Rotate270 => {
                    (direction, waypoint) = (direction.turn_left(), waypoint.turn_left());
                }
            }
        }

        Ok(Self {
            part1: ship1.manhattan_distance(),
            part2: ship2.manhattan_distance(),
        })
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
    {input: "F10\nN3\nF7\nR90\nF11", part1: 25, part2: 286},
]);
