use utils::prelude::*;

/// Parsing numbers from columns of digits.
#[derive(Clone, Debug)]
pub struct Day06 {
    part1: u64,
    part2: u64,
}

const MAX_NUMBERS: usize = 4;
const MAX_NUMBER_DIGITS: usize = 4;

impl Day06 {
    pub fn new(input: &str, _: InputType) -> Result<Self, InputError> {
        let mut lines = input.lines().map(str::as_bytes).collect::<Vec<_>>();
        if lines.len() > MAX_NUMBERS + 1 {
            return Err(InputError::new(
                input,
                0,
                "expected at most four lines of numbers followed by one line of operators",
            ));
        }
        if lines.len() < 3 {
            return Err(InputError::new(
                input,
                0,
                "expected at least two lines of numbers followed by one line of operators",
            ));
        }

        let operators = lines.pop().unwrap();
        if let Some(&l) = lines.iter().find(|l| l.len() != operators.len()) {
            return Err(InputError::new(
                input,
                l,
                "expected all number lines to match operator line length",
            ));
        }

        let mut index = 0;
        let (mut part1, mut part2) = (0, 0);
        while index < operators.len() {
            let operator: fn(&[u64]) -> u64 = match operators[index] {
                b'+' => |x| x.iter().sum(),
                b'*' => |x| x.iter().product(),
                _ => return Err(InputError::new(input, operators, "expected '+' or '*'")),
            };

            let mut column_width = 1;
            while index + column_width < operators.len() && operators[index + column_width] == b' '
            {
                column_width += 1;
            }
            let number_width = if index + column_width == operators.len() {
                // The final column has no space between it and the next column
                column_width
            } else {
                column_width - 1
            };
            if number_width > MAX_NUMBER_DIGITS {
                return Err(InputError::new(
                    input,
                    operators,
                    "too many digits in column",
                ));
            }

            let mut normal_numbers = [0; MAX_NUMBERS];
            let mut cephalopod_numbers = [0; MAX_NUMBER_DIGITS];
            for (i, &l) in lines.iter().enumerate() {
                let mut start = index;
                while start < l.len() && l[start] == b' ' {
                    start += 1;
                }
                let mut end = index + number_width - 1;
                while end > start && l[end] == b' ' {
                    end -= 1;
                }

                let mut normal_number = 0;
                for i in start..=end {
                    if !matches!(l[i], b'1'..=b'9') {
                        return Err(InputError::new(input, &l[i..], "expected '1'-'9' or ' '"));
                    }
                    normal_number = (normal_number * 10) + (l[i] - b'0') as u64;
                }
                normal_numbers[i] = normal_number;

                for (j, &b) in l[index..index + number_width].iter().enumerate() {
                    if b != b' ' {
                        // Normal number loop has already checked that any non-space bytes are '1'-'9'
                        cephalopod_numbers[j] = (cephalopod_numbers[j] * 10) + (b - b'0') as u64;
                    }
                }

                if index + number_width < l.len() && l[index + number_width] != b' ' {
                    return Err(InputError::new(
                        input,
                        &l[index + number_width..],
                        "expected ' ' between columns",
                    ));
                }
            }

            part1 += operator(&normal_numbers[..lines.len()]);
            part2 += operator(&cephalopod_numbers[..number_width]);
            index += column_width;
        }

        Ok(Self { part1, part2 })
    }

    #[must_use]
    pub fn part1(&self) -> u64 {
        self.part1
    }

    #[must_use]
    pub fn part2(&self) -> u64 {
        self.part2
    }
}

examples!(Day06 -> (u64, u64) [
    {
        input: "123 328  51 64 \n 45 64  387 23 \n  6 98  215 314\n*   +   *   +  ",
        part1: 4277556,
        part2: 3263827,
    },
]);
