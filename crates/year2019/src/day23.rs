use crate::intcode::features::Day09Features;
use crate::intcode::{Event, Interpreter};
use utils::prelude::*;

/// Interpreting machine code and routing packets between instances.
#[derive(Clone, Debug)]
pub struct Day23 {
    part1: i64,
    part2: i64,
}

const COMPUTER_COUNT: usize = 50;
const NAT_ADDRESS: usize = 255;

impl Day23 {
    pub fn new(input: &str, _: InputType) -> Result<Self, InputError> {
        let base = Interpreter::parse(input, 1)?;
        let mut computers: [Interpreter; COMPUTER_COUNT] = std::array::from_fn(|address| {
            let mut computer = base.clone();
            computer.push_input(address as i64);
            computer
        });

        let mut pending_indexes = (0..COMPUTER_COUNT).rev().collect::<Vec<_>>();
        let mut part1 = None;
        let mut nat_packet = None;
        let mut last_nat_y = None;

        loop {
            let index = match pending_indexes.pop() {
                Some(index) => index,
                None if let Some((_, y)) = nat_packet
                    && Some(y) == last_nat_y =>
                {
                    return Ok(Self {
                        part1: part1.expect("packet has been sent to NAT address"),
                        part2: y,
                    });
                }
                None if let Some((x, y)) = nat_packet => {
                    last_nat_y = Some(y);
                    computers[0].input.extend([x, y]);
                    0
                }
                None => {
                    return Err(InputError::new(
                        input,
                        0,
                        "expected packet to be sent to the NAT before the network became idle",
                    ));
                }
            };

            let mut sent_idle = false;
            loop {
                match computers[index].run::<Day09Features>() {
                    Event::Halt => {
                        return Err(InputError::new(
                            input,
                            0,
                            "expected program to output or request input, but it halted",
                        ));
                    }
                    Event::Input if sent_idle => break,
                    Event::Input => {
                        computers[index].push_input(-1);
                        sent_idle = true;
                    }
                    Event::Output(address) => {
                        let mut next_output = || match computers[index].run::<Day09Features>() {
                            Event::Output(value) => Ok(value),
                            Event::Halt | Event::Input => Err(InputError::new(
                                input,
                                0,
                                "expected program to output three values",
                            )),
                        };
                        let (x, y) = (next_output()?, next_output()?);

                        match usize::try_from(address) {
                            Ok(address @ 0..COMPUTER_COUNT) => {
                                let was_idle = computers[address].input.is_empty();
                                computers[address].input.extend([x, y]);
                                if address != index && was_idle {
                                    pending_indexes.push(address);
                                }
                            }
                            Ok(NAT_ADDRESS) => {
                                part1.get_or_insert(y);
                                nat_packet = Some((x, y));
                            }
                            _ => {
                                return Err(InputError::new(
                                    input,
                                    0,
                                    "expected packet address from 0 to 49 or 255",
                                ));
                            }
                        }
                    }
                }
            }
        }
    }

    #[must_use]
    pub fn part1(&self) -> i64 {
        self.part1
    }

    #[must_use]
    pub fn part2(&self) -> i64 {
        self.part2
    }
}

examples!(Day23 -> (i64, i64) []);
