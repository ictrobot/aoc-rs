use crate::cli::{Arguments, UsageError};
use std::env::current_exe;
use std::error::Error;
use std::ffi::OsString;
use utils::multiversion;

mod error;
mod manager;
mod mpmc;
mod oneshot;
mod output_grid;
mod process;
mod puzzle;
mod test_case;
mod thread;

pub fn main(args: &Arguments) -> Result<(), Box<dyn Error>> {
    if args.day.is_some() {
        return Err(UsageError::InvalidArguments(
            "specifying day is incompatible with --test".into(),
        )
        .into());
    }

    let cmd_template = get_cmd_template(args)?;

    let (min_year, max_year) = match args.year {
        Some(year) => (year, year),
        None => test_case::get_years(&args.inputs_dir())?.ok_or("no year directories found")?,
    };

    manager::Manager::run(min_year, max_year, cmd_template, &args.inputs_dir())?;

    Ok(())
}

fn get_cmd_template(args: &Arguments) -> Result<Vec<OsString>, Box<dyn Error>> {
    Ok(if args.extra_args.is_empty() {
        let mut cmd = vec![
            current_exe()
                .map_err(|e| format!("failed to get current executable: {e}"))?
                .into_os_string(),
            "--stdin".into(),
            "--threads".into(),
            "1".into(),
        ];
        if let Some(v) = multiversion::Version::get_override() {
            cmd.push("--multiversion".into());
            cmd.push(format!("{v:?}").into());
        }
        cmd.push("${YEAR}".into());
        cmd.push("${DAY}".into());
        cmd
    } else {
        // .skip(1) as placeholders aren't replaced in the executable path
        let cmd_args = args.extra_args.iter().skip(1);
        let has_year = cmd_args.clone().any(|x| x.contains("${YEAR}"));
        let has_day = cmd_args.clone().any(|x| x.contains("${DAY}"));

        if !has_year || !has_day {
            return Err(UsageError::InvalidArguments(
                "command template must contain ${YEAR} and ${DAY}".into(),
            )
            .into());
        }
        args.extra_args
            .iter()
            .map(std::convert::Into::into)
            .collect()
    })
}
