use utils::prelude::*;

/// Validating records and inferring column order.
///
/// Assumes there is always a remaining column with only one possible field.
/// See also [2018 Day 16](../year2018/struct.Day16.html).
#[derive(Clone, Debug)]
pub struct Day16 {
    part1: u64,
    part2: u64,
}

const MAX_FIELDS: usize = 20;
const MAX_VALUE: u16 = 999;

impl Day16 {
    pub fn new(input: &str, _: InputType) -> Result<Self, InputError> {
        let number = parser::number_range(0..=MAX_VALUE);
        let range = number
            .then(number.with_prefix(b'-'))
            .map_res(|(start, end)| {
                (start <= end)
                    .then_some((start, end))
                    .ok_or("range end cannot be less than start")
            });
        let name = parser::take_while1(u8::is_ascii_lowercase)
            .repeat_fold(b' ', 1, (), |(), _| ())
            .with_consumed()
            .map(|(_, name)| name)
            .with_suffix(": ");
        let rule = name
            .then(range.repeat_n::<2, _>(" or "))
            .map_res(|(name, ranges)| {
                (ranges[0].1 < ranges[1].0)
                    .then_some((name, ranges))
                    .ok_or("ranges must be sorted and disjoint")
            });
        let ticket = number
            .repeat_arrayvec::<MAX_FIELDS, _>(b',', 1)
            .with_consumed();

        let (rules, your_ticket, nearby_tickets) = rule
            .repeat_arrayvec::<MAX_FIELDS, _>(parser::eol(), 1)
            .with_eol()
            .with_eol()
            .then(
                ticket
                    .with_prefix("your ticket:".with_eol())
                    .with_eol()
                    .with_eol(),
            )
            .then(
                ticket
                    .repeat(parser::eol(), 1)
                    .with_prefix("nearby tickets:".with_eol()),
            )
            .parse_complete(input)?;

        for (index, &(name, _)) in rules.iter().enumerate() {
            if rules[..index].iter().any(|(other, _)| *other == name) {
                return Err(InputError::new(input, name, "duplicate field name"));
            }
        }
        for (ticket, text) in nearby_tickets.iter().chain([&your_ticket]) {
            if ticket.len() != rules.len() {
                return Err(InputError::new(
                    input,
                    *text,
                    "ticket field count does not match rule count",
                ));
            }
        }

        let all_fields = (1u32 << rules.len()) - 1;
        let departure_fields = rules
            .iter()
            .enumerate()
            .filter(|(_, (name, _))| name.starts_with(b"departure"))
            .fold(0, |mask, (field, _)| mask | 1 << field);

        // Calculate a mask of the valid fields for each value, using each range's start and end + 1
        // to toggle the field's bit, followed by XORing the changes with the previous mask in a
        // loop to give the set of valid fields at each value.
        let mut valid_fields = [0u32; MAX_VALUE as usize + 2];
        for (field, (_, ranges)) in rules.iter().enumerate() {
            for &(start, end) in ranges {
                valid_fields[usize::from(start)] ^= 1 << field;
                valid_fields[usize::from(end) + 1] ^= 1 << field;
            }
        }
        let mut fields = 0;
        for change in &mut valid_fields {
            fields ^= *change;
            *change = fields;
        }

        let (your_ticket, your_text) = your_ticket;
        if your_ticket
            .iter()
            .any(|&value| valid_fields[usize::from(value)] == 0)
        {
            return Err(InputError::new(
                input,
                your_text,
                "your ticket contains an invalid value",
            ));
        }

        let mut part1 = 0;
        let mut candidates = [all_fields; MAX_FIELDS];
        let mut column_fields = [0; MAX_FIELDS];
        for (ticket, _) in nearby_tickets {
            let mut valid_ticket = true;
            for (column, &value) in ticket.iter().enumerate() {
                column_fields[column] = valid_fields[usize::from(value)];
                if column_fields[column] == 0 {
                    part1 += u64::from(value);
                    valid_ticket = false;
                }
            }

            if valid_ticket {
                for column in 0..rules.len() {
                    candidates[column] &= column_fields[column];
                }
            }
        }

        let mut part2 = 1u64;
        for _ in 0..rules.len() {
            let Some(column) = candidates[..rules.len()]
                .iter()
                .position(|fields| fields.count_ones() == 1)
            else {
                return Err(InputError::new(
                    input,
                    0,
                    "expected unique field assignment",
                ));
            };

            let field = candidates[column];
            candidates[..rules.len()]
                .iter_mut()
                .for_each(|fields| *fields &= !field);

            if field & departure_fields != 0 {
                part2 *= u64::from(your_ticket[column]);
            }
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

examples!(Day16 -> (u64, u64) [
    {file: "day16_example0.txt", part1: 71},
]);
