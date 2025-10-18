use crate::elfcode::{HookControlFlow, Instruction, Interpreter};
use utils::number::sum_of_divisors;
use utils::prelude::*;

/// Interpreting assembly to calculate the sum of divisors.
///
/// See also [day 16](crate::Day16) and [day 21](crate::Day21).
#[derive(Clone, Debug)]
pub struct Day19 {
    interpreter: Interpreter,
}

impl Day19 {
    pub fn new(input: &str, _: InputType) -> Result<Self, InputError> {
        Ok(Self {
            interpreter: Interpreter::new(input)?,
        })
    }

    #[must_use]
    pub fn part1(&self) -> u32 {
        self.run(0)
    }

    #[must_use]
    pub fn part2(&self) -> u32 {
        self.run(1)
    }

    fn run(&self, reg0: u32) -> u32 {
        let mut reg = [reg0, 0, 0, 0, 0, 0];

        self.interpreter.run(&mut reg, |instructions, instruction_pointer, reg| {
            let addr = reg[instruction_pointer] as usize;

            // Recognize the naive sum of divisors loop and replace it with a native implementation.
            //  loop0: seti #1 $div
            //  loop1: seti #1 $mul
            //         mulr $div $mul $tmp
            //         eqrr $tmp $tgt $tmp
            //         addr $tmp $ip $ip
            //         addi $ip #1 $ip
            //         addr $div $sum $sum
            //         addi $mul #1 $mul
            //         gtrr $mul $tgt $tmp
            //         addr $ip $tmp $ip
            //         seti #loop1 $ip
            //         addi $div #1 $div
            //         gtrr $div $tgt $tmp
            //         addr $tmp $ip $ip
            //         seti #loop0 $ip
            #[rustfmt::skip]
            if let [
                Instruction::Seti(1, div),
                Instruction::Seti(1, mul),
                Instruction::Mulr(div2, mul2, tmp),
                Instruction::Eqrr(tmp2, tgt, tmp3),
                Instruction::Addr(tmp4, ip, ip2),
                Instruction::Addi(ip3, 1, ip4),
                Instruction::Addr(div3, sum, sum2),
                Instruction::Addi(mul3, 1, mul4),
                Instruction::Gtrr(mul5, tgt2, tmp5),
                Instruction::Addr(ip5, tmp6, ip6),
                Instruction::Seti(loop1, ip7),
                Instruction::Addi(div4, 1, div5),
                Instruction::Gtrr(div6, tgt3, tmp7),
                Instruction::Addr(tmp8, ip8, ip9),
                Instruction::Seti(loop0, ip10),
                ..,
            ] = instructions[addr..]
                && div == div2 && div == div3 && div == div4 && div == div5 && div == div6
                && mul == mul2 && mul == mul3 && mul == mul4 && mul == mul5
                && tmp == tmp2 && tmp == tmp3 && tmp == tmp4 && tmp == tmp5 && tmp == tmp6 && tmp == tmp7 && tmp == tmp8
                && tgt == tgt2 && tgt == tgt3
                && ip == ip2 && ip == ip3 && ip == ip4 && ip == ip5 && ip == ip6 && ip == ip7 && ip == ip8 && ip == ip9 && ip == ip10
                && sum == sum2
                && ip == instruction_pointer
                && loop0 as usize == addr
                && loop1 as usize == addr + 1
            {
                reg[sum] += sum_of_divisors(reg[tgt])
                    .expect("the target's sum of divisors should fit within a u32");
                reg[div] = reg[tgt] + 1;
                reg[mul] = reg[tgt] + 1;
                reg[tmp] = 1;
                reg[ip] += 15;
                return HookControlFlow::Next
            };

            HookControlFlow::Execute
        });

        reg[0]
    }
}

examples!(Day19 -> (u32, u32) [
    {
        input: "#ip 0\n\
            seti 5 0 1\n\
            seti 6 0 2\n\
            addi 0 1 0\n\
            addr 1 2 3\n\
            setr 1 0 0\n\
            seti 8 0 4\n\
            seti 9 0 5",
        part1: 7,
    },
]);
