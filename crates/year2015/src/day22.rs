use utils::prelude::*;

/// RPG spell combinations.
#[derive(Clone, Debug)]
pub struct Day22 {
    boss_health: u32,
    boss_damage: u32,
}

impl Day22 {
    pub fn new(input: &str, _: InputType) -> Result<Self, InputError> {
        let (boss_health, boss_damage) = parser::u32()
            .with_prefix("Hit Points: ")
            .with_suffix(parser::eol())
            .then(parser::u32().with_prefix("Damage: "))
            .parse_complete(input)?;

        Ok(Self {
            boss_health,
            boss_damage,
        })
    }

    #[must_use]
    pub fn part1(&self) -> u32 {
        self.min_mana(false)
    }

    #[must_use]
    pub fn part2(&self) -> u32 {
        self.min_mana(true)
    }

    fn min_mana(&self, hard_difficulty: bool) -> u32 {
        let mut min = u32::MAX;

        State {
            boss_health: self.boss_health,
            boss_damage: self.boss_damage,
            hard_difficulty,
            player_health: 50,
            player_mana: 500,
            spent_mana: 0,
            shield_timer: 0,
            poison_timer: 0,
            recharge_timer: 0,
        }
        .player_turn(&mut min);

        min
    }
}

#[derive(Clone, Copy, Debug)]
struct State {
    boss_health: u32,
    boss_damage: u32,
    hard_difficulty: bool,
    player_health: u32,
    player_mana: u32,
    spent_mana: u32,
    shield_timer: u32,
    poison_timer: u32,
    recharge_timer: u32,
}

impl State {
    fn player_turn(mut self, min_mana_to_win: &mut u32) {
        if self.hard_difficulty {
            if self.player_health <= 1 {
                // Lose
                return;
            }
            self.player_health -= 1;
        }

        self.apply_effects();
        if self.boss_health == 0 {
            // Win
            *min_mana_to_win = self.spent_mana.min(*min_mana_to_win);
            return;
        }

        // Poison
        if self.player_mana >= 173 && self.poison_timer == 0 {
            State {
                player_mana: self.player_mana - 173,
                spent_mana: self.spent_mana + 173,
                poison_timer: 6,
                ..self
            }
            .boss_turn(min_mana_to_win);
        }

        // Recharge
        if self.player_mana >= 229 && self.recharge_timer == 0 {
            State {
                player_mana: self.player_mana - 229,
                spent_mana: self.spent_mana + 229,
                recharge_timer: 5,
                ..self
            }
            .boss_turn(min_mana_to_win);
        }

        // Shield
        if self.player_mana >= 113 && self.shield_timer == 0 {
            State {
                player_mana: self.player_mana - 113,
                spent_mana: self.spent_mana + 113,
                shield_timer: 6,
                ..self
            }
            .boss_turn(min_mana_to_win);
        }

        // Magic missile
        if self.player_mana >= 53 {
            State {
                boss_health: self.boss_health.saturating_sub(4),
                player_mana: self.player_mana - 53,
                spent_mana: self.spent_mana + 53,
                ..self
            }
            .boss_turn(min_mana_to_win);
        }

        // Drain
        if self.player_mana >= 73 {
            State {
                boss_health: self.boss_health.saturating_sub(2),
                player_health: self.player_health + 2,
                player_mana: self.player_mana - 73,
                spent_mana: self.spent_mana + 73,
                ..self
            }
            .boss_turn(min_mana_to_win);
        }
    }

    #[inline]
    fn boss_turn(mut self, min_mana_to_win: &mut u32) {
        // Calculate a lower bound on the remaining mana required to defeat the boss by calculating
        // the minimum number of additional spells the player must cast, multiplied by the cost of
        // the cheapest spell. Return early unless this lower bound plus the already spent mana is
        // less than the current record
        let min_casts = self.boss_health / (3 + 3 + 4); // Poison + Poison + Magic Missiles
        if self.spent_mana + (min_casts * 53) >= *min_mana_to_win {
            return;
        }

        self.apply_effects();
        if self.boss_health == 0 {
            // Win
            *min_mana_to_win = self.spent_mana.min(*min_mana_to_win);
            return;
        }

        let armor = if self.shield_timer > 0 { 7 } else { 0 };
        let boss_damage = self.boss_damage.saturating_sub(armor).max(1);
        if self.player_health <= boss_damage || self.player_mana < 53 {
            // Lose
            return;
        }
        self.player_health -= boss_damage;

        self.player_turn(min_mana_to_win)
    }

    #[inline]
    fn apply_effects(&mut self) {
        if self.shield_timer > 0 {
            self.shield_timer -= 1;
        }

        if self.poison_timer > 0 {
            self.poison_timer -= 1;
            self.boss_health = self.boss_health.saturating_sub(3);
        }

        if self.recharge_timer > 0 {
            self.recharge_timer -= 1;
            self.player_mana += 101;
        }
    }
}

examples!(Day22 -> (u32, u32) []);
