use aoc::all_puzzles;
use utils::input::InputType;
use utils::Puzzle;

fn main() {
    macro_rules! matcher {
        ($([$(::$p:ident)+])*) => {$({
            fn puzzle() {
                print!("{} {}: ", $(::$p)+::YEAR, $(::$p)+::DAY);
                match $(::$p)+::read_input() {
                    Ok(s) => {
                        let solution = match $(::$p)+::new(&s, InputType::Real) {
                            Ok(v) => v,
                            Err(e) => {
                                println!("{}", e);
                                return;
                            },
                        };
                        println!("part1={} part2={}", solution.part1(), solution.part2());
                    },
                    Err(e) => println!("{}", e),
                }
            }
            puzzle();
        })*};
    }

    all_puzzles!(matcher);
}
