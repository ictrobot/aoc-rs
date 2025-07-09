use std::collections::HashMap;
use utils::geometry::Vec3;
use utils::prelude::*;

/// Simulating colliding particles.
#[derive(Clone, Debug)]
pub struct Day20 {
    particles: Vec<Particle>,
}

#[derive(Clone, Debug)]
struct Particle {
    position: Vec3<i64>,
    velocity: Vec3<i64>,
    acceleration: Vec3<i64>,
}

impl Day20 {
    pub fn new(input: &str, _: InputType) -> Result<Self, InputError> {
        let vector = parser::i64().repeat_n(b',').map(Vec3::from);

        Ok(Self {
            particles: vector
                .with_prefix("p=<")
                .then(vector.with_prefix(">, v=<"))
                .then(vector.with_prefix(">, a=<").with_suffix(">"))
                .map(|(position, velocity, acceleration)| Particle {
                    position,
                    velocity,
                    acceleration,
                })
                .parse_lines(input)?,
        })
    }

    #[must_use]
    pub fn part1(&self) -> usize {
        self.particles
            .iter()
            .enumerate()
            .min_by_key(|&(_, p)| p.position_at_time(1_000_000).manhattan_distance())
            .unwrap()
            .0
    }

    #[must_use]
    pub fn part2(&self) -> usize {
        let mut particles = self.particles.clone();
        let mut destroyed = vec![false; particles.len()];

        let mut positions = HashMap::new();
        let mut last_destroyed = 0;
        for t in 0.. {
            positions.clear();

            for (i, p) in particles.iter_mut().enumerate() {
                if destroyed[i] {
                    continue;
                }

                p.tick();

                if let Some(j) = positions.insert(p.position, i) {
                    destroyed[i] = true;
                    destroyed[j] = true;
                    last_destroyed = t;
                }
            }

            // Stop when nothing has been destroyed for 10 turns and at least one particle has been
            // destroyed.
            if last_destroyed <= t - 10 && destroyed.iter().any(|&x| x) {
                break;
            }
        }

        particles.len() - destroyed.iter().filter(|&&p| p).count()
    }
}

impl Particle {
    fn position_at_time(&self, time: u64) -> Vec3<i64> {
        self.position
            + (self.velocity * time as i64)
            + (self.acceleration * (time as i64 * time as i64 / 2))
    }

    fn tick(&mut self) {
        self.velocity += self.acceleration;
        self.position += self.velocity;
    }
}

examples!(Day20 -> (usize, usize) [
    {
        input: "p=<3,0,0>, v=<2,0,0>, a=<-1,0,0>\n\
            p=<4,0,0>, v=<0,0,0>, a=<-2,0,0>",
        part1: 0,
    },
    {
        input: "p=<-6,0,0>, v=<3,0,0>, a=<0,0,0>\n\
            p=<-4,0,0>, v=<2,0,0>, a=<0,0,0>\n\
            p=<-2,0,0>, v=<1,0,0>, a=<0,0,0>\n\
            p=<3,0,0>, v=<-1,0,0>, a=<0,0,0>",
        part2: 1,
    },
]);
