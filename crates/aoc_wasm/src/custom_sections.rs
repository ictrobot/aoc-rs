//! Generates custom sections containing the list of solutions and their examples.

use aoc::all_puzzles;

const fn examples_len<P1, P2>(examples: &[(&'static str, Option<P1>, Option<P2>)]) -> usize {
    let mut length = 0;
    let mut i = 0;
    while i < examples.len() {
        let (input, _, _) = examples[i];
        length += input.len() + 2;
        i += 1;
    }
    length
}

const fn examples_section<const N: usize, P1, P2>(
    examples: &[(&'static str, Option<P1>, Option<P2>)],
) -> [u8; N] {
    let mut output = [0; N];
    let mut o = 0;
    let mut e = 0;
    while e < examples.len() {
        output[o] = if examples[e].1.is_some() { 0x10 } else { 0 }
            | if examples[e].2.is_some() { 0x01 } else { 0 };
        o += 1;

        let mut i = 0;
        let input = examples[e].0.as_bytes();
        while i < input.len() {
            output[o] = input[i];
            i += 1;
            o += 1;
        }

        o += 1; // null byte

        e += 1;
    }
    output
}

macro_rules! matcher {
    ($(
        $y:literal => $year:ident{$(
            $d:literal => $day:ident,
        )*}
    )*) => {$(
        mod $year {$(
            mod $day {
                use aoc::utils::PuzzleExamples;

                #[link_section = "aoc_puzzles"]
                #[used]
                static PUZZLE_LIST: [u8; 6] = [
                    b'0' + ($y / 1000u16) as u8,
                    b'0' + (($y / 100u16) % 10) as u8,
                    b'0' + (($y / 10u16) % 10) as u8,
                    b'0' + ($y % 10u16) as u8,
                    b'0' + ($d / 10) % 10,
                    b'0' + $d % 10,
                ];

                #[link_section = concat!("aoc_examples_", stringify!($y), "_", stringify!($d))]
                #[used]
                static PUZZLE_EXAMPLES: [u8; super::super::examples_len(aoc::$year::$day::EXAMPLES)]
                    = super::super::examples_section(aoc::$year::$day::EXAMPLES);
            }
        )*}
    )*};
}
all_puzzles!(matcher);
