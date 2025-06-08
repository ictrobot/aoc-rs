use crate::cli::{Arguments, UsageError};
use std::error::Error;
use std::time::{Duration, Instant};

#[expect(clippy::print_stdout)]
pub fn main(args: &Arguments) -> Result<(), Box<dyn Error>> {
    if !args.extra_args.is_empty() {
        return Err(UsageError::TooManyArguments.into());
    }

    let puzzles = args.matching_puzzles();
    if puzzles.is_empty() {
        return Err(UsageError::NoSupportedPuzzles.into());
    }

    // FIXME support 80 character wide output (without time?)
    println!(
        "Puzzle  │ Part 1               │ Part 2                                 │ Time      "
    );
    println!(
        "────────┼──────────────────────┼────────────────────────────────────────┼───────────"
    );
    let mut total = Duration::default();
    for (year, day, f) in puzzles {
        let input = args
            .read_input(year, day)
            .map_err(|(path, err)| format!("{year:#} {day:#}: failed to read {path:?}: {err}"))?;

        let start = Instant::now();
        let (part1, part2) = f(&input).map_err(|err| format!("{year:#} {day:#}: {err}"))?;
        let elapsed = start.elapsed();
        total += elapsed;

        // Hack to treat "🎄" as two characters wide
        // ("🎄" is 1 wide in Unicode 8 but 2 wide in Unicode 9+)
        let part1_width = if part1 == "🎄" { 19 } else { 20 };
        let part2_width = if part2 == "🎄" { 37 } else { 38 };

        println!(
            "{year:#} {day:#} │ {part1:<part1_width$} │ {part2:<part2_width$} │ {}",
            format_duration(elapsed)
        );
    }

    println!(
        "────────┼──────────────────────┼────────────────────────────────────────┼───────────"
    );
    println!(
        "                                                                        │ {}",
        format_duration(total),
    );

    Ok(())
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
