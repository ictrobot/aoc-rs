use aoc::{PuzzleFn, PUZZLES};
use std::collections::VecDeque;
use std::error::Error;
use std::num::NonZeroUsize;
use utils::date::{Day, Year};
use utils::multiversion::{Version, VERSIONS};

#[derive(Debug, Default)]
pub struct Options {
    program_name: Option<String>,
    pub help: bool,
    pub version_override: Option<Version>,
    pub threads_override: Option<NonZeroUsize>,
    pub year: Option<Year>,
    pub day: Option<Day>,
}

impl Options {
    pub fn parse() -> Result<Self, String> {
        let mut result = Self::default();

        let mut args: VecDeque<String> = std::env::args().collect();
        result.program_name = args.pop_front();

        while let Some(option) = args.pop_front() {
            if option == "--" {
                break;
            }

            if let Some(option) = option.strip_prefix("--") {
                // Long form options
                if let Some((before, after)) = option.split_once('=') {
                    result
                        .handle_long(before, ArgumentValue::Provided(after.to_string()))
                        .map_err(|e| format!("option --{before}: {e}"))?;
                } else {
                    result
                        .handle_long(option, ArgumentValue::Available(&mut args))
                        .map_err(|e| format!("option --{option}: {e}"))?;
                }
                continue;
            }

            // Allow using "-" as a positional argument
            if option.len() > 1 {
                if let Some(option) = option.strip_prefix('-') {
                    // Short form options
                    let mut options: Vec<_> = option.chars().collect();
                    let last = options.pop().unwrap();
                    for option in options {
                        result
                            .handle_short(option, ArgumentValue::None)
                            .map_err(|e| format!("option -{option}: {e}"))?;
                    }

                    // Last short form option can consume a value
                    result
                        .handle_short(last, ArgumentValue::Available(&mut args))
                        .map_err(|e| format!("option -{last}: {e}"))?;
                    continue;
                }
            }

            args.push_front(option);
            break;
        }

        if let Some(year) = args.pop_front() {
            result.year = match year.parse() {
                Ok(y) => Some(y),
                Err(err) => return Err(err.to_string()),
            };

            if let Some(day) = args.pop_front() {
                result.day = match day.parse() {
                    Ok(y) => Some(y),
                    Err(err) => return Err(err.to_string()),
                };

                if !args.is_empty() {
                    return Err("too many arguments".to_string());
                }
            }
        }

        Ok(result)
    }

    pub fn help(&self) -> String {
        format!(
            r#"Usage:
    {program_name}
        Run all solutions

    {program_name} $year
        Run all solutions for the provided year
        
    {program_name} $year $day
        Run the solution for the provided date

Options:
    --multiversion/-m $version
        Override which implementation of multiversioned functions should be used.
        Supported versions: {multiversion_options:?}
        
    --threads/-t $threads
        Override the number of threads to use for multithreaded solutions.

    --help/-h
        Print this help

{cargo_repo}"#,
            program_name = self.program_name.as_ref().map_or("aoc", String::as_str),
            multiversion_options = *VERSIONS,
            cargo_repo = env!("CARGO_PKG_REPOSITORY"),
        )
    }

    fn handle_long(&mut self, name: &str, value: ArgumentValue) -> Result<(), Box<dyn Error>> {
        match name {
            "help" => self.option_help(value),
            "multiversion" => self.option_multiversion(value),
            "threads" => self.option_threads(value),
            _ => Err("unknown option".into()),
        }
    }

    fn handle_short(&mut self, name: char, value: ArgumentValue) -> Result<(), Box<dyn Error>> {
        match name {
            'h' => self.option_help(value),
            'm' => self.option_multiversion(value),
            't' => self.option_threads(value),
            _ => Err("unknown option".into()),
        }
    }

    fn option_help(&mut self, value: ArgumentValue) -> Result<(), Box<dyn Error>> {
        value.none()?;
        self.help = true;
        Ok(())
    }

    fn option_multiversion(&mut self, value: ArgumentValue) -> Result<(), Box<dyn Error>> {
        let value = value.required()?;
        if self.version_override.is_some() {
            return Err("option provided more than once".into());
        }
        self.version_override = Some(value.parse()?);
        Ok(())
    }

    fn option_threads(&mut self, value: ArgumentValue) -> Result<(), Box<dyn Error>> {
        let value = value.required()?;
        if self.threads_override.is_some() {
            return Err("option provided more than once".into());
        }
        self.threads_override = Some(value.parse()?);
        Ok(())
    }

    pub fn matching_puzzles(&self) -> Vec<(Year, Day, PuzzleFn)> {
        PUZZLES
            .iter()
            .copied()
            .filter(|&(y, d, ..)| self.year.unwrap_or(y) == y && self.day.unwrap_or(d) == d)
            .collect()
    }
}

#[must_use]
enum ArgumentValue<'a> {
    // Used with --long=value options
    Provided(String),
    // Used when the next argument can be consumed as the value
    Available(&'a mut VecDeque<String>),
    // Used for short options not at the end of a group
    None,
}

impl ArgumentValue<'_> {
    fn required(self) -> Result<String, Box<dyn Error>> {
        match self {
            ArgumentValue::Provided(value) => Ok(value),
            ArgumentValue::Available(args) if !args.is_empty() => Ok(args.pop_front().unwrap()),
            _ => Err("option requires an argument".into()),
        }
    }

    fn none(self) -> Result<(), Box<dyn Error>> {
        match self {
            ArgumentValue::Provided(_) => Err("option does not take an argument".into()),
            _ => Ok(()),
        }
    }
}
