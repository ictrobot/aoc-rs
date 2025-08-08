use crate::cli::{Arguments, UsageError};
use std::error::Error;
use std::io::{Read, stdin};

#[expect(clippy::print_stdout)]
pub fn main(args: &Arguments) -> Result<(), Box<dyn Error>> {
    let (Some(year), Some(day)) = (args.year, args.day) else {
        return Err(UsageError::MissingArguments("year and day must be provided".into()).into());
    };

    if args.inputs_dir.is_some() {
        return Err(
            UsageError::InvalidArguments("--inputs is incompatible with --stdin".into()).into(),
        );
    }
    if !args.extra_args.is_empty() {
        return Err(UsageError::TooManyArguments.into());
    }

    let puzzles = args.matching_puzzles();
    if puzzles.is_empty() {
        return Err(UsageError::UnsupportedPuzzle(year, day).into());
    }
    assert_eq!(puzzles.len(), 1);
    let (_, _, f) = puzzles[0];

    let mut input = String::new();
    stdin()
        .read_to_string(&mut input)
        .map_err(|err| format!("failed to read input: {err}"))?;

    let (part1, part2) = f(&input).map_err(|err| format!("{year:#} {day:#}: {err}"))?;
    assert!(!part1.contains('\n'));
    assert!(!part2.contains('\n'));

    println!("{part1}");
    println!("{part2}");

    Ok(())
}
