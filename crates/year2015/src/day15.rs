use std::array;
use utils::prelude::*;

/// Maximizing ingredient score.
#[derive(Clone, Debug)]
pub struct Day15 {
    part1: i32,
    part2: i32,
}

impl Day15 {
    pub fn new(input: &str, _: InputType) -> Result<Self, InputError> {
        let ingredients = parser::i32()
            .with_prefix(": capacity ")
            .with_prefix(parser::take_while1(u8::is_ascii_alphabetic))
            .then(parser::i32().with_prefix(", durability "))
            .then(parser::i32().with_prefix(", flavor "))
            .then(parser::i32().with_prefix(", texture "))
            .then(parser::i32().with_prefix(", calories "))
            .map(|(a, b, c, d, e)| [a, b, c, d, e])
            .parse_lines(input)?;

        let (part1, part2) = Self::ingredients(100, [0; 5], &ingredients);
        Ok(Self { part1, part2 })
    }

    fn ingredients(teaspoons: i32, totals: [i32; 5], ingredients: &[[i32; 5]]) -> (i32, i32) {
        if let Ok(two_ingredients) = ingredients.try_into() {
            return Self::two_ingredients(teaspoons, totals, two_ingredients);
        }

        let (ingredient, remaining) = ingredients.split_first().unwrap();
        (0..=teaspoons)
            .map(|t| {
                Self::ingredients(
                    teaspoons - t,
                    array::from_fn(|i| totals[i] + t * ingredient[i]),
                    remaining,
                )
            })
            .fold((0, 0), |(a1, b1), (a2, b2)| (a1.max(a2), b1.max(b2)))
    }

    fn two_ingredients(teaspoons: i32, totals: [i32; 5], ingredients: [[i32; 5]; 2]) -> (i32, i32) {
        // Return early if the total for any property is already equal to or less than zero and
        // neither of the two ingredients can increase it
        if (0..5).any(|i| totals[i] <= 0 && ingredients[0][i] <= 0 && ingredients[1][i] <= 0) {
            return (0, 0);
        }

        (0..=teaspoons)
            .map(|t| {
                let totals: [i32; 5] = array::from_fn(|i| {
                    totals[i] + (t * ingredients[0][i]) + ((teaspoons - t) * ingredients[1][i])
                });

                if totals[0] <= 0 || totals[1] <= 0 || totals[2] <= 0 || totals[3] <= 0 {
                    (0, 0)
                } else {
                    let score = totals[0] * totals[1] * totals[2] * totals[3];
                    (score, if totals[4] == 500 { score } else { 0 })
                }
            })
            .fold((0, 0), |(a1, b1), (a2, b2)| (a1.max(a2), b1.max(b2)))
    }

    #[must_use]
    pub fn part1(&self) -> i32 {
        self.part1
    }

    #[must_use]
    pub fn part2(&self) -> i32 {
        self.part2
    }
}

examples!(Day15 -> (i32, i32) [
    {
        input: "Butterscotch: capacity -1, durability -2, flavor 6, texture 3, calories 8\n\
            Cinnamon: capacity 2, durability 3, flavor -2, texture -1, calories 3",
        part1: 62842880,
        part2: 57600000,
    },
]);
