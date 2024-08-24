use utils::prelude::*;

/// Comparing chip values.
#[derive(Clone, Debug)]
pub struct Day10 {
    part1: u8,
    part2: u32,
}

#[derive(Copy, Clone, Debug)]
enum Instruction {
    Value { value: u8, bot: u8 },
    Bot { num: u8, low: Output, high: Output },
}

#[derive(Copy, Clone, Debug)]
struct Bot {
    num: u8,
    low: Output,
    high: Output,
    value: Option<u8>,
}

#[derive(Copy, Clone, Debug)]
enum Output {
    Bot(u8),
    Output(u8),
}

impl Day10 {
    pub fn new(input: &str, input_type: InputType) -> Result<Self, InputError> {
        let output = parser::u8()
            .with_prefix("bot ")
            .map(Output::Bot)
            .or(parser::u8().with_prefix("output ").map(Output::Output));
        let bot = parser::u8()
            .with_prefix("bot ")
            .then(output.with_prefix(" gives low to "))
            .then(output.with_prefix(" and high to "))
            .map(|(num, low, high)| Instruction::Bot { num, low, high });
        let value = parser::u8()
            .with_prefix("value ")
            .with_suffix(" goes to bot ")
            .then(parser::u8())
            .map(|(value, bot)| Instruction::Value { value, bot });

        let instructions = bot.or(value).parse_lines(input)?;

        let mut bots = instructions
            .iter()
            .filter_map(|&instruction| match instruction {
                Instruction::Value { .. } => None,
                Instruction::Bot { num, low, high } => Some(Bot {
                    num,
                    low,
                    high,
                    value: None,
                }),
            })
            .collect::<Vec<_>>();
        bots.sort_unstable_by_key(|b| b.num);
        if let Some((i, _)) = bots.iter().enumerate().find(|&(i, b)| i != b.num as usize) {
            return Err(InputError::new(input, input, format!("bot {i} missing")));
        }

        let mut part1 = 0;
        let mut compare_fn = |index, min, max| {
            if (input_type == InputType::Real && min == 17 && max == 61)
                || (input_type == InputType::Example && min == 2 && max == 5)
            {
                part1 = index;
            }
        };

        let mut part2 = 1;
        let mut output_fn = |index, value| {
            if index <= 2 {
                part2 *= value as u32;
            }
        };

        for &instruction in &instructions {
            if let Instruction::Value { bot, value } = instruction {
                Self::give(bot, value, &mut bots, &mut compare_fn, &mut output_fn);
            }
        }

        Ok(Self { part1, part2 })
    }

    fn give(
        bot: u8,
        value: u8,
        bots: &mut [Bot],
        compare_fn: &mut impl FnMut(u8, u8, u8),
        output_fn: &mut impl FnMut(u8, u8),
    ) {
        let bot = &mut bots[bot as usize];
        if let Some(value2) = bot.value.take() {
            let min = value.min(value2);
            let max = value.max(value2);
            compare_fn(bot.num, min, max);

            for (output, value) in [(bot.low, min), (bot.high, max)] {
                match output {
                    Output::Bot(index) => Self::give(index, value, bots, compare_fn, output_fn),
                    Output::Output(index) => output_fn(index, value),
                }
            }
        } else {
            bot.value = Some(value);
        }
    }

    #[must_use]
    pub fn part1(&self) -> u8 {
        self.part1
    }

    #[must_use]
    pub fn part2(&self) -> u32 {
        self.part2
    }
}

examples!(Day10 -> (u8, u32) [
    {
        input: "value 5 goes to bot 2\n\
            bot 2 gives low to bot 1 and high to bot 0\n\
            value 3 goes to bot 1\n\
            bot 1 gives low to output 1 and high to bot 0\n\
            bot 0 gives low to output 2 and high to output 0\n\
            value 2 goes to bot 2",
        part1: 2,
        part2: 30,
    },
]);
