use utils::number::{mod_inverse, mod_pow};
use utils::prelude::*;

/// Composing card shuffles.
#[derive(Clone, Debug)]
pub struct Day22 {
    part1_shuffle: Shuffle<PART1_DECK_SIZE>,
    part2_shuffle: Shuffle<PART2_DECK_SIZE>,
}

#[derive(Copy, Clone, Debug)]
enum Technique {
    DealIntoNewStack,
    Cut(i32),
    DealWithIncrement(u32),
}

#[derive(Copy, Clone, Debug)]
struct Shuffle<const DECK_SIZE: u64> {
    multiplier: u64,
    offset: u64,
}

const PART1_DECK_SIZE: u64 = 10_007;
const PART1_CARD: u64 = 2_019;
const PART2_DECK_SIZE: u64 = 119_315_717_514_047;
const PART2_REPEATS: u64 = 101_741_582_076_661;
const PART2_POSITION: u64 = 2_020;

impl Day22 {
    pub fn new(input: &str, _: InputType) -> Result<Self, InputError> {
        if input.is_empty() {
            return Err(InputError::new(input, 0, "expected at least one technique"));
        }

        // Both deck sizes are prime. The part 2 deck is larger than u32::MAX, so a nonzero u32 only
        // needs checking against the part 1 deck size.
        let increment = parser::number_range(1..=u32::MAX).map_res(|n| {
            (!n.is_multiple_of(PART1_DECK_SIZE as u32))
                .then_some(n)
                .ok_or("expected increment to be coprime with the deck sizes")
        });
        let technique = parser::parse_tree!(
            ("deal into new stack") => Technique::DealIntoNewStack,
            ("cut ", n @ parser::i32()) => Technique::Cut(n),
            ("deal with increment ", n @ increment) => Technique::DealWithIncrement(n),
        );

        let mut part1_shuffle = Shuffle::identity();
        let mut part2_shuffle = Shuffle::identity();
        for item in technique.with_eol().parse_iterator(input) {
            let technique = item?;
            part1_shuffle.apply_technique(technique);
            part2_shuffle.apply_technique(technique);
        }

        Ok(Self {
            part1_shuffle,
            part2_shuffle,
        })
    }

    #[must_use]
    pub fn part1(&self) -> u64 {
        self.part1_shuffle.apply_to(PART1_CARD)
    }

    #[must_use]
    pub fn part2(&self) -> u64 {
        self.part2_shuffle
            .inverse()
            .repeat(PART2_REPEATS)
            .apply_to(PART2_POSITION)
    }
}

// position(x) = (multiplier * x + offset) (mod DECK_SIZE)
impl<const DECK_SIZE: u64> Shuffle<DECK_SIZE> {
    fn identity() -> Self {
        Self {
            multiplier: 1,
            offset: 0,
        }
    }

    fn apply_technique(&mut self, technique: Technique) {
        match technique {
            Technique::DealIntoNewStack => {
                self.multiplier = DECK_SIZE - self.multiplier;
                self.offset = DECK_SIZE - self.offset - 1;
            }
            Technique::Cut(amount) => {
                self.offset =
                    (self.offset as i64 - i64::from(amount)).rem_euclid(DECK_SIZE as i64) as u64;
            }
            Technique::DealWithIncrement(increment) => {
                self.multiplier = Self::mod_mul(increment as u64, self.multiplier);
                self.offset = Self::mod_mul(increment as u64, self.offset);
            }
        }
    }

    fn apply_to(self, value: u64) -> u64 {
        (Self::mod_mul(value, self.multiplier) + self.offset) % DECK_SIZE
    }

    fn inverse(mut self) -> Self {
        let inverse_multiplier = Self::mod_inverse(self.multiplier);
        let inverse_offset = Self::mod_mul(inverse_multiplier, self.offset);
        self.multiplier = inverse_multiplier;
        self.offset = (DECK_SIZE - inverse_offset) % DECK_SIZE;
        self
    }

    fn repeat(mut self, exponent: u64) -> Self {
        let repeated_multiplier = mod_pow(
            u128::from(self.multiplier),
            u128::from(exponent),
            u128::from(DECK_SIZE),
        ) as u64;
        self.offset = if self.multiplier == 1 {
            Self::mod_mul(self.offset, exponent)
        } else {
            let numerator = (repeated_multiplier + DECK_SIZE - 1) % DECK_SIZE;
            let geometric_sum = Self::mod_mul(numerator, Self::mod_inverse(self.multiplier - 1));
            Self::mod_mul(self.offset, geometric_sum)
        };
        self.multiplier = repeated_multiplier;
        self
    }

    fn mod_inverse(value: u64) -> u64 {
        mod_inverse(value as i64, DECK_SIZE as i64).expect("part 2 requires an invertible shuffle")
            as u64
    }

    fn mod_mul(a: u64, b: u64) -> u64 {
        ((u128::from(a) * u128::from(b)) % u128::from(DECK_SIZE)) as u64
    }
}

examples!(Day22 -> (u64, u64) []);
