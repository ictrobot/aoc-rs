use std::cell::Cell;
use std::cmp::Ordering;
use utils::prelude::*;

/// Simulating fight outcomes between a reindeer's immune system and an infection.
#[derive(Clone, Debug)]
pub struct Day24 {
    immune: Vec<Group>,
    infection: Vec<Group>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct Group {
    units: u32,
    hit_points: u32,
    modifiers: [Modifier; AttackType::COUNT],
    damage: u32,
    damage_type: AttackType,
    initiative: u32,
}

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
enum Modifier {
    // Values match damage modifiers
    Immune = 0,
    #[default]
    Normal = 1,
    Weak = 2,
}

parser::parsable_enum!(
    #[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
    enum AttackType {
        #[default]
        "bludgeoning" => Bludgeoning,
        "cold" => Cold,
        "fire" => Fire,
        "radiation" => Radiation,
        "slashing" => Slashing,
    }
);

#[derive(Copy, Clone, Debug)]
enum Army {
    Immune,
    Infection,
}

impl Day24 {
    pub fn new(input: &str, _: InputType) -> Result<Self, InputError> {
        let attack_types = AttackType::PARSER.repeat_arrayvec::<{ AttackType::COUNT }, _>(", ", 1);
        let modifiers = parser::one_of((
            attack_types
                .with_prefix("(immune to ")
                .then(attack_types.with_prefix("; weak to ").optional())
                .with_suffix(") ")
                .map(|(immune, weak)| (immune, weak.unwrap_or_default())),
            attack_types
                .with_prefix("(weak to ")
                .then(attack_types.with_prefix("; immune to ").optional())
                .with_suffix(") ")
                .map(|(weak, immune)| (immune.unwrap_or_default(), weak)),
            parser::noop().map(|_| Default::default()),
        ))
        .map_res(|(immune, weak)| {
            let mut modifiers = [Modifier::Normal; AttackType::COUNT];
            for (attack_types, modifier) in [(immune, Modifier::Immune), (weak, Modifier::Weak)] {
                for &attack_type in &attack_types {
                    if modifiers[attack_type] != Modifier::Normal {
                        return Err("duplicate attack type in modifiers");
                    }
                    modifiers[attack_type] = modifier;
                }
            }
            Ok(modifiers)
        });

        let initiative_mask = Cell::new(0u32);
        let group = parser::number_range(1..=99999)
            .with_suffix(" units each with ")
            .then(parser::number_range(1..=99999).with_suffix(" hit points "))
            .then(modifiers.with_suffix("with an attack that does "))
            .then(parser::number_range(1..=99999).with_suffix(b' '))
            .then(AttackType::PARSER.with_suffix(" damage at initiative "))
            .then(parser::number_range(1..=31))
            .map_res(
                |(units, hit_points, modifiers, damage, damage_type, initiative)| {
                    let initiative = initiative - 1;
                    if initiative_mask.get() & (1 << initiative) != 0 {
                        return Err("duplicate initiative");
                    }
                    initiative_mask.set(initiative_mask.get() | (1 << initiative));
                    Ok(Group {
                        units,
                        hit_points,
                        modifiers,
                        damage,
                        damage_type,
                        initiative,
                    })
                },
            )
            .repeat(parser::eol(), 1);

        let (immune, infection) = group
            .with_prefix("Immune System:".with_eol())
            .with_eol()
            .with_eol()
            .then(group.with_prefix("Infection:".with_eol()))
            .parse_complete(input)?;

        if initiative_mask.get().trailing_ones() as usize != immune.len() + infection.len() {
            return Err(InputError::new(
                input,
                0,
                "expected initiative values to be sequential",
            ));
        }

        Ok(Self { immune, infection })
    }

    #[must_use]
    pub fn part1(&self) -> u32 {
        if let Some((units, _)) = self.fight(0) {
            return units;
        }
        panic!("no solution found");
    }

    #[must_use]
    pub fn part2(&self) -> u32 {
        for boost in 1.. {
            if let Some((units, Army::Immune)) = self.fight(boost) {
                return units;
            }
        }
        panic!("no solution found");
    }

    fn fight(&self, boost: u32) -> Option<(u32, Army)> {
        let mut immune = self.immune.clone();
        let mut infection = self.infection.clone();
        let groups = immune.len() + infection.len();

        immune.iter_mut().for_each(|g| g.damage += boost);

        let mut attacks = [None; 32];
        loop {
            immune.sort_unstable();
            infection.sort_unstable();

            let mut taken = 0u32;
            for (attacking, defending, army) in [
                (&immune, &infection, Army::Immune),
                (&infection, &immune, Army::Infection),
            ] {
                for (attack_idx, attack_group) in attacking.iter().enumerate() {
                    if let Some((defend_idx, defend_group)) = defending
                        .iter()
                        .enumerate()
                        .filter(|(_, defend_group)| {
                            taken & (1u32 << defend_group.initiative) == 0
                                && attack_group.damage(defend_group) > 0
                        })
                        .max_by_key(|(_, defend_group)| {
                            (
                                attack_group.damage(defend_group),
                                defend_group.effective_power(),
                                defend_group.initiative,
                            )
                        })
                    {
                        attacks[attack_group.initiative as usize] =
                            Some((attack_idx, defend_idx, army));
                        taken |= 1u32 << defend_group.initiative;
                    }
                }
            }

            let mut changed = false;
            for (attack_idx, defend_idx, army) in
                attacks[..groups].iter_mut().rev().flat_map(Option::take)
            {
                let (attack_group, defend_group) = match army {
                    Army::Immune => (&immune[attack_idx], &mut infection[defend_idx]),
                    Army::Infection => (&infection[attack_idx], &mut immune[defend_idx]),
                };
                let killed = attack_group.damage(defend_group) / defend_group.hit_points;
                defend_group.units = defend_group.units.saturating_sub(killed);
                changed |= killed > 0;
            }

            if !changed {
                return None;
            }

            immune.retain(|g| g.units > 0);
            infection.retain(|g| g.units > 0);

            if immune.is_empty() && infection.is_empty() {
                return None;
            } else if immune.is_empty() {
                return Some((infection.iter().map(|g| g.units).sum(), Army::Infection));
            } else if infection.is_empty() {
                return Some((immune.iter().map(|g| g.units).sum(), Army::Immune));
            }
        }
    }
}

impl Group {
    #[inline]
    fn effective_power(&self) -> u32 {
        self.units * self.damage
    }

    #[inline]
    fn damage(&self, target: &Group) -> u32 {
        self.effective_power() * target.modifiers[self.damage_type] as u32
    }
}

impl Ord for Group {
    #[inline]
    fn cmp(&self, other: &Self) -> Ordering {
        self.effective_power()
            .cmp(&other.effective_power())
            .reverse()
            .then(self.initiative.cmp(&other.initiative).reverse())
    }
}

impl PartialOrd for Group {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

examples!(Day24 -> (u32, u32) [
    {file: "day24_example0.txt", part1: 5216, part2: 51},
]);
