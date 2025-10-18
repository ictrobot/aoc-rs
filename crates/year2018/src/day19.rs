use utils::number::sum_of_divisors;
use utils::prelude::*;

/// Interpreting assembly to calculate the sum of divisors.
///
/// See also [day 16](crate::Day16), which uses the same instruction set.
#[derive(Clone, Debug)]
pub struct Day19 {
    instruction_pointer: Register,
    instructions: Vec<Instruction>,
}

// Avoids bounds checks when indexing the register array
utils::enumerable_enum! {
    #[repr(u32)]
    #[derive(Copy, Clone, Debug, PartialEq)]
    enum Register {
        A,
        B,
        C,
        D,
        E,
        F,
    }
}

#[derive(Copy, Clone, Debug)]
enum Instruction {
    Addr(Register, Register, Register),
    Addi(Register, u32, Register),
    Mulr(Register, Register, Register),
    Muli(Register, u32, Register),
    Banr(Register, Register, Register),
    Bani(Register, u32, Register),
    Borr(Register, Register, Register),
    Bori(Register, u32, Register),
    Setr(Register, Register),
    Seti(u32, Register),
    Gtir(u32, Register, Register),
    Gtri(Register, u32, Register),
    Gtrr(Register, Register, Register),
    Eqir(u32, Register, Register),
    Eqri(Register, u32, Register),
    Eqrr(Register, Register, Register),
}

impl Day19 {
    pub fn new(input: &str, _: InputType) -> Result<Self, InputError> {
        let register =
            parser::byte_range(b'0'..=b'5').map(|b| Register::from_discriminant((b - b'0') as u32));
        let rrr_instructions = parser::literal_map!(
            "addr " => Instruction::Addr as fn(_, _, _) -> _,
            "mulr " => Instruction::Mulr,
            "banr " => Instruction::Banr,
            "borr " => Instruction::Borr,
            "gtrr " => Instruction::Gtrr,
            "eqrr " => Instruction::Eqrr,
        );
        let rir_instructions = parser::literal_map!(
            "addi " => Instruction::Addi as fn(_, _, _) -> _,
            "muli " => Instruction::Muli,
            "bani " => Instruction::Bani,
            "bori " => Instruction::Bori,
            "gtri " => Instruction::Gtri,
            "eqri " => Instruction::Eqri,
        );
        let instruction = parser::parse_tree!(
            (i @ rrr_instructions, a @ register, b' ', b @ register, b' ', c @ register) => i(a, b, c),
            (i @ rir_instructions, a @ register, b' ', b @ parser::u32(), b' ', c @ register) => i(a, b, c),
            ("setr ", a @ register, b' ', parser::u32(), b' ', c @ register) => Instruction::Setr(a, c),
            ("seti ", a @ parser::u32(), b' ', parser::u32(), b' ', c @ register) => Instruction::Seti(a, c),
            ("gtir ", a @ parser::u32(), b' ', b @ register, b' ', c @ register) => Instruction::Gtir(a, b, c),
            ("eqir ", a @ parser::u32(), b' ', b @ register, b' ', c @ register) => Instruction::Eqir(a, b, c),
        );

        let (instruction_pointer, instructions) = register
            .with_prefix("#ip ")
            .with_suffix(parser::eol())
            .then(instruction.repeat(parser::eol(), 1))
            .parse_complete(input)?;

        Ok(Self {
            instruction_pointer,
            instructions,
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
        while let addr = reg[self.instruction_pointer] as usize
            && addr < self.instructions.len()
        {
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
            //         addi $div $1 $div
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
            ] = self.instructions[addr..]
                && div == div2 && div == div3 && div == div4 && div == div5 && div == div6
                && mul == mul2 && mul == mul3 && mul == mul4 && mul == mul5
                && tmp == tmp2 && tmp == tmp3 && tmp == tmp4 && tmp == tmp5 && tmp == tmp6 && tmp == tmp7 && tmp == tmp8
                && tgt == tgt2 && tgt == tgt3
                && ip == ip2 && ip == ip3 && ip == ip4 && ip == ip5 && ip == ip6 && ip == ip7 && ip == ip8 && ip == ip9 && ip == ip10
                && sum == sum2
                && ip == self.instruction_pointer
                && loop0 as usize == addr
                && loop1 as usize == addr + 1
            {
                reg[sum] += sum_of_divisors(reg[tgt])
                    .expect("the target's sum of divisors should fit within a u32");
                reg[div] = reg[tgt] + 1;
                reg[mul] = reg[tgt] + 1;
                reg[tmp] = 1;
                reg[ip] += 15;
                continue;
            };

            match self.instructions[addr] {
                Instruction::Addr(a, b, c) => reg[c] = reg[a] + reg[b],
                Instruction::Addi(a, b, c) => reg[c] = reg[a] + b,
                Instruction::Mulr(a, b, c) => reg[c] = reg[a] * reg[b],
                Instruction::Muli(a, b, c) => reg[c] = reg[a] * b,
                Instruction::Banr(a, b, c) => reg[c] = reg[a] & reg[b],
                Instruction::Bani(a, b, c) => reg[c] = reg[a] & b,
                Instruction::Borr(a, b, c) => reg[c] = reg[a] | reg[b],
                Instruction::Bori(a, b, c) => reg[c] = reg[a] | b,
                Instruction::Setr(a, c) => reg[c] = reg[a],
                Instruction::Seti(a, c) => reg[c] = a,
                Instruction::Gtir(a, b, c) => reg[c] = u32::from(a > reg[b]),
                Instruction::Gtri(a, b, c) => reg[c] = u32::from(reg[a] > b),
                Instruction::Gtrr(a, b, c) => reg[c] = u32::from(reg[a] > reg[b]),
                Instruction::Eqir(a, b, c) => reg[c] = u32::from(a == reg[b]),
                Instruction::Eqri(a, b, c) => reg[c] = u32::from(reg[a] == b),
                Instruction::Eqrr(a, b, c) => reg[c] = u32::from(reg[a] == reg[b]),
            }

            reg[self.instruction_pointer] += 1;
        }

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
