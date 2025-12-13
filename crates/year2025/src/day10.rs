use std::collections::HashMap;
use utils::array::ArrayVec;
use utils::bit::BitIterator;
use utils::prelude::*;

/// Finding the minimum number of operations that sum to a target.
///
/// This implementation is based on
/// [/u/tenthmascot's post "Bifurcate your way to victory!"](https://www.reddit.com/r/adventofcode/comments/1pk87hl/2025_day_10_part_2_bifurcate_your_way_to_victory/).
#[derive(Clone, Debug)]
pub struct Day10 {
    part1: u32,
    part2: u32,
}

const MAX_TARGETS: usize = 10;
const MAX_BUTTONS: usize = 15;
const MAX_PARITY_OPTIONS: usize = 16;

impl Day10 {
    pub fn new(input: &str, _: InputType) -> Result<Self, InputError> {
        let machine = parser::byte_map!(
            b'.' => false,
            b'#' => true,
        )
        .repeat_arrayvec::<MAX_TARGETS, _>(parser::noop(), 1)
        .map(|v| {
            (
                v.iter().rfold(0, |acc, &n| (acc << 1) | u32::from(n)),
                v.len(),
            )
        })
        .with_prefix(b'[')
        .with_suffix("] ")
        .then(
            parser::number_range(0..=MAX_TARGETS - 1)
                .repeat_arrayvec::<MAX_TARGETS, _>(b',', 1)
                .map_res(|v| {
                    let mut mask = 0u32;
                    for &n in v.iter().rev() {
                        if mask & (1 << n) != 0 {
                            return Err("duplicate button within wiring schematic");
                        }
                        mask |= 1 << n;
                    }
                    Ok(mask)
                })
                .with_prefix(b'(')
                .with_suffix(b')')
                .repeat_arrayvec::<MAX_BUTTONS, _>(b' ', 1),
        )
        .then(
            parser::u16()
                .repeat_arrayvec(b',', 1)
                .with_prefix(" {")
                .with_suffix(b'}'),
        )
        .map_res(|((lights, light_count), buttons, targets)| {
            let all_buttons = buttons.iter().copied().fold(0, |acc, b| acc | b);
            if all_buttons.trailing_ones() as usize != light_count
                || all_buttons.leading_zeros() as usize != 32 - light_count
            {
                return Err("wiring schematics do not match light count");
            }
            if targets.len() != light_count {
                return Err("joltage targets do not match light count");
            }
            Ok((lights, buttons, targets))
        })
        .with_eol();

        let (mut part1, mut part2) = (0, 0);
        for line in machine.parse_iterator(input) {
            let (lights, buttons, targets) = line?;
            let (p1, p2) = Self::calculate(lights, buttons, targets);
            part1 += p1;
            part2 += p2;
        }

        Ok(Self { part1, part2 })
    }

    #[must_use]
    pub fn part1(&self) -> u32 {
        self.part1
    }

    #[must_use]
    pub fn part2(&self) -> u32 {
        self.part2
    }

    #[inline]
    fn calculate(
        lights: u32,
        buttons: ArrayVec<u32, MAX_BUTTONS>,
        targets: ArrayVec<u16, MAX_TARGETS>,
    ) -> (u32, u32) {
        // Convert the button mask into an array for fast sums below
        let mut single_button_results = [[0u16; MAX_TARGETS]; MAX_BUTTONS];
        for (i, &b) in buttons.iter().enumerate() {
            for (bit_pos, _) in BitIterator::ones(b) {
                single_button_results[i][bit_pos as usize] = 1;
            }
        }

        // Precalculate the results of pressing every button combination, grouped by the parity of
        // the results
        let combination_count = 1usize << buttons.len();
        let mut combination_results = vec![[0u16; MAX_TARGETS]; combination_count];

        let parity_states = 1usize << targets.len();
        let mut parity_combinations = vec![ArrayVec::new(); parity_states];

        #[expect(clippy::needless_range_loop, reason = "range loop is ~50% faster")]
        for combination in 0..combination_count {
            let result = &mut combination_results[combination];
            for (button_idx, _) in BitIterator::ones(combination) {
                for i in 0..MAX_TARGETS {
                    result[i] += single_button_results[button_idx as usize][i];
                }
            }

            let parity_mask = result
                .iter()
                .enumerate()
                .fold(0, |acc, (i, &v)| acc | ((v & 1) as usize) << i);
            parity_combinations[parity_mask]
                .push(combination as u16)
                .expect("expected less than MAX_PARITY_OPTIONS options per parity mask");
        }

        // Minimum presses for each light to match its parity
        let part1 = parity_combinations[lights as usize]
            .iter()
            .map(|&combinations| combinations.count_ones())
            .min()
            .expect("no solution found");

        // Minimum presses for each counter to match its target
        let mut cache = HashMap::with_capacity(1024);
        cache.insert([0; MAX_TARGETS], Some(0));
        let part2 = Self::target_min_presses(
            targets.as_array(),
            &parity_combinations,
            &combination_results,
            &mut cache,
        )
        .expect("no solution found");

        (part1, part2)
    }

    fn target_min_presses(
        targets: &[u16; MAX_TARGETS],
        parity_combinations: &[ArrayVec<u16, MAX_PARITY_OPTIONS>],
        combination_results: &[[u16; MAX_TARGETS]],
        cache: &mut HashMap<[u16; MAX_TARGETS], Option<u32>>,
    ) -> Option<u32> {
        if let Some(cached_solution) = cache.get(targets) {
            return *cached_solution;
        }

        let parity_mask = targets
            .iter()
            .enumerate()
            .fold(0, |acc, (i, &v)| acc | ((v & 1) as usize) << i);

        let mut best = None;
        let mut remaining_targets = [0u16; MAX_TARGETS];
        for &combination in &parity_combinations[parity_mask] {
            // After pressing the buttons from the combination, the remaining targets must be even.
            // Any additional presses must be in pairs to preserve this even parity, so it is safe
            // to divide each remaining target by 2, solve recursively, then double the result.
            // This limits the recursion depth to O(log(max_target)).

            let mut possible = true;
            for ((&current, &amount), next) in targets
                .iter()
                .zip(&combination_results[combination as usize])
                .zip(remaining_targets.iter_mut())
            {
                possible &= current >= amount;
                *next = current.wrapping_sub(amount) / 2;
            }

            if possible
                && let Some(remaining_solution) = Self::target_min_presses(
                    &remaining_targets,
                    parity_combinations,
                    combination_results,
                    cache,
                )
                && let solution = combination.count_ones() + 2 * remaining_solution
                && best.is_none_or(|b| b > solution)
            {
                best = Some(solution);
            }
        }

        cache.insert(*targets, best);
        best
    }
}

examples!(Day10 -> (u32, u32) [
    {file: "day10_example0.txt", part1: 7, part2: 33},
]);
