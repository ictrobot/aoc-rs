use crate::cli::Options;
use std::fs::read_to_string;
use std::io;
use std::path::PathBuf;
use std::process::exit;
use std::time::{Duration, Instant};
use utils::date::{Day, Year};
use utils::multiversion::Version;

mod cli;

fn main() {
    let args = match Options::parse() {
        Ok(x) => x,
        Err(err) => {
            eprintln!("{err}");
            exit(2);
        }
    };
    if args.help {
        println!("{}", args.help());
        exit(0);
    }
    if let Some(version) = args.version_override {
        Version::set_override(version);
    }

    let puzzles = args.matching_puzzles();
    if puzzles.is_empty() {
        eprintln!("no matching solutions");
        exit(1);
    }

    println!("Puzzle  │ Part 1                      │ Part 2                      │ Time      ");
    println!("────────┼─────────────────────────────┼─────────────────────────────┼───────────");

    let mut total = Duration::default();
    for (year, day, f) in puzzles {
        let input = match read_input(year, day) {
            Ok(input) => input,
            Err((path, err)) => {
                println!("{year:#} {day:#}: failed to read {path:?}: {err}");
                exit(1);
            }
        };

        let start = Instant::now();
        match f(&input) {
            Ok((part1, part2)) => {
                let elapsed = start.elapsed();
                total += elapsed;

                println!(
                    "{year:#} {day:#} │ {part1:<27} │ {part2:<27} │ {}",
                    format_duration(elapsed)
                );
            }
            Err(input_err) => {
                println!("{year:#} {day:#}: {input_err}");
                exit(1);
            }
        }
    }

    println!("────────┴─────────────────────────────┴─────────────────────────────┼───────────");
    println!(
        "                                                                    │ {}",
        format_duration(total),
    );
}

pub fn read_input(year: Year, day: Day) -> Result<String, (String, io::Error)> {
    let mut path = PathBuf::new();
    path.push("inputs");
    path.push(format!("year{year:#}"));
    path.push(format!("day{day:#}.txt"));
    match read_to_string(&path) {
        Ok(s) => Ok(s.trim_ascii_end().replace("\r\n", "\n")),
        Err(err) => Err((path.to_string_lossy().to_string(), err)),
    }
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
