use crate::cli::mode::test::puzzle::{PuzzleVec, Status};
use std::cmp::Ordering;
use std::io::{self, Write};
use std::time::Duration;
use utils::date::{Day, Year};

const SPINNER: &[&str] = &["⠋", "⠙", "⠸", "⠴", "⠦", "⠇"];
pub const SPINNER_INTERVAL: Duration = Duration::from_millis(100);
pub const UPDATE_INTERVAL: Duration = Duration::from_nanos(1_000_000_000 / 60);

pub struct OutputGrid<W: Write> {
    min_year: Year,
    max_year: Year,
    out: W,
    cursor_row: usize,
    cursor_col: usize,
    statuses: PuzzleVec<Status>,
}

impl<W: Write> OutputGrid<W> {
    pub fn new(min_year: Year, max_year: Year, out: W) -> io::Result<Self> {
        let mut grid = OutputGrid {
            min_year,
            max_year,
            out,
            cursor_row: 0,
            cursor_col: 0,
            statuses: PuzzleVec::new(min_year, max_year, |_, _| Status::default()),
        };

        grid.print_grid()?;

        Ok(grid)
    }

    pub fn update(&mut self, year: Year, day: Day, status: Status) -> io::Result<()> {
        if self.statuses[(year, day)] != status {
            self.statuses[(year, day)] = status;
            self.redraw(year, day)?;
        }
        Ok(())
    }

    pub fn update_spinners(&mut self) -> io::Result<()> {
        for (year, day) in self.statuses.puzzles() {
            if matches!(self.statuses[(year, day)], Status::Running { .. }) {
                self.redraw(year, day)?;
            }
        }
        Ok(())
    }

    pub fn set_pending_to_unknown(&mut self) -> io::Result<()> {
        for (year, day) in self.statuses.puzzles() {
            if self.statuses[(year, day)].is_pending() {
                self.update(year, day, Status::Unknown)?;
            }
        }
        Ok(())
    }

    pub fn flush(&mut self) -> io::Result<()> {
        self.return_to_end()?;
        self.out.flush()
    }

    fn print_grid(&mut self) -> io::Result<()> {
        write!(self.out, "    ")?;
        for d in 1..=25 {
            write!(self.out, " {d:02}")?;
        }
        writeln!(self.out)?;

        for y in self.min_year.to_u16()..=self.max_year.to_u16() {
            write!(self.out, "{y}")?;

            for _ in 1..=25 {
                write!(self.out, "  {}", Self::symbol(Status::Initial))?;
            }

            if y != self.max_year.to_u16() {
                writeln!(self.out)?;
            }
        }

        (self.cursor_row, self.cursor_col) = self.position(self.max_year, Day::new_const::<25>());
        self.out.flush()
    }

    fn position(&self, year: Year, day: Day) -> (usize, usize) {
        (
            year.to_u16() as usize - self.min_year.to_u16() as usize,
            4 + 3 * (day.to_u8() as usize - 1),
        )
    }

    fn return_to_end(&mut self) -> io::Result<()> {
        let (row, col) = self.position(self.max_year, Day::new_const::<25>());
        self.move_cursor_to(row, col)
    }

    fn move_cursor_to(&mut self, row: usize, col: usize) -> io::Result<()> {
        match self.cursor_row.cmp(&row) {
            Ordering::Greater => write!(self.out, "\x1B[{}A", self.cursor_row - row)?,
            Ordering::Less => write!(self.out, "\x1B[{}B", row - self.cursor_row)?,
            Ordering::Equal => {}
        }
        self.cursor_row = row;

        match self.cursor_col.cmp(&col) {
            Ordering::Greater => write!(self.out, "\x1B[{}D", self.cursor_col - col)?,
            Ordering::Less => write!(self.out, "\x1B[{}C", col - self.cursor_col)?,
            Ordering::Equal => {}
        }
        self.cursor_col = col;

        Ok(())
    }

    fn redraw(&mut self, year: Year, day: Day) -> io::Result<()> {
        let status = self.statuses[(year, day)];

        let (row, col) = self.position(year, day);
        self.move_cursor_to(row, col - 1)?;

        write!(self.out, "{}", Self::symbol(status))?;
        self.cursor_col += 1;

        Ok(())
    }

    fn symbol(status: Status) -> &'static str {
        if let Status::Running { started } = status {
            let age = (started.elapsed().as_nanos() / SPINNER_INTERVAL.as_nanos()) as usize;
            SPINNER[age % SPINNER.len()]
        } else {
            status.symbol()
        }
    }
}

impl<W: Write> Drop for OutputGrid<W> {
    fn drop(&mut self) {
        let _ = self
            .return_to_end()
            .and_then(|()| writeln!(self.out))
            .and_then(|()| self.out.flush());
    }
}
