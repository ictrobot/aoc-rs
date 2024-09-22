use utils::prelude::*;

/// Unscrambling passwords.
#[derive(Clone, Debug)]
pub struct Day21 {
    input_type: InputType,
    operations: Vec<Operation>,
}

#[derive(Copy, Clone, Debug)]
enum Operation {
    SwapPosition(u32, u32),
    SwapLetter(u8, u8),
    RotateLeft(u32),
    RotateRight(u32),
    RotateLetter(u8),
    RotateLetterReverse(u8),
    Reverse(u32, u32),
    Move(u32, u32),
}

impl Day21 {
    pub fn new(input: &str, input_type: InputType) -> Result<Self, InputError> {
        let (position, letter) = match input_type {
            InputType::Example => (parser::number_range(0..=4), parser::byte_range(b'a'..=b'e')),
            InputType::Real => (parser::number_range(0..=7), parser::byte_range(b'a'..=b'h')),
        };

        Ok(Self {
            input_type,
            operations: parser::one_of((
                position
                    .with_prefix("swap position ")
                    .then(position.with_prefix(" with position "))
                    .map(|(from, to)| Operation::SwapPosition(from, to)),
                letter
                    .with_prefix("swap letter ")
                    .then(letter.with_prefix(" with letter "))
                    .map(|(a, b)| Operation::SwapLetter(a, b)),
                position
                    .with_prefix("rotate left ")
                    .with_suffix(" step".then("s".optional()))
                    .map(Operation::RotateLeft),
                position
                    .with_prefix("rotate right ")
                    .with_suffix(" step".then("s".optional()))
                    .map(Operation::RotateRight),
                letter
                    .with_prefix("rotate based on position of letter ")
                    .map(Operation::RotateLetter),
                position
                    .with_prefix("reverse positions ")
                    .then(position.with_prefix(" through "))
                    .map(|(start, end)| Operation::Reverse(start, end)),
                position
                    .with_prefix("move position ")
                    .then(position.with_prefix(" to position "))
                    .map(|(from, to)| Operation::Move(from, to)),
            ))
            .parse_lines(input)?,
        })
    }

    #[must_use]
    pub fn part1(&self) -> String {
        let operations = self.operations.iter().copied();
        let scrambled = match self.input_type {
            InputType::Example => Scrambler::scramble(operations, *b"abcde").to_vec(),
            InputType::Real => Scrambler::scramble(operations, *b"abcdefgh").to_vec(),
        };
        String::from_utf8(scrambled).unwrap()
    }

    #[must_use]
    pub fn part2(&self) -> String {
        let operations = self.operations.iter().rev().map(|&op| match op {
            Operation::RotateLeft(shift) => Operation::RotateRight(shift),
            Operation::RotateRight(shift) => Operation::RotateLeft(shift),
            Operation::RotateLetter(shift) => Operation::RotateLetterReverse(shift),
            Operation::Move(from, to) => Operation::Move(to, from),
            _ => op,
        });

        let scrambled = match self.input_type {
            InputType::Example => panic!("no part 2 example"),
            InputType::Real => Scrambler::scramble(operations, *b"fbgdceah").to_vec(),
        };
        String::from_utf8(scrambled).unwrap()
    }
}

struct Scrambler<const N: usize>;
impl<const N: usize> Scrambler<N> {
    const REVERSE_LETTER_ROTATIONS: Option<[usize; N]> = Self::reverse_letter_rotations();

    fn scramble(operations: impl Iterator<Item = Operation>, mut password: [u8; N]) -> [u8; N] {
        for operation in operations {
            match operation {
                Operation::SwapPosition(from, to) => {
                    password.swap(from as usize, to as usize);
                }
                Operation::SwapLetter(a, b) => {
                    let x = password.iter().position(|&c| c == a).unwrap();
                    let y = password.iter().position(|&c| c == b).unwrap();
                    password.swap(x, y);
                }
                Operation::RotateLeft(steps) => {
                    password.rotate_left(steps as usize);
                }
                Operation::RotateRight(steps) => {
                    password.rotate_right(steps as usize);
                }
                Operation::RotateLetter(letter) => {
                    let i = password.iter().position(|&c| c == letter).unwrap();
                    password.rotate_right(Self::rotate_letter_amount(i) % N);
                }
                Operation::RotateLetterReverse(letter) => {
                    let reverse_rotations = Self::REVERSE_LETTER_ROTATIONS.unwrap_or_else(|| {
                        panic!("letter rotation for {N} long passwords isn't unique")
                    });
                    let i = password.iter().position(|&c| c == letter).unwrap();
                    password.rotate_left(reverse_rotations[i]);
                }
                Operation::Reverse(start, end) => {
                    password[start as usize..=end as usize].reverse();
                }
                Operation::Move(from, to) => {
                    let v = password[from as usize];
                    if from < to {
                        password.copy_within(from as usize + 1..=to as usize, from as usize);
                    } else {
                        password.copy_within(to as usize..from as usize, to as usize + 1);
                    }
                    password[to as usize] = v;
                }
            }
        }

        password
    }

    const fn rotate_letter_amount(i: usize) -> usize {
        if i < 4 {
            i + 1
        } else {
            i + 2
        }
    }

    const fn reverse_letter_rotations() -> Option<[usize; N]> {
        let mut reverse = [usize::MAX; N];
        let mut i = 0;
        while i < N {
            let dest = (i + Self::rotate_letter_amount(i)) % N;
            if reverse[dest] != usize::MAX {
                return None;
            }
            reverse[dest] = (N + dest - i) % N;
            i += 1;
        }
        Some(reverse)
    }
}

examples!(Day21 -> (&'static str, &'static str) [
    {file: "day21_example0.txt", part1: "decab"},
]);
