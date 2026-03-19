use utils::number::lcm;
use utils::prelude::*;

/// Simulating gravity between pairs.
#[derive(Clone, Debug)]
pub struct Day12 {
    pos: [[i32; 4]; 3],
    part1_steps: u16,
}

impl Day12 {
    pub fn new(input: &str, input_type: InputType) -> Result<Self, InputError> {
        let moon = parser::i32()
            .with_prefix("<x=")
            .then(parser::i32().with_prefix(", y="))
            .then(parser::i32().with_prefix(", z=").with_suffix(">"));
        let moons = moon.repeat_n::<4, _>(parser::eol()).parse_complete(input)?;

        let mut pos = [[0; 4]; 3];
        for (i, (x, y, z)) in moons.into_iter().enumerate() {
            pos[0][i] = x;
            pos[1][i] = y;
            pos[2][i] = z;
        }

        Ok(Self {
            pos,
            part1_steps: match input_type {
                InputType::Example => 10,
                InputType::Real => 1000,
            },
        })
    }

    #[must_use]
    pub fn part1(&self) -> u32 {
        let mut pos = self.pos;
        let mut vel = [[0; 4]; 3];

        for _ in 0..self.part1_steps {
            Self::step_axis(&mut pos[0], &mut vel[0]);
            Self::step_axis(&mut pos[1], &mut vel[1]);
            Self::step_axis(&mut pos[2], &mut vel[2]);
        }

        let mut total = 0;
        for moon in 0..4 {
            let potential = pos[0][moon].unsigned_abs()
                + pos[1][moon].unsigned_abs()
                + pos[2][moon].unsigned_abs();
            let kinetic = vel[0][moon].unsigned_abs()
                + vel[1][moon].unsigned_abs()
                + vel[2][moon].unsigned_abs();
            total += potential * kinetic;
        }

        total
    }

    #[must_use]
    pub fn part2(&self) -> u64 {
        let x = Self::axis_period(self.pos[0]);
        let y = Self::axis_period(self.pos[1]);
        let z = Self::axis_period(self.pos[2]);
        lcm(lcm(x as i64, y as i64), z as i64) as u64
    }

    #[inline]
    fn axis_period(mut pos: [i32; 4]) -> u64 {
        let mut vel = [0; 4];
        let mut steps = 0;

        loop {
            Self::step_axis(&mut pos, &mut vel);
            steps += 1;

            if vel == [0; 4] {
                return 2 * steps;
            }
        }
    }

    #[inline]
    fn step_axis(pos: &mut [i32; 4], vel: &mut [i32; 4]) {
        let [mut p0, mut p1, mut p2, mut p3] = *pos;
        let [mut v0, mut v1, mut v2, mut v3] = *vel;

        let d01 = Self::gravity_delta(p0, p1);
        let d02 = Self::gravity_delta(p0, p2);
        let d03 = Self::gravity_delta(p0, p3);
        let d12 = Self::gravity_delta(p1, p2);
        let d13 = Self::gravity_delta(p1, p3);
        let d23 = Self::gravity_delta(p2, p3);

        v0 += d01 + d02 + d03;
        v1 += d12 + d13 - d01;
        v2 += d23 - d02 - d12;
        v3 -= d03 + d13 + d23;

        p0 += v0;
        p1 += v1;
        p2 += v2;
        p3 += v3;

        *pos = [p0, p1, p2, p3];
        *vel = [v0, v1, v2, v3];
    }

    #[inline]
    fn gravity_delta(a: i32, b: i32) -> i32 {
        i32::from(a < b) - i32::from(a > b)
    }
}

examples!(Day12 -> (u32, u64) [
    {
        input: "<x=-1, y=0, z=2>\n\
            <x=2, y=-10, z=-7>\n\
            <x=4, y=-8, z=8>\n\
            <x=3, y=5, z=-1>",
        part1: 179,
        part2: 2772,
    },
    {
        input: "<x=-8, y=-10, z=0>\n\
            <x=5, y=5, z=10>\n\
            <x=2, y=-7, z=3>\n\
            <x=9, y=-8, z=-3>",
        // part1 example using this input uses a different step count to the previous example
        part2: 4686774924,
    },
]);
