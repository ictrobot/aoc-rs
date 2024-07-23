use utils::prelude::*;

/// Matching Aunt Sue's gift clues.
#[derive(Clone, Debug)]
pub struct Day16<'a> {
    input: &'a str,
}

impl<'a> Day16<'a> {
    pub fn new(input: &'a str, _: InputType) -> Result<Self, InputError> {
        Ok(Self { input })
    }

    #[must_use]
    pub fn part1(&self) -> usize {
        self.find(|s| {
            matches!(
                s,
                "children: 3"
                    | "cats: 7"
                    | "samoyeds: 2"
                    | "pomeranians: 3"
                    | "akitas: 0"
                    | "vizslas: 0"
                    | "goldfish: 5"
                    | "trees: 3"
                    | "cars: 2"
                    | "perfumes: 1"
            )
        })
    }

    #[must_use]
    pub fn part2(&self) -> usize {
        self.find(|s| matches!(
            s.as_bytes(),
            b"children: 3"
                | b"samoyeds: 2"
                | b"akitas: 0"
                | b"vizslas: 0"
                | b"cars: 2"
                | b"perfumes: 1"
                // Greater than 7 cats
                | [b'c', b'a', b't', b's', b':', b' ', b'8'..=b'9']
                | [b'c', b'a', b't', b's', b':', b' ', b'1'..=b'9', b'0'..=b'9', ..]
                // Greater than 3 trees
                | [b't', b'r', b'e', b'e', b's', b':', b' ', b'4'..=b'9']
                | [b't', b'r', b'e', b'e', b's', b':', b' ', b'1'..=b'9', b'0'..=b'9', ..]
                // Fewer than 3 pomeranians
                | [b'p', b'o', b'm', b'e', b'r', b'a', b'n', b'i', b'a', b'n', b's', b':', b' ', b'0'..=b'2']
                // Fewer than 5 goldfish
                | [b'g', b'o', b'l', b'd', b'f', b'i', b's', b'h', b':', b' ', b'0'..=b'4']
        ))
    }

    fn find(&self, f: impl Fn(&str) -> bool) -> usize {
        self.input
            .lines()
            .position(|line| {
                let (_, line) = line.split_once(": ").unwrap();
                line.split(", ").all(&f)
            })
            .expect("no aunts match")
            + 1
    }
}

examples!(Day16<'_> -> (usize, usize) []);
