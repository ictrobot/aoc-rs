use aoc::all_puzzles;
use utils::input::InputType;
use utils::Puzzle;

fn main() {
    macro_rules! matcher {
        ($([$(::$p:ident)+])*) => {$(
            print!("{} {}: ", $(::$p)+::YEAR, $(::$p)+::DAY);
            match $(::$p)+::read_input() {
                Ok(s) => {
                    let solution = $(::$p)+::new(&s, InputType::Real).unwrap();
                    println!("part1={} part2={}", solution.part1(), solution.part2());
                },
                Err(e) => println!("{}", e),
            }
        )*};
    }

    all_puzzles!(matcher);
}
