use crate::cli::FailedNoErrorMessage;
use crate::cli::mode::test::output_grid::{OutputGrid, SPINNER_INTERVAL, UPDATE_INTERVAL};
use crate::cli::mode::test::process::{ProcessEvent, ProcessPool};
use crate::cli::mode::test::puzzle::{Puzzle, PuzzleVec, Status};
use crate::cli::mode::test::test_case::{TestCase, TestCases};
use std::collections::BTreeSet;
use std::error::Error;
use std::ffi::OsString;
use std::io::{StdoutLock, stdout};
use std::path::Path;
use std::process::Command;
use std::time::{Duration, Instant};
use std::{io, iter};
use utils::date::{Day, Year};
use utils::multithreading;

const CMD_TIMEOUT: Duration = Duration::from_secs(10);

#[derive(Debug)]
pub struct Manager {
    cmd_template: Vec<OsString>,
    test_cases: TestCases,
    process_pool: ProcessPool<(Year, Day), (Year, Day, TestCase)>,
    pending_updates: BTreeSet<(Year, Day)>,
    puzzles: PuzzleVec<Puzzle>,
}

impl Manager {
    pub fn run(
        min_year: Year,
        max_year: Year,
        cmd_template: Vec<OsString>,
        input_dir: &Path,
    ) -> Result<(), Box<dyn Error>> {
        let mut manager = Self {
            cmd_template,
            test_cases: TestCases::new(input_dir, min_year, max_year)?,
            process_pool: ProcessPool::new(multithreading::get_thread_count())?,
            pending_updates: BTreeSet::new(),
            puzzles: PuzzleVec::new(min_year, max_year, |_, _| Puzzle::default()),
        };
        let mut stdout = stdout().lock();
        let mut grid = OutputGrid::new(min_year, max_year, &mut stdout)?;

        if let Err(err) = manager.main_loop(&mut grid) {
            // Try to replace any non-final status with Unknown and drop the grid to reset the
            // cursor. This may not succeed depending on the type of the previous error.
            let _ = grid.set_pending_to_unknown();
            return Err(err.into());
        }

        // Finalize the grid and reset the cursor
        manager.update_grid(&mut grid)?;
        drop(grid);

        manager.print_summary();
        manager.return_value()
    }

    fn main_loop(&mut self, grid: &mut OutputGrid<&mut StdoutLock>) -> io::Result<()> {
        let mut next_spinner_tick = Instant::now() + SPINNER_INTERVAL;
        let mut next_update = Instant::now();

        while !self.test_cases.is_done() || self.process_pool.pending_results() > 0 {
            let now = Instant::now();

            if now >= next_spinner_tick {
                grid.update_spinners()?;
                next_spinner_tick += SPINNER_INTERVAL;
                next_update = next_update.min(now);
            }
            if !self.pending_updates.is_empty() && now >= next_update {
                self.update_grid(grid)?;
                if next_update + UPDATE_INTERVAL < now {
                    next_update = now + UPDATE_INTERVAL;
                } else {
                    next_update += UPDATE_INTERVAL;
                }
            }
            grid.flush()?;

            self.enqueue_processes()?;
            self.process_result(
                if self.pending_updates.is_empty() {
                    next_spinner_tick
                } else {
                    next_update.min(next_spinner_tick)
                }
                .saturating_duration_since(now),
            )?;
        }

        Ok(())
    }

    fn update_grid(&mut self, grid: &mut OutputGrid<&mut StdoutLock>) -> io::Result<()> {
        for &(year, day) in &self.pending_updates {
            grid.update(year, day, self.puzzles[(year, day)].get_status())?;
        }
        self.pending_updates.clear();
        Ok(())
    }

    fn enqueue_processes(&mut self) -> io::Result<()> {
        if self.test_cases.is_done() {
            self.process_pool.close();
            return Ok(());
        }

        while self.process_pool.pending_results() <= self.process_pool.max_processes() * 2 {
            let Some((year, day, test_cases)) = self.test_cases.next()? else {
                break;
            };

            if self.puzzles[(year, day)].set_case_count(test_cases.len()) {
                self.pending_updates.insert((year, day));
            }

            for case in test_cases {
                self.enqueue_process(year, day, case);
            }
        }

        Ok(())
    }

    fn enqueue_process(&mut self, year: Year, day: Day, test_case: TestCase) {
        let mut cmd = Command::new(&self.cmd_template[0]);
        cmd.args(self.cmd_template[1..].iter().map(|s| {
            // Only replace the placeholders in valid utf-8 strings. This is safe because:
            // - Placeholder strings in the default command template are always valid utf-8
            // - Placeholder strings in custom command templates are first parsed as arguments, which
            //   requires valid utf-8
            if let Some(s) = s.to_str() {
                s.replace("${YEAR}", &year.to_u16().to_string())
                    .replace("${DAY}", &day.to_u8().to_string())
                    .into()
            } else {
                s.clone()
            }
        }));

        self.process_pool.enqueue(
            cmd,
            test_case.input.clone(),
            CMD_TIMEOUT,
            (year, day),
            (year, day, test_case),
        );
    }

    fn process_result(&mut self, timeout: Duration) -> io::Result<()> {
        if let Some(result) = self.process_pool.recv_timeout(timeout) {
            match result {
                ProcessEvent::Started((year, day)) => {
                    if self.puzzles[(year, day)].case_started() {
                        self.pending_updates.insert((year, day));
                    }
                }
                ProcessEvent::Finished((year, day, test_case), result) => {
                    if self.puzzles[(year, day)].case_finished(test_case, result?)? {
                        self.pending_updates.insert((year, day));
                    }
                }
            }
        }
        Ok(())
    }

    #[expect(clippy::print_stdout)]
    fn print_summary(&self) {
        println!("\nSummary:");

        let mut puzzles = self.puzzles.puzzles().peekable();
        let mut last_year = None;
        while let Some((year, day)) = puzzles.next() {
            let status = self.puzzles[(year, day)].get_status();

            // Group years together where all puzzles have matching groupable statuses
            if day == Day::new_const::<1>() && !status.has_failures() {
                let mut max_year = None;
                let mut succeeded = 0;
                let mut total = 0;
                while let Some(&(y, _)) = puzzles.peek()
                    && self.puzzles[y].iter().all(|p| p.get_status() == status)
                {
                    max_year = Some(y);
                    for p in &self.puzzles[y] {
                        succeeded += p.get_succeeded();
                        total += p.get_case_count();
                    }
                    take_while_peeking(&mut puzzles, |&(y2, _)| y == y2).for_each(drop);
                }
                if let Some(max_year) = max_year {
                    if year == max_year {
                        println!(
                            "{} {year} {status:?} ({succeeded}/{total})",
                            status.symbol()
                        );
                    } else {
                        println!(
                            "{} {year}-{max_year:#} {status:?} ({succeeded}/{total})",
                            status.symbol()
                        );
                    }
                    continue;
                }
            }

            if last_year != Some(year) {
                println!("{} {year}", Status::Mixed.symbol());
                last_year = Some(year);
            }

            let mut succeeded = self.puzzles[(year, day)].get_succeeded();
            let mut total = self.puzzles[(year, day)].get_case_count();
            let failures = self.puzzles[(year, day)].get_failures();

            // Print each failure, never group
            if !failures.is_empty() && !failures.iter().all(|(_, e)| e.is_unsupported_puzzle()) {
                println!(
                    "  {} {day} {status:?} ({succeeded}/{total})",
                    status.symbol()
                );
                for (path, err) in failures {
                    println!("    {} {}: {err}", Status::Failed.symbol(), path.display());
                }
                continue;
            }

            // Group days with the same status and without failures together
            let mut max_day = None;
            for (y, d) in take_while_peeking(&mut puzzles, |&(y, d)| {
                y == year && self.puzzles[(y, d)].get_status() == status
            }) {
                max_day = Some(d);
                succeeded += self.puzzles[(y, d)].get_succeeded();
                total += self.puzzles[(y, d)].get_case_count();
            }
            if let Some(max_day) = max_day {
                println!(
                    "  {} {day}-{max_day:#} {status:?} ({succeeded}/{total})",
                    status.symbol()
                );
            } else {
                println!(
                    "  {} {day} {status:?} ({succeeded}/{total})",
                    status.symbol()
                );
            }
        }
    }

    fn return_value(&self) -> Result<(), Box<dyn Error>> {
        // Exit with failure if any tests failed (excluding unsupported errors), or if no tests
        // succeeded
        let mut any_succeeded = false;
        for (_, p) in self.puzzles.iter() {
            any_succeeded |= p.get_succeeded() > 0;
            for (_, e) in p.get_failures() {
                if !e.is_unsupported_puzzle() {
                    return Err(FailedNoErrorMessage.into());
                }
            }
        }
        if any_succeeded {
            Ok(())
        } else {
            Err(FailedNoErrorMessage.into())
        }
    }
}

fn take_while_peeking<T>(
    iter: &mut iter::Peekable<impl Iterator<Item = T>>,
    mut pred: impl FnMut(&T) -> bool,
) -> impl Iterator<Item = T> {
    iter::from_fn(move || {
        if pred(iter.peek()?) {
            Some(iter.next().unwrap())
        } else {
            None
        }
    })
}
