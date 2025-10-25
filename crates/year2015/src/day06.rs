use utils::prelude::*;

/// Light grid.
///
/// Reducing the 1000x1000 grid based on the unique x and y bounds in the input gives a ~4x speedup.
/// For example, the 1000x1000 grid for this input:
///
/// ```text
/// turn on 0,0 through 999,999
/// toggle 0,0 through 999,0
/// turn off 499,499 through 500,500
/// ```
///
/// Can be converted into this 3x4 grid:
///
/// ```text
/// | 0,0   through 498,0   | 499,0   through 500,0   | 501,0   through 999,0   |
/// | 0,1   through 498,498 | 499,1   through 500,498 | 501,1   through 999,498 |
/// | 0,499 through 498,500 | 499,499 through 500,500 | 501,499 through 999,500 |
/// | 0,501 through 498,999 | 499,501 through 500,999 | 501,501 through 999,999 |
/// ```
#[derive(Clone, Debug)]
pub struct Day06 {
    instructions: Vec<(Action, (u16, u16, u16, u16))>,
    row_widths: Vec<u16>,
    col_heights: Vec<u16>,
}

#[derive(Clone, Copy, Debug)]
enum Action {
    TurnOn,
    TurnOff,
    Toggle,
}

impl Day06 {
    pub fn new(input: &str, _: InputType) -> Result<Self, InputError> {
        let mut instructions = parser::literal_map!(
            "turn off " => Action::TurnOff,
            "turn on " => Action::TurnOn,
            "toggle " => Action::Toggle,
        )
        .then(parser::u16())
        .then(parser::u16().with_prefix(b','))
        .then(parser::u16().with_prefix(" through "))
        .then(parser::u16().with_prefix(b','))
        .map(|(action, x1, y1, x2, y2)| (action, (x1.min(x2), y1.min(y2), x1.max(x2), y1.max(y2))))
        .parse_lines(input)?;

        let mut x_values = Vec::with_capacity(instructions.len() * 2);
        let mut y_values = Vec::with_capacity(instructions.len() * 2);
        instructions.iter().for_each(|&(_, (x1, y1, x2, y2))| {
            x_values.push(x1);
            x_values.push(x2 + 1);
            y_values.push(y1);
            y_values.push(y2 + 1);
        });
        x_values.sort_unstable();
        x_values.dedup();
        y_values.sort_unstable();
        y_values.dedup();

        let row_widths: Vec<u16> = x_values.windows(2).map(|w| w[1] - w[0]).collect();
        let col_heights: Vec<u16> = y_values.windows(2).map(|w| w[1] - w[0]).collect();

        let mut x_map = [0; 1000];
        for (i, w) in x_values.windows(2).enumerate() {
            x_map[w[0] as usize..w[1] as usize].fill(i as u16)
        }
        let mut y_map = [0; 1000];
        for (i, w) in y_values.windows(2).enumerate() {
            y_map[w[0] as usize..w[1] as usize].fill(i as u16)
        }

        instructions.iter_mut().for_each(|(_, (x1, y1, x2, y2))| {
            *x1 = x_map[*x1 as usize];
            *y1 = y_map[*y1 as usize];
            *x2 = x_map[*x2 as usize];
            *y2 = y_map[*y2 as usize];
        });

        Ok(Self {
            instructions,
            row_widths,
            col_heights,
        })
    }

    #[must_use]
    pub fn part1(&self) -> u32 {
        self.total_lights(|slice, action| match action {
            Action::TurnOn => slice.iter_mut().for_each(|v| *v = 1),
            Action::TurnOff => slice.iter_mut().for_each(|v| *v = 0),
            Action::Toggle => slice.iter_mut().for_each(|v| *v ^= 1),
        })
    }

    #[must_use]
    pub fn part2(&self) -> u32 {
        self.total_lights(|slice, action| match action {
            Action::TurnOn => slice.iter_mut().for_each(|v| *v += 1),
            Action::TurnOff => slice.iter_mut().for_each(|v| *v = v.saturating_sub(1)),
            Action::Toggle => slice.iter_mut().for_each(|v| *v += 2),
        })
    }

    fn total_lights(&self, f: impl Fn(&mut [u8], Action)) -> u32 {
        let width = self.row_widths.len();
        let height = self.col_heights.len();
        let mut grid = vec![0u8; width * height];

        for &(action, (x1, y1, x2, y2)) in &self.instructions {
            for y in y1..=y2 {
                let index1 = (y as usize) * width + x1 as usize;
                let index2 = (y as usize) * width + x2 as usize;
                f(&mut grid[index1..=index2], action);
            }
        }

        let mut total = 0;
        for y in 0..height {
            for x in 0..width {
                total += grid[y * width + x] as u32
                    * self.row_widths[x] as u32
                    * self.col_heights[y] as u32;
            }
        }
        total
    }
}

examples!(Day06 -> (u32, u32) [
    {
        input: "turn on 0,0 through 999,999\n\
            toggle 0,0 through 999,0\n\
            turn off 499,499 through 500,500",
        part1: 998996,
    },
    {
        input: "turn on 0,0 through 0,0\n\
            toggle 0,0 through 999,999",
        part2: 2000001,
    },
]);
