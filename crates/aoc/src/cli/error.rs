use std::error::Error;
use std::fmt::{Debug, Display, Formatter};
use std::process::ExitCode;
use utils::date::Date;

#[derive(Debug)]
pub enum UsageError {
    InvalidArguments(Box<dyn Error>),
    MissingArguments(Box<dyn Error>),
    TooManyArguments,
    UnsupportedPuzzle(Date),
    NoSupportedPuzzles,
}

impl Display for UsageError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            UsageError::InvalidArguments(err) => write!(f, "invalid arguments: {err}"),
            UsageError::MissingArguments(err) => write!(f, "missing required arguments: {err}"),
            UsageError::TooManyArguments => write!(f, "too many arguments"),
            UsageError::UnsupportedPuzzle(d) => {
                write!(f, "unsupported puzzle: {:#} day {:#}", d.year(), d.day())
            }
            UsageError::NoSupportedPuzzles => write!(f, "no matching supported puzzles"),
        }
    }
}

impl Error for UsageError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            UsageError::InvalidArguments(err) | UsageError::MissingArguments(err) => {
                Some(err.as_ref())
            }
            _ => None,
        }
    }
}

impl UsageError {
    pub fn exit_code() -> ExitCode {
        ExitCode::from(2)
    }
}

// Used by aoc::cli::mode::test to indicate that the process should exit with failure, but without
// printing an error message as it has already printed the failures.
#[derive(Debug)]
pub struct FailedNoErrorMessage;

impl Display for FailedNoErrorMessage {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "failed")
    }
}

impl Error for FailedNoErrorMessage {}
