use utils::prelude::*;

/// Parsing and validating records.
#[derive(Clone, Debug)]
pub struct Day04 {
    part1: usize,
    part2: usize,
}

const BIRTH_YEAR: u8 = 0;
const ISSUE_YEAR: u8 = 1;
const EXPIRATION_YEAR: u8 = 2;
const HEIGHT: u8 = 3;
const HAIR_COLOR: u8 = 4;
const EYE_COLOR: u8 = 5;
const PASSPORT_ID: u8 = 6;
const COUNTRY_ID: u8 = 7;
const REQUIRED_FIELDS: u8 = 0b0111_1111;

impl Day04 {
    pub fn new(input: &str, _: InputType) -> Result<Self, InputError> {
        let key = parser::literal_map!(
            "byr:" => BIRTH_YEAR,
            "iyr:" => ISSUE_YEAR,
            "eyr:" => EXPIRATION_YEAR,
            "hgt:" => HEIGHT,
            "hcl:" => HAIR_COLOR,
            "ecl:" => EYE_COLOR,
            "pid:" => PASSPORT_ID,
            "cid:" => COUNTRY_ID,
        );
        let value = parser::take_while1(u8::is_ascii_graphic);
        let kv = parser::parse_tree!(
            (k @ key, v @ value) =>> {
                (b' ') => (k, v, false),
                (parser::eol()) =>> {
                    (parser::eol()) => (k, v, true),
                    (parser::noop()) => (k, v, false),
                },
            },
        );

        let (mut part1, mut part2) = (0, 0);
        let (mut fields, mut valid) = (0, true);
        for entry in kv.parse_iterator(input) {
            let (field, value, end) = entry?;

            let bit = 1 << field;
            if fields & bit != 0 {
                return Err(InputError::new(input, value, "duplicate passport field"));
            }
            fields |= bit;
            valid &= Self::valid_value(field, value);

            if end {
                let complete = fields & REQUIRED_FIELDS == REQUIRED_FIELDS;
                part1 += usize::from(complete);
                part2 += usize::from(complete & valid);
                (fields, valid) = (0, true);
            }
        }

        if fields != 0 {
            return Err(InputError::new(
                input,
                input.len(),
                "expected passport field after space",
            ));
        }

        Ok(Self { part1, part2 })
    }

    #[inline]
    fn valid_value(index: u8, value: &[u8]) -> bool {
        match index {
            BIRTH_YEAR => Self::num_range(value, b"1920", b"2002"),
            ISSUE_YEAR => Self::num_range(value, b"2010", b"2020"),
            EXPIRATION_YEAR => Self::num_range(value, b"2020", b"2030"),
            HEIGHT => match value {
                [value @ .., b'c', b'm'] => Self::num_range(value, b"150", b"193"),
                [value @ .., b'i', b'n'] => Self::num_range(value, b"59", b"76"),
                _ => false,
            },
            HAIR_COLOR => {
                value.len() == 7
                    && value[0] == b'#'
                    && value[1..]
                        .iter()
                        .all(|b| matches!(b, b'0'..=b'9' | b'a'..=b'f'))
            }
            EYE_COLOR => {
                matches!(
                    value,
                    b"amb" | b"blu" | b"brn" | b"gry" | b"grn" | b"hzl" | b"oth"
                )
            }
            PASSPORT_ID => value.len() == 9 && value.iter().all(u8::is_ascii_digit),
            COUNTRY_ID => true,
            _ => unreachable!(),
        }
    }

    #[inline]
    fn num_range<const N: usize>(value: &[u8], min: &[u8; N], max: &[u8; N]) -> bool {
        value.len() == N && value.iter().all(u8::is_ascii_digit) && value >= min && value <= max
    }

    #[must_use]
    pub fn part1(&self) -> usize {
        self.part1
    }

    #[must_use]
    pub fn part2(&self) -> usize {
        self.part2
    }
}

examples!(Day04 -> (usize, usize) [
    {file: "day04_example0.txt", part1: 2},
    {file: "day04_example1.txt", part2: 0},
    {file: "day04_example2.txt", part2: 4},
]);
