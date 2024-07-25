use aoc::all_puzzles;
use std::time::{Duration, Instant};
use utils::input::InputType;
use utils::Puzzle;

fn main() {
    println!("Puzzle  │ Part 1                      │ Part 2                      │ Time      ");
    println!("────────┼─────────────────────────────┼─────────────────────────────┼───────────");

    let mut total = Duration::default();

    macro_rules! matcher {
        ($([$(::$p:ident)+])*) => {$({
            fn puzzle(total: &mut Duration) {
                print!("{:#} {:#} │ ", $(::$p)+::YEAR, $(::$p)+::DAY);
                match $(::$p)+::read_input() {
                    Ok(s) => {
                        let start = Instant::now();
                        match $(::$p)+::new(&s, InputType::Real) {
                            Ok(solution) => {
                                let part1 = solution.part1();
                                let part2 = solution.part2();
                                let elapsed = start.elapsed();
                                *total += elapsed;

                                println!(
                                    "{part1:<27} │ {part2:<27} │ {}",
                                    format_duration(elapsed),
                                );
                            },
                            Err(e) => println!("{}", e),
                        }
                    },
                    Err(e) => println!("{}", e),
                }
            }
            puzzle(&mut total);
        })*};
    }
    all_puzzles!(matcher);

    println!("────────┴─────────────────────────────┴─────────────────────────────┼───────────");
    println!(
        "                                                                    │ {}",
        format_duration(total),
    );
}

fn format_duration(d: Duration) -> String {
    let (unit, multiplier) = if d.as_micros() < 1000 {
        ("µ", 1_000_000.)
    } else {
        ("m", 1_000.)
    };

    let float = d.as_secs_f64() * multiplier;
    let precision = if float < 1000. { 3 } else { 0 };
    format!("{float:7.precision$} {unit}s")
}
