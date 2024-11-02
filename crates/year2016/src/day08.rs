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
            .with_suffix(parser::eol())
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
            //111112222233333444445555566666
            0b011001001010010111101001010010 => 'A',
            0b111001001011100100101001011100 => 'B',
            0b011001001010000100001001001100 => 'C',
            0b111101000011100100001000011110 => 'E',
            0b111101000011100100001000010000 => 'F',
            0b011001001010000101101001001110 => 'G',
            0b100101001011110100101001010010 => 'H',
            0b011100010000100001000010001110 => 'I',
            0b001100001000010000101001001100 => 'J',
            0b100101010011000101001010010010 => 'K',
            0b100001000010000100001000011110 => 'L',
            0b011001001010010100101001001100 => 'O',
            0b111001001010010111001000010000 => 'P',
            0b111001001010010111001010010010 => 'R',
            0b011101000010000011000001011100 => 'S',
            0b100101001010010100101001001100 => 'U',
            0b100011000101010001000010000100 => 'Y',
            0b111100001000100010001000011110 => 'Z',
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
