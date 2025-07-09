use utils::geometry::Vec2;
use utils::prelude::*;

/// Solving linear systems.
#[derive(Clone, Debug)]
pub struct Day13 {
    machines: Vec<Machine>,
}

#[derive(Copy, Clone, Debug)]
struct Machine {
    button_a: Vec2<u64>,
    button_b: Vec2<u64>,
    prize: Vec2<u64>,
}

impl Day13 {
    pub fn new(input: &str, _: InputType) -> Result<Self, InputError> {
        Ok(Self {
            machines: parser::u64()
                .with_prefix("Button A: X+")
                .then(parser::u64().with_prefix(", Y+"))
                .then(parser::u64().with_prefix("\nButton B: X+"))
                .then(parser::u64().with_prefix(", Y+"))
                .then(parser::u64().with_prefix("\nPrize: X="))
                .then(parser::u64().with_prefix(", Y="))
                .with_suffix(parser::eol())
                .map_res(|(ax, ay, bx, by, px, py)| {
                    let m = Machine {
                        button_a: Vec2::new(ax, ay),
                        button_b: Vec2::new(bx, by),
                        prize: Vec2::new(px, py),
                    };

                    // Check the two buttons are linear independent, meaning there is only one
                    // solution for the linear equations.
                    if det(m.button_a, m.button_b) == 0 {
                        Err("expected buttons to be linearly independent")
                    } else {
                        Ok(m)
                    }
                })
                .parse_lines(input)?,
        })
    }

    #[must_use]
    pub fn part1(&self) -> u64 {
        self.machines
            .iter()
            .map(|m| m.required_tokens().unwrap_or(0))
            .sum()
    }

    #[must_use]
    pub fn part2(&self) -> u64 {
        self.machines
            .iter()
            .map(|&(mut m)| {
                m.prize += Vec2::new(10000000000000, 10000000000000);
                m.required_tokens().unwrap_or(0)
            })
            .sum()
    }
}

impl Machine {
    fn required_tokens(&self) -> Option<u64> {
        self.solve().map(|(a, b)| a * 3 + b)
    }

    fn solve(&self) -> Option<(u64, u64)> {
        // https://en.wikipedia.org/wiki/Cramer%27s_rule#Explicit_formulas_for_small_systems
        let det_denominator = det(self.button_a, self.button_b);
        if det_denominator == 0 {
            return None;
        }

        let det_a = det(self.prize, self.button_b);
        let det_b = det(self.button_a, self.prize);
        if det_a % det_denominator != 0 || det_b % det_denominator != 0 {
            return None;
        }

        if let Ok(count_a) = (det_a / det_denominator).try_into()
            && let Ok(count_b) = (det_b / det_denominator).try_into()
        {
            return Some((count_a, count_b));
        }

        None
    }
}

fn det(a: Vec2<u64>, b: Vec2<u64>) -> i64 {
    (a.x as i64) * (b.y as i64) - (b.x as i64) * (a.y as i64)
}

examples!(Day13 -> (u64, u64) [
    {file: "day13_example0.txt", part1: 480},
]);
