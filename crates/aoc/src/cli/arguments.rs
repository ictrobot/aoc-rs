use crate::cli::UsageError;
use crate::cli::mode::{self, MainFn};
use aoc::{PUZZLES, PuzzleFn};
use std::collections::VecDeque;
use std::error::Error;
use std::num::NonZeroUsize;
use std::path::PathBuf;
use std::{fs, io};
use utils::date::{Day, Year};
use utils::multiversion::{VERSIONS, Version};

#[derive(Debug, Default)]
pub struct Arguments {
    program_name: Option<String>,
    pub help: bool,
    pub version_override: Option<Version>,
    pub threads_override: Option<NonZeroUsize>,
    pub inputs_dir: Option<PathBuf>,
    mode: Option<MainFn>,
    pub year: Option<Year>,
    pub day: Option<Day>,
    pub extra_args: VecDeque<String>,
}

impl Arguments {
    pub fn parse() -> Result<Self, UsageError> {
        let mut result = Self::default();

        let mut args: VecDeque<String> = std::env::args().collect();
        result.program_name = args.pop_front();

        while let Some(option) = args.pop_front() {
            if option == "--" {
                result.extra_args = args;
                return Ok(result);
            }

            if let Some(option) = option.strip_prefix("--") {
                // Long form options
                if let Some((before, after)) = option.split_once('=') {
                    result
                        .handle_long(before, ArgumentValue::Provided(after.to_string()))
                        .map_err(|e| {
                            UsageError::InvalidArguments(format!("option --{before}: {e}").into())
                        })?;
                } else {
                    result
                        .handle_long(option, ArgumentValue::Available(&mut args))
                        .map_err(|e| {
                            UsageError::InvalidArguments(format!("option --{option}: {e}").into())
                        })?;
                }
                continue;
            }

            // Allow using "-" as a positional argument
            if option.len() > 1
                && let Some(option) = option.strip_prefix('-')
            {
                // Short form options
                let mut options: Vec<_> = option.chars().collect();
                let last = options.pop().unwrap();
                for option in options {
                    result
                        .handle_short(option, ArgumentValue::None)
                        .map_err(|e| {
                            UsageError::InvalidArguments(format!("option -{option}: {e}").into())
                        })?;
                }

                // The last short form option can consume a value
                result
                    .handle_short(last, ArgumentValue::Available(&mut args))
                    .map_err(|e| {
                        UsageError::InvalidArguments(format!("option -{last}: {e}").into())
                    })?;
                continue;
            }

            args.push_front(option);
            break;
        }

        if let Some(i) = args.iter().position(|x| x == "--") {
            result.extra_args = args.split_off(i + 1);
            args.pop_back();
        }

        if let Some(year) = args.pop_front() {
            result.year = match year.parse() {
                Ok(y) => Some(y),
                Err(err) => return Err(UsageError::InvalidArguments(err.into())),
            };

            if let Some(day) = args.pop_front() {
                result.day = match day.parse() {
                    Ok(y) => Some(y),
                    Err(err) => return Err(UsageError::InvalidArguments(err.into())),
                };

                if !args.is_empty() {
                    return Err(UsageError::TooManyArguments);
                }
            }
        }

        Ok(result)
    }

    pub fn help_string(&self) -> String {
        format!(
            r"Usage:
    {program_name}
        Run all solutions

    {program_name} $year
        Run all solutions for the provided year

    {program_name} $year $day
        Run the solution for the provided date

Options:
    --stdin
        Run a single solution, reading input from stdin. $year and $day must be provided.

    --test
        Runs all solutions against all inputs from the inputs directory, comparing the outputs to
        the stored correct answers in the inputs directory. $year may be provided to only test the
        provided year. A custom command template may be provided following a `--` argument to test
        another binary. Requires the 'test-runner' feature to be enabled.

    --multiversion/-m $version
        Override which implementation of multiversioned functions should be used.
        Supported versions: {multiversion_options:?}

    --threads/-t $threads
        Override the number of threads to use for multithreaded solutions.
        In `--test` mode this controls the number of simultaneous tests.

    --inputs $dir
        Specify the directory storing inputs. Defaults to './inputs'.

    --help/-h
        Print this help

{cargo_repo}",
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
            "inputs" => self.option_inputs(value),
            "stdin" => self.option_mode(value, mode::stdin::main),
            #[cfg(feature = "test-runner")]
            "test" => self.option_mode(value, mode::test::main),
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

    fn option_inputs(&mut self, value: ArgumentValue) -> Result<(), Box<dyn Error>> {
        let value = value.required()?.into();
        if self.inputs_dir.is_some() {
            return Err("option provided more than once".into());
        }
        if !fs::metadata(&value).is_ok_and(|m| m.is_dir()) {
            return Err("inputs path must be a directory".into());
        }
        self.inputs_dir = Some(value);
        Ok(())
    }

    fn option_mode(&mut self, value: ArgumentValue, mode: MainFn) -> Result<(), Box<dyn Error>> {
        value.none()?;
        if self.mode.is_some() {
            return Err("mode options are mutually exclusive".into());
        }
        self.mode = Some(mode);
        Ok(())
    }

    pub fn main_fn(&self) -> MainFn {
        self.mode.unwrap_or(mode::default::main)
    }

    pub fn matching_puzzles(&self) -> Vec<(Year, Day, PuzzleFn)> {
        PUZZLES
            .iter()
            .copied()
            .filter(|&(y, d, ..)| self.year.unwrap_or(y) == y && self.day.unwrap_or(d) == d)
            .collect()
    }

    pub fn inputs_dir(&self) -> PathBuf {
        self.inputs_dir
            .clone()
            .unwrap_or_else(|| PathBuf::from("./inputs"))
    }

    pub fn read_input(&self, year: Year, day: Day) -> Result<String, (String, io::Error)> {
        let mut path = self.inputs_dir();
        path.push(format!("year{year:#}"));
        path.push(format!("day{day:#}.txt"));
        fs::read_to_string(&path).map_err(|err| (path.to_string_lossy().to_string(), err))
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
