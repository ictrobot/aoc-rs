use utils::point::Point2D;
use utils::prelude::*;

/// Recognizing text formed by converging points.
#[derive(Clone, Debug)]
pub struct Day10 {
    message: String,
    seconds: u32,
}

impl Day10 {
    pub fn new(input: &str, _: InputType) -> Result<Self, InputError> {
        let point = parser::i32()
            .with_prefix(parser::take_while(|&x| x == b' '))
            .repeat_n(b',')
            .map(Point2D::from);

        let mut points = point
            .with_prefix("position=<")
            .with_suffix("> velocity=<")
            .then(point)
            .with_suffix(">")
            .repeat(parser::eol(), 2)
            .parse_complete(input)?;

        let mut seconds = 0;
        let (mut last_x_diff, mut last_y_diff) = (i32::MAX, i32::MAX);
        let message = 'time: loop {
            let (mut min_x, mut max_x) = (i32::MAX, i32::MIN);
            let (mut min_y, mut min_y_vel, mut max_y, mut max_y_vel) = (i32::MAX, 0, i32::MIN, 0);
            for (p, v) in &points {
                min_x = min_x.min(p.x);
                max_x = max_x.max(p.x);

                if p.y < min_y {
                    min_y = p.y;
                    min_y_vel = v.y;
                }
                if p.y > max_y {
                    max_y = p.y;
                    max_y_vel = v.y;
                }
            }

            let (x_diff, y_diff) = (max_x.saturating_sub(min_x), max_y.saturating_sub(min_y));
            if x_diff > last_x_diff
                || y_diff > last_y_diff
                || (x_diff == last_x_diff && y_diff == last_y_diff)
            {
                return Err(InputError::new(input, 0, "points never converge"));
            }
            (last_x_diff, last_y_diff) = (x_diff, y_diff);

            // Letters are 6 wide and 10 tall, with 2 wide gaps between them
            if y_diff != 9 || x_diff % 8 != 5 {
                let advance_by = if y_diff >= 10 && min_y_vel.saturating_sub(max_y_vel) > 0 {
                    ((y_diff - 9) / min_y_vel.saturating_sub(max_y_vel)).max(1)
                } else {
                    1
                };

                for (p, v) in points.iter_mut() {
                    *p += *v * advance_by;
                }
                seconds += advance_by as u32;

                continue;
            }

            let len = ((max_x - min_x + 3) / 8) as usize;
            let mut letters = vec![0u64; len];
            for (p, _) in points.iter() {
                if (p.x - min_x) % 8 >= 6 {
                    // Point where there should be a 2-wide gap
                    continue 'time;
                }

                letters[(p.x - min_x) as usize / 8] |=
                    1 << (59 - ((p.x - min_x) % 8 + 6 * (p.y - min_y)));
            }
            break letters.into_iter().map(Self::ocr).collect::<String>();
        };

        Ok(Self { message, seconds })
    }

    fn ocr(letter: u64) -> char {
        //   ##    #####    ####   ######  ######   ####   #    #     ###  #    #  #       #    #
        //  #  #   #    #  #    #  #       #       #    #  #    #      #   #   #   #       ##   #
        // #    #  #    #  #       #       #       #       #    #      #   #  #    #       ##   #
        // #    #  #    #  #       #       #       #       #    #      #   # #     #       # #  #
        // #    #  #####   #       #####   #####   #       ######      #   ##      #       # #  #
        // ######  #    #  #       #       #       #  ###  #    #      #   ##      #       #  # #
        // #    #  #    #  #       #       #       #    #  #    #      #   # #     #       #  # #
        // #    #  #    #  #       #       #       #    #  #    #  #   #   #  #    #       #   ##
        // #    #  #    #  #    #  #       #       #   ##  #    #  #   #   #   #   #       #   ##
        // #    #  #####    ####   ######  #        ### #  #    #   ###    #    #  ######  #    #
        //
        // #####   #####   #    #  ######
        // #    #  #    #  #    #       #
        // #    #  #    #   #  #        #
        // #    #  #    #   #  #       #
        // #####   #####     ##       #
        // #       #  #      ##      #
        // #       #   #    #  #    #
        // #       #   #    #  #   #
        // #       #    #  #    #  #
        // #       #    #  #    #  ######
        match letter {
            //000000111111222222333333444444555555666666777777888888999999
            0b001100010010100001100001100001111111100001100001100001100001 => 'A',
            0b111110100001100001100001111110100001100001100001100001111110 => 'B',
            0b011110100001100000100000100000100000100000100000100001011110 => 'C',
            0b111111100000100000100000111110100000100000100000100000111111 => 'E',
            0b111111100000100000100000111110100000100000100000100000100000 => 'F',
            0b011110100001100000100000100000100111100001100001100011011101 => 'G',
            0b100001100001100001100001111111100001100001100001100001100001 => 'H',
            0b000111000010000010000010000010000010000010100010100010011100 => 'J',
            0b100001100010100100101000110000110000101000100100100010100001 => 'K',
            0b100000100000100000100000100000100000100000100000100000111111 => 'L',
            0b100001110001110001101001101001100101100101100011100011100001 => 'N',
            0b111110100001100001100001111110100000100000100000100000100000 => 'P',
            0b111110100001100001100001111110100100100010100010100001100001 => 'R',
            0b100001100001010010010010001100001100010010010010100001100001 => 'X',
            0b111111000001000001000010000100001000010000100000100000111111 => 'Z',
            _ => Self::unknown_letter(letter),
        }
    }

    #[cold]
    fn unknown_letter(letter: u64) -> char {
        let mut display = String::new();
        for b in (0..60).rev() {
            display.push(if letter & (1 << b) == 0 { ' ' } else { '#' });
            if b % 6 == 0 {
                display.push('\n');
            }
        }
        panic!("unknown letter {letter:#062b}:\n{display}");
    }

    #[must_use]
    pub fn part1(&self) -> &str {
        &self.message
    }

    #[must_use]
    pub fn part2(&self) -> u32 {
        self.seconds
    }
}

examples!(Day10 -> (&'static str, u32) []);
