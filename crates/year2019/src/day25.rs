use crate::intcode::features::Day09Features;
use crate::intcode::{Event, Interpreter};
use core::assert_matches;
use utils::array::ArrayVec;
use utils::bit::BitIterator;
use utils::geometry::Direction;
use utils::prelude::*;

/// Interpreting machine code to complete a text adventure game.
///
/// This solution makes many assumptions derived from running the puzzle input.
#[derive(Clone, Debug)]
pub struct Day25 {
    interpreter: Interpreter,
}

const DIRECTIONS: [&str; 4] = ["north", "east", "south", "west"];
const DOORS_HEADER: &str = "Doors here lead:";
const ITEMS_HEADER: &str = "Items here:";
const TOO_LIGHT: &str = "Droids on this ship are heavier than the detected value";
const TOO_HEAVY: &str = "Droids on this ship are lighter than the detected value";
const CODE_PREFIX: &str = "typing ";
const CODE_SUFFIX: &str = " on the keypad at the main airlock";

impl Day25 {
    pub fn new(input: &str, _: InputType) -> Result<Self, InputError> {
        Ok(Self {
            interpreter: Interpreter::parse(input, 1)?,
        })
    }

    #[must_use]
    pub fn part1(&self) -> u64 {
        Searcher {
            interpreter: self.interpreter.clone(),
            output: String::with_capacity(512),
            items: Vec::new(),
            path: Vec::new(),
            floor_route: Vec::new(),
        }
        .search()
    }

    #[must_use]
    pub fn part2(&self) -> &'static str {
        "🎄"
    }
}

#[derive(Debug)]
struct Searcher {
    interpreter: Interpreter,
    output: String,
    items: Vec<String>,
    path: Vec<Direction>,
    floor_route: Vec<Direction>,
}

impl Searcher {
    fn search(&mut self) -> u64 {
        self.read_output();

        // Explore the whole map, taking every safe item and recording the route to the
        // pressure-sensitive floor
        self.explore(None);

        // Follow the route back to the room before the pressure-sensitive floor
        let mut route = std::mem::take(&mut self.floor_route);
        let Some(floor_direction) = route.pop() else {
            panic!("expected to find the pressure-sensitive floor")
        };
        for direction in route {
            self.move_command(direction);
        }

        assert!(
            self.items.len() < u64::BITS as usize,
            "expected fewer inventory items"
        );
        let combinations = 1u64 << self.items.len();

        // Drop subsets of the inventory until the weight is accepted.
        // Skip sets that are supersets of a set known to be too light or subsets of one known to be
        // too heavy. Trying sets in Gray code order means that each set differs from the previous
        // one by one bit, increasing the number of subset/superset matches and skipped sets.
        let mut dropped = 0;
        let mut too_heavy = Vec::<u64>::new();
        let mut too_light = Vec::<u64>::new();
        for step in 0..combinations {
            let next = step ^ (step >> 1);
            if too_light.iter().any(|&mask| mask & !next == 0)
                || too_heavy.iter().any(|&mask| next & !mask == 0)
            {
                continue;
            }

            for (item, _) in BitIterator::ones(next & !dropped) {
                self.drop_item(item as usize);
            }
            for (item, _) in BitIterator::ones(dropped & !next) {
                self.take_item(item as usize);
            }
            dropped = next;

            self.move_command(floor_direction);

            if let Some((_, rest)) = self.output.split_once(CODE_PREFIX)
                && let Some((code, _)) = rest.split_once(CODE_SUFFIX)
                && let Ok(code) = code.parse()
            {
                return code;
            } else if self.output.contains(TOO_LIGHT) {
                too_light.push(dropped);
            } else if self.output.contains(TOO_HEAVY) {
                too_heavy.push(dropped);
            } else {
                panic!("unexpected output: {}", self.output);
            }
        }

        panic!("no solution found")
    }

    fn explore(&mut self, entered: Option<Direction>) {
        if let Some(direction) = entered {
            self.move_command(direction);
            if self.output.contains(TOO_LIGHT) || self.output.contains(TOO_HEAVY) {
                self.floor_route.clone_from(&self.path);
                return;
            }
        }

        let mut lines = self.output.lines();
        let mut doors = ArrayVec::<Direction, 4>::new();
        if lines.any(|line| line == DOORS_HEADER) {
            for line in lines.by_ref() {
                let Some(value) = line.strip_prefix("- ") else {
                    break;
                };

                let Some(direction) = DIRECTIONS
                    .iter()
                    .position(|&x| x == value)
                    .map(|x| Direction::from(x as u8))
                else {
                    panic!("unexpected door: {value}");
                };

                if entered.is_none_or(|entered| direction != !entered) {
                    doors.push(direction).unwrap();
                }
            }
        }

        let mut room_items = Vec::new();
        if lines.any(|line| line == ITEMS_HEADER) {
            for line in lines {
                let Some(item) = line.strip_prefix("- ") else {
                    break;
                };
                if !Self::dangerous(item) {
                    room_items.push(item.to_owned());
                }
            }
        }

        for item in room_items {
            self.items.push(item);
            self.take_item(self.items.len() - 1);
        }

        for &direction in &doors {
            self.path.push(direction);
            self.explore(Some(direction));
            self.path.pop();
        }

        if let Some(direction) = entered {
            self.move_command(!direction);
        }
    }

    fn move_command(&mut self, direction: Direction) {
        self.interpreter.push_bytes(DIRECTIONS[direction as usize]);
        self.interpreter.push_bytes("\n");

        self.read_output();
        assert!(
            self.output.trim_ascii_start().starts_with("== "),
            "unexpected output after move command:\n{}",
            self.output
        );
    }

    fn take_item(&mut self, item: usize) {
        self.item_command("take", item);
    }

    fn drop_item(&mut self, item: usize) {
        self.item_command("drop", item);
    }

    fn item_command(&mut self, action: &str, item: usize) {
        self.interpreter.push_bytes(action);
        self.interpreter.push_bytes(" ");
        self.interpreter.push_bytes(&self.items[item]);
        self.interpreter.push_bytes("\n");

        self.read_output();
        assert!(
            self.output
                .trim_ascii_start()
                .strip_prefix("You ")
                .and_then(|rest| rest.strip_prefix(action))
                .and_then(|rest| rest.strip_prefix(" the "))
                .is_some_and(|rest| rest.starts_with(&self.items[item])),
            "unexpected output after item command: {}",
            self.output
        );
    }

    fn read_output(&mut self) {
        self.output.clear();
        while let Event::Output(x) = self.interpreter.run::<Day09Features>() {
            assert_matches!(x, 0..=127, "expected ascii output");
            self.output.push(x as u8 as char);
        }
    }

    fn dangerous(item: &str) -> bool {
        matches!(
            item,
            "escape pod" | "giant electromagnet" | "infinite loop" | "molten lava" | "photons"
        )
    }
}

examples!(Day25 -> (u64, &'static str) []);
