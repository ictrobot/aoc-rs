use std::collections::HashMap;
use std::str;
use utils::prelude::*;

/// Finding the unbalanced subtree.
#[derive(Clone, Debug)]
pub struct Day07<'a> {
    programs: Vec<Program<'a>>,
    bottom: usize,
}

#[derive(Clone, Debug)]
struct Program<'a> {
    name: &'a [u8],
    weight: u32,
    parent: Option<usize>,
    children: Vec<usize>,
}

impl<'a> Day07<'a> {
    pub fn new(input: &'a str, _: InputType) -> Result<Self, InputError> {
        let name = parser::take_while1(u8::is_ascii_lowercase);
        let lines = name
            .with_suffix(" (")
            .then(parser::u32().with_suffix(")"))
            .then(name.repeat(", ", 0).with_prefix(" -> ".optional()))
            .parse_lines(input)?;

        let mut programs = lines
            .iter()
            .map(|&(name, weight, _)| Program {
                name,
                weight,
                parent: None,
                children: Vec::new(),
            })
            .collect::<Vec<_>>();

        let name_map = lines
            .iter()
            .enumerate()
            .map(|(index, &(name, _, _))| (name, index))
            .collect::<HashMap<_, _>>();

        for (parent, (_, _, children)) in lines.into_iter().enumerate() {
            // Use into_iter so that the children Vec<&[u8]> can be reused as the children
            // Vec<usize>, avoiding an extra allocation and free per input line.
            let children = children
                .into_iter()
                .map(|name| {
                    if let Some(&child) = name_map.get(name) {
                        programs[child].parent = Some(parent);
                        Ok(child)
                    } else {
                        Err(InputError::new(
                            input,
                            0,
                            format!("program {:?} missing on LHS", str::from_utf8(name).unwrap()),
                        ))
                    }
                })
                .collect::<Result<Vec<_>, _>>()?;
            programs[parent].children = children;
        }

        let Some(bottom) = programs.iter().position(|p| p.parent.is_none()) else {
            return Err(InputError::new(
                input,
                0,
                "expected one program to have no parent",
            ));
        };

        Ok(Self { programs, bottom })
    }

    #[must_use]
    pub fn part1(&self) -> &str {
        str::from_utf8(self.programs[self.bottom].name).unwrap()
    }

    #[must_use]
    pub fn part2(&self) -> u32 {
        self.check(self.bottom).expect_err("tower is balanced")
    }

    fn check(&self, index: usize) -> Result<u32, u32> {
        let program = &self.programs[index];
        if program.children.is_empty() {
            // Programs with no children are always balanced.
            Ok(program.weight)
        } else if program.children.len() < 3 {
            // Programs with one child are balanced as there aren't multiple sub-towers to disagree.
            // Programs with two children must also be balanced as it is impossible to tell which
            // sub-tower is wrong if you only have two different values.
            let first_weight = self.check(program.children[0])?;
            let all_children = first_weight * program.children.len() as u32;
            Ok(program.weight + all_children)
        } else {
            let first_weight = self.check(program.children[0])?;
            let mut first_matches = 0;
            let mut second_weight = None;
            for &child in &program.children[1..] {
                let weight = self.check(child)?;
                if weight == first_weight {
                    first_matches += 1;
                } else if second_weight.is_none() {
                    second_weight = Some((weight, child));
                } else if second_weight.unwrap().0 != weight {
                    panic!(
                        "program {:?} has children with 3 different weights",
                        str::from_utf8(program.name).unwrap()
                    );
                }
            }

            let Some((second_weight, second_index)) = second_weight else {
                // All children match, this sub-tower is balanced
                let all_children = first_weight * program.children.len() as u32;
                return Ok(program.weight + all_children);
            };

            // Found the unbalanced sub-tower
            let (correct_weight, wrong_weight, wrong_index) = if first_matches == 0 {
                // First child wrong
                (second_weight, first_weight, program.children[0])
            } else {
                // Second weight wrong
                (first_weight, second_weight, second_index)
            };

            Err(correct_weight + self.programs[wrong_index].weight - wrong_weight)
        }
    }
}

examples!(Day07<'_> -> (&'static str, u32) [
    {file: "day07_example0.txt", part1: "tknk", part2: 60},
]);
