use crate::elfcode::{HookControlFlow, Instruction, Interpreter, Register};
use std::collections::HashSet;
use utils::prelude::*;

/// Interpreting assembly to find the longest running input.
///
/// See also [day 16](crate::Day16) and [day 19](crate::Day19).
#[derive(Clone, Debug)]
pub struct Day21 {
    interpreter: Interpreter,
}

impl Day21 {
    pub fn new(input: &str, _: InputType) -> Result<Self, InputError> {
        let interpreter = Interpreter::new(input)?;

        let instruction_pointer = interpreter.instruction_pointer();
        if instruction_pointer == Register::A {
            return Err(InputError::new(input, 0, "expected #ip to be non-zero"));
        }

        let instructions = interpreter.instructions();
        if instructions
            .iter()
            .flat_map(|i| i.registers())
            .filter(|&r| r == Register::A)
            .count()
            != 1
        {
            return Err(InputError::new(
                input,
                0,
                "expected exactly one use of register 0",
            ));
        }

        if let [
            ..,
            Instruction::Eqrr(_, Register::A, tmp),
            Instruction::Addr(tmp2, ip, ip2),
            Instruction::Seti(_, ip3),
        ] = *instructions
            && tmp == tmp2
            && ip == ip2
            && ip == ip3
            && ip == instruction_pointer
            && tmp != instruction_pointer
        {
            // Register 0 is used in exactly one place, in the 3rd last instruction, and the outer
            // loop terminates when it matches a computed value.
            Ok(Self { interpreter })
        } else {
            Err(InputError::new(
                input,
                input.len() - 1,
                "expected register 0 matching to control termination of the outer loop",
            ))
        }
    }

    #[must_use]
    pub fn part1(&self) -> u32 {
        self.next(&mut [0; 6])
    }

    #[must_use]
    pub fn part2(&self) -> u32 {
        // Using a HashSet is faster than using Brent's algorithm as it minimizes the number of next
        // calls, which are slow.
        let mut reg = [0; 6];
        let mut seen = HashSet::with_capacity(20000);
        let mut last_first_seen = 0;
        loop {
            let target = self.next(&mut reg);
            if seen.insert(target) {
                last_first_seen = target;
            } else {
                break;
            }
        }
        last_first_seen
    }

    fn next(&self, reg: &mut [u32; 6]) -> u32 {
        let mut target = None;

        self.interpreter
            .run(reg, |instructions, instruction_pointer, reg| {
                let addr = reg[instruction_pointer] as usize;

                if let Instruction::Eqrr(tgt, Register::A, tmp) = instructions[addr] {
                    target = Some(reg[tgt]);

                    // Simulate r0 not equaling the target, so the program always continues when
                    // the interpreter is next resumed.
                    reg[tmp] = 0;
                    reg[instruction_pointer] += 1;

                    return HookControlFlow::Halt;
                }

                // Recognize the division by 256 and replace it with a native computation.
                //  start: seti #0 $quo
                //         addi $quo #1 $tmp
                //         muli $tmp #256 $tmp
                //         gtrr $tmp $num $tmp
                //         addr $tmp $ip $ip
                //         addi $ip #1 $ip
                //         seti #end $ip
                //         addi $quo #1 $quo
                //  end:   seti #start $ip
                #[rustfmt::skip]
                if let [
                    Instruction::Seti(0, quo),
                    Instruction::Addi(quo2, 1, tmp),
                    Instruction::Muli(tmp2, 256, tmp3),
                    Instruction::Gtrr(tmp4, num, tmp5),
                    Instruction::Addr(tmp6, ip, ip2),
                    Instruction::Addi(ip3, 1, ip4),
                    Instruction::Seti(end, ip5),
                    Instruction::Addi(quo3, 1, quo4),
                    Instruction::Seti(start, ip6),
                    ..,
                ] = instructions[addr..]
                    && quo == quo2 && quo == quo3 && quo == quo4
                    && tmp == tmp2 && tmp == tmp3 && tmp == tmp4 && tmp == tmp5 && tmp == tmp6
                    && ip == ip2 && ip == ip3 && ip == ip4 && ip == ip5 && ip == ip6
                    && start as usize == addr
                    && end as usize == addr + 8
                {
                    reg[quo] = reg[num] >> 8;
                    reg[tmp] = 1;
                    reg[ip] += 9;
                    return HookControlFlow::Next;
                };

                HookControlFlow::Execute
            });

        target.expect("no solution found")
    }
}

examples!(Day21 -> (u32, u32) [
    // Custom example
    {
        // Equivalent to
        //
        //  r2 = 0;
        //  do {
        //    r2++;
        //    if (r2 == 100) r2 = 1;
        //  } while (r2 != r0);
        //
        // Produces sequence 1, 2, 3, ..., 98, 99, 1, 2, 3, ..., 98, 99, 1, ...
        input: "#ip 4\n\
            seti 0 0 2\n\
            addi 2 1 2\n\
            eqri 2 100 1\n\
            addr 1 4 4\n\
            addi 4 1 4\n\
            seti 1 0 2\n\
            eqrr 2 0 1\n\
            addr 1 4 4\n\
            seti 0 0 4",
        part1: 1,
        part2: 99,
    },
]);
