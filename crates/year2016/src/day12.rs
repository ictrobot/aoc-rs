use utils::prelude::*;

/// Interpreting assembly, again.
///
/// The key optimization is that
/// ```text
/// inc $r1
/// dec $r2
/// jnz $r2 -2
/// ```
/// can be replaced with `$r1 += $r2` followed by `$r2 = 0`. This reduces the number of simulated
/// cycles ~5,000 times for part 1 and ~100,000 times for part 2, to around ~200 cycles each.
#[derive(Clone, Debug)]
pub struct Day12 {
    instructions: Vec<Instruction>,
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
enum Register {
    A,
    B,
    C,
    D,
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
enum Value {
    Register(Register),
    Number(i32),
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
enum Instruction {
    Copy(Value, Register),
    Increment(Register),
    Decrement(Register),
    JumpIfNotZero(Value, i32),
}

impl Day12 {
    pub fn new(input: &str, _: InputType) -> Result<Self, InputError> {
        let register = b'a'
            .map(|_| Register::A)
            .or(b'b'.map(|_| Register::B))
            .or(b'c'.map(|_| Register::C))
            .or(b'd'.map(|_| Register::D));
        let value = register
            .map(Value::Register)
            .or(parser::i32().map(Value::Number));

        Ok(Self {
            instructions: register
                .with_prefix("inc ")
                .map(Instruction::Increment)
                .or(register.with_prefix("dec ").map(Instruction::Decrement))
                .or(value
                    .with_prefix("cpy ")
                    .then(register.with_prefix(" "))
                    .map(|(v, r)| Instruction::Copy(v, r)))
                .or(value
                    .with_prefix("jnz ")
                    .then(parser::i32().with_prefix(" "))
                    .map(|(v, o)| Instruction::JumpIfNotZero(v, o)))
                .parse_lines(input)?,
        })
    }

    #[must_use]
    pub fn part1(&self) -> i32 {
        self.execute([0; 4])
    }

    #[must_use]
    pub fn part2(&self) -> i32 {
        self.execute([0, 0, 1, 0])
    }

    fn execute(&self, mut reg: [i32; 4]) -> i32 {
        let mut pc = 0;

        while pc < self.instructions.len() {
            // Recognize the following pattern of instructions which can be simplified to addition
            //  inc $r1
            //  dec $r2
            //  jnz $r2 -2
            if let [Instruction::Increment(r1), Instruction::Decrement(r2), Instruction::JumpIfNotZero(Value::Register(r3), -2), ..] =
                self.instructions[pc..]
            {
                if r2 == r3 {
                    reg[r1 as usize] += reg[r2 as usize];
                    reg[r2 as usize] = 0;
                    pc += 3;
                    continue;
                }
            }

            match self.instructions[pc] {
                Instruction::Copy(v, dst) => reg[dst as usize] = v.get(&reg),
                Instruction::Increment(dst) => reg[dst as usize] += 1,
                Instruction::Decrement(dst) => reg[dst as usize] -= 1,
                Instruction::JumpIfNotZero(v, offset) => {
                    if v.get(&reg) != 0 {
                        let Some(new_pc) = pc.checked_add_signed(offset as isize) else {
                            break;
                        };
                        pc = new_pc;
                        continue;
                    }
                }
            }

            pc += 1;
        }

        reg[0]
    }
}

impl Value {
    #[inline]
    fn get(self, registers: &[i32; 4]) -> i32 {
        match self {
            Value::Register(r) => registers[r as usize],
            Value::Number(n) => n,
        }
    }
}

examples!(Day12 -> (i32, i32) [
    {
        input: "cpy 41 a\n\
            inc a\n\
            inc a\n\
            dec a\n\
            jnz a 2\n\
            dec a",
        part1: 42
    },
]);
