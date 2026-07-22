use utils::prelude::*;

/// Decoding binary numbers.
#[derive(Clone, Debug)]
pub struct Day05 {
    seats: [bool; 1024],
    max: u16,
}

impl Day05 {
    pub fn new(input: &str, _: InputType) -> Result<Self, InputError> {
        let row =
            parser::byte_map!(b'F' => 0, b'B' => 1)
                .repeat_fold(parser::noop(), 7, 0, |acc, b| (acc << 1) | b);
        let col =
            parser::byte_map!(b'L' => 0, b'R' => 1)
                .repeat_fold(parser::noop(), 3, 0, |acc, b| (acc << 1) | b);
        let pass = row
            .then(col)
            .map(|(row, col)| (row << 3) | col)
            .with_consumed()
            .with_eol();

        let mut seats = [false; 1024];
        let mut max = 0;

        for pass in pass.parse_iterator(input) {
            let (seat, pass) = pass?;
            if pass.len() != 10 {
                return Err(InputError::new(
                    input,
                    pass,
                    "expected 10-character boarding pass",
                ));
            }

            if seats[seat as usize] {
                return Err(InputError::new(input, pass, "duplicate boarding pass"));
            }
            seats[seat as usize] = true;
            max = max.max(seat);
        }

        Ok(Self { seats, max })
    }

    #[must_use]
    pub fn part1(&self) -> u16 {
        self.max
    }

    #[must_use]
    pub fn part2(&self) -> u16 {
        self.seats
            .array_windows()
            .position(|&[a, b, c]| a && !b && c)
            .map(|seat| (seat + 1) as u16)
            .expect("no solution found")
    }
}

examples!(Day05 -> (u16, u16) [
    {input: "FBFBBFFRLR", part1: 357},
    {input: "BFFFBBFRRR", part1: 567},
    {input: "FFFBBBFRRR", part1: 119},
    {input: "BBFFBBFRLL", part1: 820},
    {input: "BFFFBBFRRR\nFFFBBBFRRR\nBBFFBBFRLL", part1: 820},
]);
