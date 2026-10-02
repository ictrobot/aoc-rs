use utils::prelude::*;

/// Recognizing text formed by stacking layers.
#[derive(Clone, Debug)]
pub struct Day08 {
    part1: u32,
    image: [u8; LAYER_LEN],
}

const WIDTH: usize = 25;
const HEIGHT: usize = 6;
const LAYER_LEN: usize = WIDTH * HEIGHT;

impl Day08 {
    pub fn new(input: &str, _: InputType) -> Result<Self, InputError> {
        let (layers, remainder) = input.as_bytes().as_chunks::<LAYER_LEN>();
        if layers.is_empty() || !remainder.is_empty() {
            return Err(InputError::new(
                input,
                0,
                "expected input length to be a multiple of the layer size",
            ));
        }

        let mut image = [b'2'; LAYER_LEN];
        let (mut min_zeroes, mut part1) = (u8::MAX, 0);

        for layer in layers {
            let (mut zeroes, mut ones, mut twos) = (0, 0, 0);
            for (&b, o) in layer.iter().zip(image.iter_mut()) {
                zeroes += u8::from(b == b'0');
                ones += u8::from(b == b'1');
                twos += u8::from(b == b'2');
                *o = if *o == b'2' { b } else { *o };
            }

            if zeroes < min_zeroes {
                min_zeroes = zeroes;
                part1 = u32::from(ones) * u32::from(twos);
            }

            if zeroes + ones + twos != LAYER_LEN as u8 {
                return Err(InputError::new(
                    input,
                    layer
                        .iter()
                        .copied()
                        .find(|&b| b != b'0' && b != b'1' && b != b'2')
                        .unwrap() as char,
                    "expected '0', '1' or '2'",
                ));
            }
        }

        Ok(Self { part1, image })
    }

    #[must_use]
    pub fn part1(&self) -> u32 {
        self.part1
    }

    #[must_use]
    pub fn part2(&self) -> String {
        let mut output = String::with_capacity(WIDTH / 5);
        let (rows, []) = self.image.as_chunks::<WIDTH>() else {
            unreachable!("LAYER_LEN is a multiple of WIDTH");
        };

        for x in (0..WIDTH).step_by(5) {
            let mut letter = 0;
            for row in rows {
                for &pixel in &row[x..x + 5] {
                    letter = (letter << 1) | u32::from(pixel == b'1');
                }
            }

            output.push(Self::ocr(letter));
        }

        output
    }

    #[inline]
    pub(crate) fn ocr(letter: u32) -> char {
        //  ##  ###   ##  #### ####  ##  #  #   ## #  # #    ###  ###  #  # #   # ####
        // #  # #  # #  # #    #    #  # #  #    # # #  #    #  # #  # #  # #   #    #
        // #  # ###  #    ###  ###  #    ####    # ##   #    #  # #  # #  #  # #    #
        // #### #  # #    #    #    # ## #  #    # # #  #    ###  ###  #  #   #    #
        // #  # #  # #  # #    #    #  # #  # #  # # #  #    #    # #  #  #   #   #
        // #  # ###   ##  #### #     ### #  #  ##  #  # #### #    #  #  ##    #   ####
        match letter {
            //11111_22222_33333_44444_55555_66666
            0b01100_10010_10010_11110_10010_10010 => 'A',
            0b11100_10010_11100_10010_10010_11100 => 'B',
            0b01100_10010_10000_10000_10010_01100 => 'C',
            0b11110_10000_11100_10000_10000_11110 => 'E',
            0b11110_10000_11100_10000_10000_10000 => 'F',
            0b01100_10010_10000_10110_10010_01110 => 'G',
            0b10010_10010_11110_10010_10010_10010 => 'H',
            0b00110_00010_00010_00010_10010_01100 => 'J',
            0b10010_10100_11000_10100_10100_10010 => 'K',
            0b10000_10000_10000_10000_10000_11110 => 'L',
            0b11100_10010_10010_11100_10000_10000 => 'P',
            0b11100_10010_10010_11100_10100_10010 => 'R',
            0b10010_10010_10010_10010_10010_01100 => 'U',
            0b10001_10001_01010_00100_00100_00100 => 'Y',
            0b11110_00010_00100_01000_10000_11110 => 'Z',
            _ => Self::unknown_letter(letter),
        }
    }

    #[cold]
    fn unknown_letter(letter: u32) -> char {
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

examples!(Day08 -> (u32, &'static str) []);
