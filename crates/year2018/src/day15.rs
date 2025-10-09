use utils::prelude::*;

/// Simulating battle outcomes between elves and goblins.
#[derive(Clone, Debug)]
pub struct Day15 {
    wall_mask: [u32; 32],
    units: Vec<Unit>,
}

#[derive(Clone, Debug)]
struct Battle {
    units: Vec<Unit>,
    grid: [u16; 1024],
    wall_mask: [u32; 32],
    unit_masks: [[u32; 32]; 2],
    bfs_layers: Vec<[u32; 32]>,
    unit_counts: [usize; 2],
    attack_power: [u32; 2],
    elves_must_live: bool,
}

#[derive(Clone, Debug)]
struct Unit {
    pos: usize,
    unit_type: UnitType,
    health: u32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum UnitType {
    Elf,
    Goblin,
}

impl Day15 {
    pub fn new(input: &str, _: InputType) -> Result<Self, InputError> {
        let mut wall_mask = [u32::MAX; 32];
        let mut units = Vec::with_capacity(32);

        for (row, line) in input.lines().enumerate() {
            if row >= 32 {
                return Err(InputError::new(input, line, "too many rows"));
            }
            if line.len() > 32 {
                return Err(InputError::new(input, line, "too many columns"));
            }
            for (col, b) in line.bytes().enumerate() {
                let unit_type = match b {
                    b'.' => None,
                    b'E' => Some(UnitType::Elf),
                    b'G' => Some(UnitType::Goblin),
                    b'#' => continue,
                    _ => {
                        return Err(InputError::new(
                            input,
                            b as char,
                            "expected '.', '#', 'E' or 'G'",
                        ));
                    }
                };

                wall_mask[row] &= !(1 << col);

                if let Some(unit_type) = unit_type {
                    units.push(Unit {
                        pos: row * 32 + col,
                        unit_type,
                        health: 200,
                    });
                }
            }
        }

        if wall_mask[0] != u32::MAX
            || wall_mask[31] != u32::MAX
            || wall_mask.iter().any(|&x| x & 1 == 0 || x & (1 << 31) == 0)
        {
            return Err(InputError::new(input, 0, "expected grid to be enclosed"));
        }

        Ok(Self { wall_mask, units })
    }

    #[must_use]
    pub fn part1(&self) -> u32 {
        self.battle_outcome(3, false).unwrap()
    }

    #[must_use]
    pub fn part2(&self) -> u32 {
        for attack in 4..=200 {
            if let Some(score) = self.battle_outcome(attack, true) {
                return score;
            }
        }
        panic!("no solution found");
    }

    fn battle_outcome(&self, elf_attack: u32, elves_must_live: bool) -> Option<u32> {
        let mut b = Battle {
            units: self.units.clone(),
            grid: [u16::MAX; 1024],
            wall_mask: self.wall_mask,
            unit_masks: [[0u32; 32]; 2],
            unit_counts: self.units.iter().fold([0, 0], |mut acc, u| {
                acc[u.unit_type as usize] += 1;
                acc
            }),
            attack_power: [elf_attack, 3],
            bfs_layers: vec![[0u32; 32]; 64],
            elves_must_live,
        };
        for i in 0..self.units.len() {
            b.add_unit_to_grid(i);
        }
        b.outcome()
    }
}

impl Battle {
    fn outcome(mut self) -> Option<u32> {
        for round in 0.. {
            for unit_idx in 0..self.units.len() {
                if self.units[unit_idx].health == 0 {
                    continue;
                }

                // Move
                let mut adjacent_enemy = self.adjacent_enemy(unit_idx);
                if adjacent_enemy.is_none()
                    && let Some(next) = self.next_move(unit_idx)
                {
                    self.remove_unit_from_grid(unit_idx);
                    self.units[unit_idx].pos = next;
                    self.add_unit_to_grid(unit_idx);

                    adjacent_enemy = self.adjacent_enemy(unit_idx);
                }

                // Attack
                let Some(enemy_idx) = adjacent_enemy else {
                    continue;
                };

                let attack = self.attack_power[self.units[unit_idx].unit_type as usize];
                self.units[enemy_idx].health = self.units[enemy_idx].health.saturating_sub(attack);

                if self.units[enemy_idx].health == 0 {
                    self.remove_unit_from_grid(enemy_idx);

                    let enemy_type = self.units[enemy_idx].unit_type;
                    if enemy_type == UnitType::Elf && self.elves_must_live {
                        return None;
                    }

                    self.unit_counts[enemy_type as usize] -= 1;
                    if self.unit_counts[enemy_type as usize] == 0 {
                        let round_complete =
                            self.units[unit_idx + 1..].iter().all(|u| u.health == 0);
                        let full_rounds = round + u32::from(round_complete);
                        let remaining_health = self.units.iter().map(|u| u.health).sum::<u32>();
                        return Some(full_rounds * remaining_health);
                    }
                }
            }

            self.units.retain(|u| u.health > 0);
            self.units.sort_unstable_by_key(|u| u.pos);
            for (i, u) in self.units.iter().enumerate() {
                self.grid[u.pos] = i as u16;
            }
        }

        unreachable!();
    }

    fn adjacent_enemy(&self, unit_idx: usize) -> Option<usize> {
        let (mut enemy_index, mut enemy_health) = (None, u32::MAX);

        let Unit { pos, unit_type, .. } = self.units[unit_idx];
        for enemy_pos in [pos - 32, pos - 1, pos + 1, pos + 32] {
            if let Some(&idx @ 0..1024) = self.grid.get(enemy_pos)
                && self.units[idx as usize].unit_type != unit_type
                && self.units[idx as usize].health < enemy_health
            {
                enemy_index = Some(idx as usize);
                enemy_health = self.units[idx as usize].health;
            }
        }

        enemy_index
    }

    fn next_move(&mut self, unit_idx: usize) -> Option<usize> {
        let unit_type = self.units[unit_idx].unit_type;
        let ally_mask = &self.unit_masks[unit_type as usize];
        let enemy_mask = self.unit_masks[unit_type.enemy() as usize];

        let mut enemy_adjacent_mask = [0u32; 32];
        for i in 1..31 {
            enemy_adjacent_mask[i] =
                (enemy_mask[i] << 1) | (enemy_mask[i] >> 1) | enemy_mask[i - 1] | enemy_mask[i + 1];
        }

        let unit_pos = self.units[unit_idx].pos;
        let unit_adjacent = [unit_pos - 32, unit_pos - 1, unit_pos + 1, unit_pos + 32];

        // Forward BFS pass from unit adjacent squares to enemy adjacent squares.
        // bfs_layers[i] holds the set of squares reachable after i steps.
        self.bfs_layers[0].fill(0);
        for pos in unit_adjacent {
            let (row, col) = (pos / 32, pos % 32);
            if (self.wall_mask[row] | enemy_mask[row] | ally_mask[row]) & (1 << col) != 0 {
                continue;
            }
            if enemy_adjacent_mask[row] & (1 << col) != 0 {
                return Some(pos);
            }
            self.bfs_layers[0][row] |= 1 << col;
        }

        let mut steps = 0;
        let (target_row, target_col) = 'forward: loop {
            steps += 1;
            if steps >= self.bfs_layers.len() {
                self.bfs_layers.push([0u32; 32]);
            }

            let [prev, next] = self
                .bfs_layers
                .get_disjoint_mut([steps - 1, steps])
                .unwrap();
            for i in 1..31 {
                next[i] = (prev[i] | (prev[i] << 1) | (prev[i] >> 1) | prev[i - 1] | prev[i + 1])
                    & !(self.wall_mask[i] | ally_mask[i]);
            }

            // Check if the pass reached an enemy adjacent square.
            // This must be checked in (row, call) order to select the right target when multiple
            // are reachable.
            for row in 1..31 {
                let mask = next[row] & enemy_adjacent_mask[row];
                if mask != 0 {
                    break 'forward (row, mask.trailing_zeros() as usize);
                }
            }

            if next == prev {
                return None;
            }
        };

        // Reverse BFS pass from the target enemy adjacent square back to one of the 4 unit
        // adjacent squares the unit can move to.
        self.bfs_layers[steps].fill(0);
        self.bfs_layers[steps][target_row] |= 1 << target_col;

        for step in (1..=steps).rev() {
            let [next, prev] = self.bfs_layers.get_disjoint_mut([step - 1, step]).unwrap();

            for i in 1..31 {
                next[i] &= (prev[i] << 1) | (prev[i] >> 1) | prev[i - 1] | prev[i + 1];
            }
        }

        // Must be checked in (row, col) order
        for pos in unit_adjacent {
            if self.bfs_layers[0][pos / 32] & (1 << (pos % 32)) != 0 {
                return Some(pos);
            }
        }

        unreachable!("forward bfs succeeded, so there must be a reverse path");
    }

    fn remove_unit_from_grid(&mut self, unit_index: usize) {
        let Unit { pos, unit_type, .. } = self.units[unit_index];

        self.grid[pos] = u16::MAX;
        self.unit_masks[unit_type as usize][pos / 32] &= !(1 << (pos % 32));
    }

    fn add_unit_to_grid(&mut self, unit_index: usize) {
        let Unit { pos, unit_type, .. } = self.units[unit_index];

        self.grid[pos] = unit_index as u16;
        self.unit_masks[unit_type as usize][pos / 32] |= 1 << (pos % 32);
    }
}

impl UnitType {
    fn enemy(self) -> Self {
        match self {
            UnitType::Elf => UnitType::Goblin,
            UnitType::Goblin => UnitType::Elf,
        }
    }
}

examples!(Day15 -> (u32, u32) [
    {file: "day15_example0.txt", part1: 27730, part2: 4988},
    {file: "day15_example1.txt", part1: 36334},
    {file: "day15_example2.txt", part1: 39514, part2: 31284},
    {file: "day15_example3.txt", part1: 27755, part2: 3478},
    {file: "day15_example4.txt", part1: 28944, part2: 6474},
    {file: "day15_example5.txt", part1: 18740, part2: 1140},
]);
