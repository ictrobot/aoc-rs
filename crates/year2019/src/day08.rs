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

        for x in (0..WIDTH).step_by(5) {
            let mut letter = 0;
            for row in self.image.chunks_exact(WIDTH) {
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
            //111112222233333444445555566666
            0b011001001010010111101001010010 => 'A',
            0b111001001011100100101001011100 => 'B',
            0b011001001010000100001001001100 => 'C',
            0b111101000011100100001000011110 => 'E',
            0b111101000011100100001000010000 => 'F',
            0b011001001010000101101001001110 => 'G',
            0b100101001011110100101001010010 => 'H',
            0b001100001000010000101001001100 => 'J',
            0b100101010011000101001010010010 => 'K',
            0b100001000010000100001000011110 => 'L',
            0b111001001010010111001000010000 => 'P',
            0b111001001010010111001010010010 => 'R',
            0b100101001010010100101001001100 => 'U',
            0b100011000101010001000010000100 => 'Y',
            0b111100001000100010001000011110 => 'Z',
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
