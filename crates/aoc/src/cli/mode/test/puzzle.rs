use crate::cli::mode::test::error::TestError;
use crate::cli::mode::test::process::ProcessResult;
use crate::cli::mode::test::test_case::TestCase;
use std::io;
use std::iter::FusedIterator;
use std::ops::{Index, IndexMut};
use std::path::PathBuf;
use std::time::Instant;
use utils::date::{Day, Year};

#[derive(Debug, Default)]
pub struct Puzzle {
    started: Option<Instant>,
    test_cases: usize,
    succeeded: usize,
    failures: Vec<(PathBuf, TestError)>,
}

impl Puzzle {
    #[must_use = "returns whether the status has changed"]
    pub fn set_case_count(&mut self, count: usize) -> bool {
        assert_eq!(self.test_cases, 0, "Test case count already set");
        assert_eq!(self.started, None, "Test case count already set");
        if count == 0 {
            // Setting started immediately marks the day as unknown
            self.started = Some(Instant::now());
            true
        } else {
            self.test_cases = count;
            false
        }
    }

    #[must_use = "returns whether the status has changed"]
    pub fn case_started(&mut self) -> bool {
        assert_ne!(self.test_cases, 0);
        if self.started.is_none() {
            self.started = Some(Instant::now());
            true
        } else {
            false
        }
    }

    pub fn case_finished(
        &mut self,
        test_case: TestCase,
        result: ProcessResult,
    ) -> io::Result<bool> {
        let error = if result.killed {
            Some(TestError::Killed)
        } else if !result.exit_status.success() {
            Some(TestError::SolverError {
                status: result.exit_status,
                stderr: result.stderr?,
                stdout: result.stdout?,
            })
        } else if let Some(err) = result.stdin {
            // If the process exited successfully, it should have read the entire input
            return Err(err);
        } else {
            let stdout = result.stdout?;
            let stderr = result.stderr?;

            if !stderr.is_empty() {
                Some(TestError::SolverError {
                    status: result.exit_status,
                    stderr,
                    stdout,
                })
            } else if stdout.lines().count() != 2 {
                Some(TestError::InvalidOutput)
            } else {
                let mut lines = stdout.lines();

                let part1 = {
                    let answer = lines.next().unwrap();
                    if answer == test_case.part1 {
                        None
                    } else {
                        Some((answer.to_string(), test_case.part1))
                    }
                };

                let part2 = {
                    let answer = lines.next().unwrap();
                    match test_case.part2 {
                        Some(expected) if answer == expected => None,
                        Some(expected) => Some((answer.to_string(), expected.clone())),
                        None => None,
                    }
                };

                if part1.is_none() && part2.is_none() {
                    None
                } else {
                    Some(TestError::IncorrectAnswer { part1, part2 })
                }
            }
        };

        match error {
            None => self.succeeded += 1,
            Some(e) => self.failures.push((test_case.input_path, e)),
        }

        Ok(self.succeeded + self.failures.len() == self.test_cases)
    }

    pub fn get_status(&self) -> Status {
        if let Some(started) = self.started {
            if self.test_cases == 0 {
                Status::Unknown
            } else if self.succeeded + self.failures.len() < self.test_cases {
                Status::Running { started }
            } else if self.failures.is_empty() {
                Status::Passed
            } else if self.failures.len() < self.test_cases {
                Status::Mixed
            } else if self.failures.iter().all(|(_, e)| e.is_unsupported_puzzle()) {
                Status::Unsupported
            } else {
                Status::Failed
            }
        } else {
            Status::Initial
        }
    }

    pub fn get_case_count(&self) -> usize {
        self.test_cases
    }

    pub fn get_succeeded(&self) -> usize {
        self.succeeded
    }

    pub fn get_failures(&self) -> &[(PathBuf, TestError)] {
        self.failures.as_slice()
    }

    pub fn iter(min_year: Year, max_year: Year) -> impl FusedIterator<Item = (Year, Day)> {
        (min_year.to_u16()..=max_year.to_u16())
            .map(|y| Year::new(y).unwrap())
            .flat_map(|y| (1..=25).map(|d| Day::new(d).unwrap()).map(move |d| (y, d)))
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub enum Status {
    #[default]
    Initial,
    Running {
        started: Instant,
    },
    Passed,
    Failed,
    Unsupported,
    Mixed,
    Unknown,
}

impl Status {
    pub const fn symbol(&self) -> &'static str {
        match self {
            // Symbol for Status::Running overwritten in OutputGrid::symbol with spinner
            Status::Initial | Status::Running { .. } => " ",
            Status::Passed => "\x1B[0;32m✓\x1B[0m",
            Status::Failed => "\x1B[1;31m✗\x1B[0m",
            Status::Mixed => "\x1B[1;33m~\x1B[0m",
            Status::Unsupported => "\x1B[0;90m-\x1B[0m",
            Status::Unknown => "\x1B[1;90m?\x1B[0m",
        }
    }

    pub const fn is_pending(&self) -> bool {
        matches!(self, Status::Initial | Status::Running { .. })
    }

    pub const fn has_failures(&self) -> bool {
        matches!(self, Status::Failed | Status::Mixed)
    }
}

#[derive(Clone, Debug)]
pub struct PuzzleVec<T> {
    min_year: Year,
    max_year: Year,
    vec: Vec<T>,
}

impl<T> PuzzleVec<T> {
    pub fn new(min_year: Year, max_year: Year, init_fn: impl Fn(Year, Day) -> T) -> Self {
        Self {
            min_year,
            max_year,
            vec: Puzzle::iter(min_year, max_year)
                .map(|(y, d)| init_fn(y, d))
                .collect(),
        }
    }

    #[inline]
    fn index(&self, year: Year, day: Day) -> usize {
        assert!(year >= self.min_year && year <= self.max_year);
        25 * (year.to_u16() as usize - self.min_year.to_u16() as usize) + (day.to_u8() as usize - 1)
    }

    pub fn puzzles(&self) -> impl FusedIterator<Item = (Year, Day)> + use<T> {
        Puzzle::iter(self.min_year, self.max_year)
    }

    pub fn iter(&self) -> impl FusedIterator<Item = ((Year, Day), &T)> {
        self.puzzles().zip(self.vec.iter())
    }
}

impl<T> Index<Year> for PuzzleVec<T> {
    type Output = [T];

    #[inline]
    fn index(&self, year: Year) -> &Self::Output {
        let index = self.index(year, Day::new_const::<1>());
        &self.vec[index..index + 25]
    }
}

impl<T> IndexMut<Year> for PuzzleVec<T> {
    #[inline]
    fn index_mut(&mut self, year: Year) -> &mut Self::Output {
        let index = self.index(year, Day::new_const::<1>());
        &mut self.vec[index..index + 25]
    }
}

impl<T> Index<(Year, Day)> for PuzzleVec<T> {
    type Output = T;

    #[inline]
    fn index(&self, (year, day): (Year, Day)) -> &Self::Output {
        &self.vec[self.index(year, day)]
    }
}

impl<T> IndexMut<(Year, Day)> for PuzzleVec<T> {
    #[inline]
    fn index_mut(&mut self, (year, day): (Year, Day)) -> &mut Self::Output {
        let index = self.index(year, day);
        &mut self.vec[index]
    }
}
