use std::error::Error;
use std::fmt::{Display, Formatter};
use std::process::ExitStatus;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TestError {
    SolverError {
        status: ExitStatus,
        stderr: String,
        stdout: String,
    },
    InvalidOutput,
    IncorrectAnswer {
        part1: Option<(String, String)>,
        part2: Option<(String, String)>,
    },
    Killed,
}

impl TestError {
    pub fn is_unsupported_puzzle(&self) -> bool {
        match self {
            TestError::SolverError { status, .. } if status.code() == Some(2) => true,
            TestError::InvalidOutput => true,
            _ => false,
        }
    }
}

impl Display for TestError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            TestError::SolverError { stderr, .. }
                if self.is_unsupported_puzzle() && !stderr.is_empty() =>
            {
                write!(f, "unsupported puzzle (exit code 2): {stderr:?}")
            }
            TestError::SolverError { .. } if self.is_unsupported_puzzle() => {
                write!(f, "unsupported puzzle (exit code 2)")
            }
            TestError::SolverError { status, stderr, .. } if !stderr.is_empty() => {
                write!(f, "{status}: {stderr:?}")
            }
            TestError::SolverError { status, stdout, .. } if !stdout.is_empty() => {
                write!(f, "{status}: {stdout:?}")
            }
            TestError::SolverError { status, .. } => {
                write!(f, "{status}")
            }
            TestError::InvalidOutput => write!(
                f,
                "unsupported puzzle (output didn't match the expected format)"
            ),
            TestError::IncorrectAnswer { part1, part2 } => {
                if let Some((answer, expected)) = part1 {
                    write!(f, "part 1 incorrect, got {answer:?}, expected {expected:?}")?;
                }
                if let Some((answer, expected)) = part2 {
                    if !part1.is_none() {
                        write!(f, ". ")?;
                    }
                    write!(f, "part 2 incorrect, got {answer:?}, expected {expected:?}")?;
                }
                Ok(())
            }
            TestError::Killed => write!(f, "solver exceeded timeout"),
        }
    }
}

impl Error for TestError {}
