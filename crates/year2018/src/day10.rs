use utils::geometry::Vec2;
use utils::prelude::*;

/// Recognizing text formed by converging points.
#[derive(Clone, Debug)]
pub struct Day10 {
    message: String,
    seconds: u32,
}

impl Day10 {
    pub fn new(input: &str, _: InputType) -> Result<Self, InputError> {
        let vec2 = parser::i32()
            .with_prefix(parser::take_while(|&x| x == b' '))
            .repeat_n(b',')
            .map(Vec2::from);

        let mut points = vec2
            .with_prefix("position=<")
            .with_suffix("> velocity=<")
            .then(vec2)
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

                for (p, v) in &mut points {
                    *p += *v * advance_by;
                }
                seconds += advance_by as u32;

                continue;
            }

            let len = ((max_x - min_x + 3) / 8) as usize;
            let mut letters = vec![0u64; len];
            for (p, _) in &points {
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

    #[expect(clippy::unreadable_literal)]
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
            //000000_111111_222222_333333_444444_555555_666666_777777_888888_999999
            0b001100_010010_100001_100001_100001_111111_100001_100001_100001_100001 => 'A',
            0b111110_100001_100001_100001_111110_100001_100001_100001_100001_111110 => 'B',
            0b011110_100001_100000_100000_100000_100000_100000_100000_100001_011110 => 'C',
            0b111111_100000_100000_100000_111110_100000_100000_100000_100000_111111 => 'E',
            0b111111_100000_100000_100000_111110_100000_100000_100000_100000_100000 => 'F',
            0b011110_100001_100000_100000_100000_100111_100001_100001_100011_011101 => 'G',
            0b100001_100001_100001_100001_111111_100001_100001_100001_100001_100001 => 'H',
            0b000111_000010_000010_000010_000010_000010_000010_100010_100010_011100 => 'J',
            0b100001_100010_100100_101000_110000_110000_101000_100100_100010_100001 => 'K',
            0b100000_100000_100000_100000_100000_100000_100000_100000_100000_111111 => 'L',
            0b100001_110001_110001_101001_101001_100101_100101_100011_100011_100001 => 'N',
            0b111110_100001_100001_100001_111110_100000_100000_100000_100000_100000 => 'P',
            0b111110_100001_100001_100001_111110_100100_100010_100010_100001_100001 => 'R',
            0b100001_100001_010010_010010_001100_001100_010010_010010_100001_100001 => 'X',
            0b111111_000001_000001_000010_000100_001000_010000_100000_100000_111111 => 'Z',
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
