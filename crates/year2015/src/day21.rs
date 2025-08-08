use utils::prelude::*;

/// RPG item shop combinations.
#[derive(Clone, Debug)]
pub struct Day21 {
    part1: u32,
    part2: u32,
}

const WEAPONS: [(u32, u32); 5] = [(8, 4), (10, 5), (25, 6), (40, 7), (74, 8)];
const ARMOR: [(u32, u32); 6] = [(0, 0), (13, 1), (31, 2), (53, 3), (75, 4), (102, 5)];
const RINGS: [(u32, u32, u32); 8] = [
    (0, 0, 0),
    (0, 0, 0),
    (25, 1, 0),
    (50, 2, 0),
    (100, 3, 0),
    (20, 0, 1),
    (40, 0, 2),
    (80, 0, 3),
];

impl Day21 {
    pub fn new(input: &str, _: InputType) -> Result<Self, InputError> {
        let (boss_health, boss_damage, boss_armor) = parser::u32()
            .with_prefix("Hit Points: ")
            .then(parser::u32().with_prefix(parser::eol().then("Damage: ")))
            .then(parser::u32().with_prefix(parser::eol().then("Armor: ")))
            .parse_complete(input)?;

        let mut min_gold_win = u32::MAX;
        let mut max_gold_loss = 0;
        for weapon in WEAPONS {
            for armor in ARMOR {
                for (i, &ring1) in RINGS.iter().enumerate() {
                    for &ring2 in &RINGS[i + 1..] {
                        let gold = weapon.0 + armor.0 + ring1.0 + ring2.0;
                        let player_damage = weapon.1 + ring1.1 + ring2.1;
                        let player_armor = armor.1 + ring1.2 + ring2.2;

                        let player_deals = player_damage.saturating_sub(boss_armor).max(1);
                        let turn_boss_dies = boss_health.div_ceil(player_deals);

                        let boss_deals = boss_damage.saturating_sub(player_armor).max(1);
                        let turn_player_dies = 100u32.div_ceil(boss_deals);

                        if turn_boss_dies <= turn_player_dies {
                            min_gold_win = min_gold_win.min(gold);
                        } else {
                            max_gold_loss = max_gold_loss.max(gold);
                        }
                    }
                }
            }
        }

        Ok(Self {
            part1: min_gold_win,
            part2: max_gold_loss,
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

examples!(Day21 -> (u32, u32) []);
