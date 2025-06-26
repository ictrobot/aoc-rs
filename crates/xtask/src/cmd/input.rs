use crate::common::{create_dir, day_mod_name, repo_dir_path, write_file, year_create_name};
use std::error::Error;
use std::fs::read_to_string;
use std::io::{ErrorKind, Write};
use std::path::Path;
use std::process::{Command, Stdio};
use std::time::SystemTime;
use std::{env, panic};
use utils::date::{Date, Day, Year};

// Follow UA format from https://www.reddit.com/r/adventofcode/comments/z9dhtd/please_include_your_contact_info_in_the_useragent/
const USER_AGENT: &str = concat!(
    env!("CARGO_PKG_REPOSITORY"),
    " by ",
    env!("CARGO_PKG_AUTHORS")
);

const TOKEN_VAR: &str = "AOC_TOKEN";
const TOKEN_FILE: &str = ".aoc_token";

#[cfg(not(target_os = "windows"))]
const HOME_VAR: &str = "HOME";
#[cfg(target_os = "windows")]
const HOME_VAR: &str = "USERPROFILE";

pub fn main(mut args: impl Iterator<Item = String>) -> Result<(), Box<dyn Error>> {
    let year = crate::year_arg(&mut args)?;
    let day = crate::day_arg(&mut args)?;
    crate::ensure_no_args(args)?;

    download(year, day)
}

pub fn download(year: Year, day: Day) -> Result<(), Box<dyn Error>> {
    if (Date { year, day }).release_time() > SystemTime::now() {
        return Err("puzzle is not released yet".into());
    }

    let input = fetch(year, day)?;

    let year_inputs_dir = repo_dir_path().join("inputs").join(year_create_name(year));
    if !year_inputs_dir.is_dir() {
        create_dir(&year_inputs_dir)?;
    }

    write_file(
        year_inputs_dir
            .join(day_mod_name(day))
            .with_extension("txt"),
        input,
    )
}

fn fetch(year: Year, day: Day) -> Result<String, Box<dyn Error>> {
    let token = read_session_token()?;
    if token.chars().any(|c| !c.is_ascii_alphanumeric()) {
        return Err("invalid session token".into());
    }

    let url = format!(
        "https://adventofcode.com/{}/day/{}/input",
        year.to_u16(),
        day.to_u8(),
    );
    println!("fetching {url}");

    // Use config provided to stdin to avoid leaking cookies via cli arguments
    let config = format!(
        r#"url "{url}"
user-agent "{USER_AGENT}"
cookie "session={token}"
silent
show-error
fail
proto "=https"
"#
    );

    let mut child = Command::new("curl")
        .args(["--config", "-"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| {
            if e.kind() == ErrorKind::NotFound {
                "curl was not found".to_string()
            } else {
                format!("running curl to download input failed: {e}")
            }
        })?;

    // Writing more than a pipe buffer's worth of data without reading stdout/stderr may cause
    // deadlocks, so use a thread to write to stdin (see std::process::Stdio docs)
    let mut stdin = child.stdin.take().ok_or("failed to open stdin")?;
    let handle = std::thread::spawn(move || stdin.write_all(config.as_bytes()));

    // Wait for the process to exit and collect stdout & stderr
    let output = child.wait_with_output()?;

    // Check thread wrote to stdin successfully
    match handle.join() {
        Err(e) => panic::resume_unwind(e), // Thread panicked, propagate error
        Ok(res) => res?,                   // Check the return value of write_all
    }

    match output.status.code() {
        Some(0) => Ok(String::from_utf8(output.stdout)?),
        Some(22) => {
            // Returned by --fail/--fail-with-body when status >= 400
            Err(format!(
                "unexpected HTTP status returned, try updating your session token: {}",
                String::from_utf8(output.stderr)?.trim(),
            )
            .into())
        }
        _ => {
            // Any other error
            Err(format!(
                "curl exited with code {}: {}",
                output.status,
                String::from_utf8(output.stderr)?.trim(),
            )
            .into())
        }
    }
}

fn read_session_token() -> Result<String, String> {
    if let Ok(token) = env::var(TOKEN_VAR) {
        return Ok(token);
    }
    if let Ok(token) = read_to_string(TOKEN_FILE) {
        return Ok(token.trim_end().to_string());
    }
    if let Ok(home) = env::var(HOME_VAR)
        && let Ok(token) = read_to_string(Path::new(&home).join(TOKEN_FILE))
    {
        return Ok(token.trim_end().to_string());
    }

    Err(format!(
        "failed to read session token from any of the following:
- the {TOKEN_VAR} environment variable
- the {TOKEN_FILE} file in the current directory
- the {TOKEN_FILE} file in your home directory"
    ))
}
