use utils::prelude::*;

/// Converting pixels to text.
#[derive(Clone, Debug)]
pub struct Day08 {
    grid: [[bool; 50]; 6],
}

#[derive(Copy, Clone, Debug)]
enum Instruction {
    Rect { width: u32, height: u32 },
    RotateRow { y: u32, by: u32 },
    RotateCol { x: u32, by: u32 },
}

impl Day08 {
    pub fn new(input: &str, _: InputType) -> Result<Self, InputError> {
        let rect = parser::u32()
            .with_prefix("rect ")
            .with_suffix("x")
            .then(parser::u32())
            .map(|(width, height)| Instruction::Rect { width, height });
        let rotate_row = parser::u32()
            .with_prefix("rotate row y=")
            .with_suffix(" by ")
            .then(parser::u32())
            .map(|(y, by)| Instruction::RotateRow { y, by });
        let rotate_col = parser::u32()
            .with_prefix("rotate column x=")
            .with_suffix(" by ")
            .then(parser::u32())
            .map(|(x, by)| Instruction::RotateCol { x, by });

        let mut grid = [[false; 50]; 6];
        for item in parser::one_of((rect, rotate_row, rotate_col))
            .with_eol()
            .parse_iterator(input)
        {
            match item? {
                Instruction::Rect { width, height } => {
                    for row in &mut grid[..height as usize] {
                        row[..width as usize].fill(true);
                    }
                }
                Instruction::RotateRow { y, by } => grid[y as usize].rotate_right(by as usize),
                Instruction::RotateCol { x, by } => {
                    let col = grid.map(|row| row[x as usize]);
                    for y in 0..6 {
                        grid[y][x as usize] = col[(y + 6 - by as usize) % 6];
                    }
                }
            }
        }

        Ok(Self { grid })
    }

    #[must_use]
    pub fn part1(&self) -> usize {
        self.grid.as_flattened().iter().filter(|&&x| x).count()
    }

    #[must_use]
    pub fn part2(&self) -> String {
        let mut output = String::with_capacity(10);

        for i in (0..50).step_by(5) {
            let mut letter = 0;
            for row in self.grid {
                for &b in &row[i..i + 5] {
                    letter <<= 1;
                    if b {
                        letter |= 1;
                    }
                }
            }
            output.push(Self::ocr(letter));
        }

        output
    }

    fn ocr(letter: u32) -> char {
        //  ##  ###   ##  #### ####  ##  #  #  ###   ## #  # #     ##  ###  ###   ### #  # #   #####
        // #  # #  # #  # #    #    #  # #  #   #     # # #  #    #  # #  # #  # #    #  # #   #   #
        // #  # ###  #    ###  ###  #    ####   #     # ##   #    #  # #  # #  # #    #  #  # #   #
        // #### #  # #    #    #    # ## #  #   #     # # #  #    #  # ###  ###   ##  #  #   #   #
        // #  # #  # #  # #    #    #  # #  #   #  #  # # #  #    #  # #    # #     # #  #   #  #
        // #  # ###   ##  #### #     ### #  #  ###  ##  #  # ####  ##  #    #  # ###   ##    #  ####
        match letter {
            //11111_22222_33333_44444_55555_66666
            0b01100_10010_10010_11110_10010_10010 => 'A',
            0b11100_10010_11100_10010_10010_11100 => 'B',
            0b01100_10010_10000_10000_10010_01100 => 'C',
            0b11110_10000_11100_10000_10000_11110 => 'E',
            0b11110_10000_11100_10000_10000_10000 => 'F',
            0b01100_10010_10000_10110_10010_01110 => 'G',
            0b10010_10010_11110_10010_10010_10010 => 'H',
            0b01110_00100_00100_00100_00100_01110 => 'I',
            0b00110_00010_00010_00010_10010_01100 => 'J',
            0b10010_10100_11000_10100_10100_10010 => 'K',
            0b10000_10000_10000_10000_10000_11110 => 'L',
            0b01100_10010_10010_10010_10010_01100 => 'O',
            0b11100_10010_10010_11100_10000_10000 => 'P',
            0b11100_10010_10010_11100_10100_10010 => 'R',
            0b01110_10000_10000_01100_00010_11100 => 'S',
            0b10010_10010_10010_10010_10010_01100 => 'U',
            0b10001_10001_01010_00100_00100_00100 => 'Y',
            0b11110_00010_00100_01000_10000_11110 => 'Z',
            _ => {
                let mut display = String::new();
                for b in (0..30).rev() {
                    display.push(if letter & (1 << b) == 0 { ' ' } else { '#' });
                    if b % 5 == 0 {
                        display.push('\n');
                    }
                }
                panic!("unknown letter {letter:#032b}:\n{display}");
            }
        }
    }
}

examples!(Day08 -> (usize, &'static str) [
    {
        input: "rect 3x2\n\
            rotate column x=1 by 1\n\
            rotate row y=0 by 4\n\
            rotate column x=1 by 1",
        part1: 6,
    },
]);
