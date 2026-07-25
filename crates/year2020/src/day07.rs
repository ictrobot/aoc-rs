use utils::array::ArrayVec;
use utils::hash::FastMap;
use utils::prelude::*;

/// Counting nested bags.
#[derive(Clone, Debug)]
pub struct Day07 {
    rules: Vec<Contents>,
    target: usize,
}

const MAX_INSIDE: usize = 4;
type Contents = ArrayVec<(u16, u8), MAX_INSIDE>;

#[derive(Clone, Copy, Debug)]
enum State<T> {
    Unvisited,
    CurrentlyVisiting,
    Visited(T),
}

impl Day07 {
    pub fn new(input: &str, _: InputType) -> Result<Self, InputError> {
        let word = parser::take_while1(u8::is_ascii_lowercase);
        let color = word
            .with_suffix(b' ')
            .then(word)
            .with_consumed()
            .map(|(_, consumed)| consumed);
        let rule = color
            .with_suffix(" bags contain ")
            .then(
                parser::parse_tree!(
                    ("1 ".commit(), c @ color, " bag") => (1, c),
                    (count @ parser::nonzero_u8(), b' ', c @ color, " bags") => (count.get(), c),
                )
                .repeat_arrayvec::<MAX_INSIDE, _>(", ", 1)
                .or("no other bags".map(|_| ArrayVec::new())),
            )
            .with_suffix(b'.')
            .with_eol();

        let mut bag_ids = FastMap::with_capacity(1024);
        let mut rules = Vec::with_capacity(1024);
        let mut intern = |name, rules: &mut Vec<Option<Contents>>| {
            *bag_ids.entry(name).or_insert_with(|| {
                let id = rules.len() as u16;
                rules.push(None);
                id
            })
        };
        for item in rule.parse_iterator(input) {
            let (outer, contains) = item?;
            let outer_id = intern(outer, &mut rules);
            if rules[usize::from(outer_id)].is_some() {
                return Err(InputError::new(input, outer, "duplicate bag rule"));
            }

            let mut children = Contents::new();
            for &(count, inner) in &contains {
                let inner_id = intern(inner, &mut rules);
                if inner_id == outer_id {
                    return Err(InputError::new(input, inner, "bag cannot contain itself"));
                }
                if children.iter().any(|&(id, _)| id == inner_id) {
                    return Err(InputError::new(input, inner, "duplicate bag"));
                }
                children.push((inner_id, count)).unwrap();
            }

            rules[usize::from(outer_id)] = Some(children);
        }

        let Some(rules) = rules.into_iter().collect() else {
            return Err(InputError::new(input, 0, "missing bag rule"));
        };
        let Some(&target) = bag_ids.get(b"shiny gold".as_slice()) else {
            return Err(InputError::new(input, 0, "no shiny gold bag"));
        };

        Ok(Self {
            rules,
            target: usize::from(target),
        })
    }

    #[must_use]
    pub fn part1(&self) -> usize {
        fn contains_target(
            target: usize,
            rules: &[Contents],
            bag: usize,
            states: &mut [State<bool>],
        ) -> bool {
            match states[bag] {
                State::CurrentlyVisiting => panic!("no solution found: rules contain a cycle"),
                State::Visited(b) => return b,
                State::Unvisited => states[bag] = State::CurrentlyVisiting,
            }

            let mut result = false;
            for &(child, _) in &rules[bag] {
                let child = usize::from(child);
                result |= child == target || contains_target(target, rules, child, states);
            }

            states[bag] = State::Visited(result);
            result
        }

        let mut states = vec![State::Unvisited; self.rules.len()];
        (0..self.rules.len())
            .filter(|&bag| contains_target(self.target, &self.rules, bag, &mut states))
            .count()
    }

    #[must_use]
    pub fn part2(&self) -> u64 {
        fn total_bags(rules: &[Contents], bag: usize, states: &mut [State<u64>]) -> u64 {
            match states[bag] {
                State::CurrentlyVisiting => panic!("no solution found: rules contain a cycle"),
                State::Visited(b) => return b,
                State::Unvisited => states[bag] = State::CurrentlyVisiting,
            }

            let mut result = 0u64;
            for &(child, count) in &rules[bag] {
                result += count as u64 * (1 + total_bags(rules, usize::from(child), states));
            }

            states[bag] = State::Visited(result);
            result
        }

        let mut states = vec![State::Unvisited; self.rules.len()];
        total_bags(&self.rules, self.target, &mut states)
    }
}

examples!(Day07 -> (usize, u64) [
    {file: "day07_example0.txt", part1: 4, part2: 32},
    {file: "day07_example1.txt", part2: 126},
]);
