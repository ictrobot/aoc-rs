use std::error::Error;
use std::io::{self, Write};
use std::thread::sleep;
use std::time::{Duration, SystemTime};
use utils::date::Date;

const ANSI_LINE_START: &str = "\x1B[0G";
const ANSI_CLEAR_LINE: &str = "\x1B[2K";

pub fn main(args: impl Iterator<Item = String>) -> Result<(), Box<dyn Error>> {
    let mut args = args.peekable();

    let date = if args.peek().is_none() {
        Date::next_puzzle().unwrap()
    } else {
        let year = crate::year_arg(&mut args)?;
        let day = crate::day_arg(&mut args)?;
        crate::ensure_no_args(args)?;
        Date { year, day }
    };
    let release = date.release_time();

    let mut previous = (0, 0, 0, 61);
    while let Ok(remaining) = release.duration_since(SystemTime::now()) {
        let days = remaining.as_secs() / 86400;
        let hours = (remaining.as_secs() % 86400) / 3600;
        let minutes = (remaining.as_secs() % 3600) / 60;
        let seconds = remaining.as_secs() % 60;

        let current = (days, hours, minutes, seconds);
        if previous != current {
            previous = current;

            print!(
                "{ANSI_LINE_START}{ANSI_CLEAR_LINE}{date} will be released in \
                    {days} day{}, {hours} hour{}, {minutes} minute{} and {seconds} second{}",
                if days == 1 { "" } else { "s" },
                if hours == 1 { "" } else { "s" },
                if minutes == 1 { "" } else { "s" },
                if seconds == 1 { "" } else { "s" },
            );
            io::stdout().flush()?;
        }

        sleep(Duration::from_millis(50));
    }

    println!("{ANSI_LINE_START}{ANSI_CLEAR_LINE}{date} is released!");

    super::input::download(date.year, date.day)
}
