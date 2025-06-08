use crate::cli::mode::test::puzzle::Puzzle;
use crate::cli::mode::test::thread::AutoJoinHandle;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};
use std::sync::mpsc::{Receiver, SyncSender, TryRecvError, sync_channel};
use std::thread::Builder;
use std::{fs, io};
use utils::date::{Day, Year};

// Using multiple threads to read inputs makes a big difference if the inputs aren't cached, i.e.
// on the first run or after running `echo 3 | sudo tee /proc/sys/vm/drop_caches`.
const THREADS: usize = 8;
const CHANNEL_BOUND: usize = 1;

#[derive(Debug)]
pub struct TestCases {
    next: usize,
    receivers: Vec<Receiver<TestCasesWorkerResult>>,
    handles: Vec<AutoJoinHandle<()>>,
}

type TestCasesWorkerResult = (Year, Day, io::Result<Vec<TestCase>>);

impl TestCases {
    pub fn new(input_dir: &Path, min_year: Year, max_year: Year) -> io::Result<Self> {
        let mut receivers = Vec::with_capacity(THREADS);
        let mut handles = Vec::with_capacity(THREADS);
        for i in 0..THREADS {
            // Use sync channels to avoid reading too many test cases before they are needed,
            // increasing memory usage.
            let (sender, receiver) = sync_channel(CHANNEL_BOUND);

            receivers.push(receiver);
            handles.push(
                Builder::new()
                    .name(format!("case-reader-{i}"))
                    .spawn(Self::worker(
                        input_dir.to_owned(),
                        min_year,
                        max_year,
                        sender,
                        i,
                    ))?
                    .into(),
            );
        }

        Ok(Self {
            next: 0,
            receivers,
            handles,
        })
    }

    pub fn next(&mut self) -> io::Result<Option<(Year, Day, Vec<TestCase>)>> {
        if self.receivers.is_empty() {
            return Ok(None);
        }

        let result = match self.receivers[self.next % THREADS].try_recv() {
            Ok((year, day, Ok(cases))) => {
                // Only advance to the next receiver once a value is received. This ensures the test
                // cases are returned in (Year, Day) order.
                self.next += 1;
                return Ok(Some((year, day, cases)));
            }
            Ok((_, _, Err(err))) => Err(err),
            Err(TryRecvError::Empty) => return Ok(None),
            Err(TryRecvError::Disconnected) => Ok(None),
        };

        // There are no more test cases or a worker thread encountered an error. Clean up by
        // dropping the receivers and joining the worker threads. Assign Vec::new() to also drop
        // the backing allocations.
        self.receivers = Vec::new();
        self.handles = Vec::new();

        result
    }

    pub fn is_done(&self) -> bool {
        self.receivers.is_empty()
    }

    fn worker(
        input_dir: PathBuf,
        min_year: Year,
        max_year: Year,
        sender: SyncSender<TestCasesWorkerResult>,
        worker_num: usize,
    ) -> impl FnOnce() + Send + 'static {
        move || {
            for (year, day) in Puzzle::iter(min_year, max_year)
                .skip(worker_num)
                .step_by(THREADS)
            {
                match read_test_cases(&input_dir, year, day) {
                    Ok(cases) => {
                        if sender.send((year, day, Ok(cases))).is_err() {
                            return; // The receiver has gone away
                        }
                    }
                    Err(err) => {
                        let _ = sender.send((year, day, Err(err)));
                        return;
                    }
                }
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct TestCase {
    pub input: String,
    pub part1: String,
    pub part2: Option<String>,
    pub input_path: PathBuf,
}

pub fn get_years(inputs_dir: &Path) -> io::Result<Option<(Year, Year)>> {
    let mut min_max = None;
    for entry in fs::read_dir(inputs_dir)? {
        let entry = entry?;
        let name = entry.file_name();
        let Some(name) = name.to_str() else {
            continue;
        };
        let Some(year_num) = name.strip_prefix("year") else {
            continue;
        };
        if year_num.chars().any(|c| !c.is_ascii_digit()) || year_num.len() != 4 {
            continue;
        }
        let Ok(year) = year_num.parse::<Year>() else {
            continue;
        };

        min_max = match min_max {
            None => Some((year, year)),
            Some((min, max)) => Some((min.min(year), max.max(year))),
        };
    }
    Ok(min_max)
}

pub fn read_test_cases(inputs_dir: &Path, year: Year, day: Day) -> io::Result<Vec<TestCase>> {
    let mut cases = Vec::new();
    let mut try_add_case =
        |mut input_path: PathBuf, part1_path: PathBuf, part2_path: PathBuf| -> io::Result<()> {
            let Some(input) = read_file(&input_path)? else {
                return Ok(());
            };
            let Some(part1) = read_file(&part1_path)? else {
                return Ok(());
            };
            let part2 = read_file(&part2_path)?;

            // Convert absolute paths into paths relative to the input dir where possible
            if let Ok(p) = input_path.strip_prefix(inputs_dir) {
                input_path = p.to_path_buf();
            }

            cases.push(TestCase {
                input,
                part1,
                part2,
                input_path,
            });
            Ok(())
        };

    // Handles:
    //  ${input_dir}/year${year}/day${day}.txt
    //  ${input_dir}/year${year}/day${day}-part1.txt
    //  ${input_dir}/year${year}/day${day}-part2.txt
    let year_dir = inputs_dir.join(format!("year{year:#}"));
    try_add_case(
        year_dir.join(format!("day{day:#}.txt")),
        year_dir.join(format!("day{day:#}-part1.txt")),
        year_dir.join(format!("day{day:#}-part2.txt")),
    )?;

    // Handles:
    //  ${input_dir}/year${year}/day${day}/input.txt
    //  ${input_dir}/year${year}/day${day}/part1.txt
    //  ${input_dir}/year${year}/day${day}/part2.txt
    let day_dir = year_dir.join(format!("day{day:#}"));
    try_add_case(
        day_dir.join("input.txt"),
        day_dir.join("part1.txt"),
        day_dir.join("part2.txt"),
    )?;

    // Handles:
    //  ${input_dir}/year${year}/day${day}/${identifier}/input.txt
    //  ${input_dir}/year${year}/day${day}/${identifier}/part1.txt
    //  ${input_dir}/year${year}/day${day}/${identifier}/part2.txt
    // Useful for extra examples or community test cases
    match fs::read_dir(day_dir) {
        Ok(entries) => {
            for entry in entries {
                let entry = entry?;

                // Only skip normal files. Try to read test cases from directories and symlinks
                if entry.file_type()?.is_file() {
                    continue;
                }

                let entry_path = entry.path();
                try_add_case(
                    entry_path.join("input.txt"),
                    entry_path.join("part1.txt"),
                    entry_path.join("part2.txt"),
                )?;
            }
        }
        Err(err) => match err.kind() {
            ErrorKind::NotFound | ErrorKind::NotADirectory => {}
            _ => return Err(err),
        },
    }

    Ok(cases)
}

fn read_file(path: &Path) -> io::Result<Option<String>> {
    match fs::read_to_string(path) {
        Ok(s) => Ok(Some(s)),
        Err(err) => match err.kind() {
            ErrorKind::NotFound | ErrorKind::NotADirectory | ErrorKind::IsADirectory => Ok(None),
            _ => Err(err),
        },
    }
}
