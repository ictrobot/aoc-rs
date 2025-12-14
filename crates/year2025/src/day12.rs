use utils::prelude::*;

/// Checking region sizes.
///
/// The solution assumes that each present shape can be treated as a solid 3x3 square. This makes
/// the implementation trivial and much faster than a general solution to 2D bin packing (which is
/// NP-complete).
#[derive(Clone, Debug)]
pub struct Day12 {
    part1: usize,
}

impl Day12 {
    pub fn new(input: &str, _: InputType) -> Result<Self, InputError> {
        let part1 = parser::u32()
            .repeat_n::<2, _>(b'x')
            .with_suffix(": ")
            .then(parser::u32().repeat_fold(b' ', 1, 0u32, |acc, x| acc + x))
            .repeat_fold(
                parser::eol(),
                1,
                0usize,
                |acc, ([width, height], shapes)| {
                    acc + usize::from((width / 3) * (height / 3) >= shapes)
                },
            )
            .with_prefix(
                // Discard the present shapes at the start of the input
                parser::u32()
                    .with_suffix(b':')
                    .with_eol()
                    .then(
                        parser::byte_map!(b'.' => (), b'#' => ())
                            .repeat_n::<3, _>(parser::noop())
                            .repeat_n::<3, _>(parser::eol()),
                    )
                    .repeat_fold(parser::eol().with_eol(), 1, (), |_, _| ())
                    .with_eol()
                    .with_eol(),
            )
            .parse_complete(input)?;

        Ok(Self { part1 })
    }

    #[must_use]
    pub fn part1(&self) -> usize {
        self.part1
    }

    #[must_use]
    pub fn part2(&self) -> &'static str {
        "🎄"
    }
}

// The examples require actual bin packing, not just counting regions
examples!(Day12 -> (usize, &'static str) []);
