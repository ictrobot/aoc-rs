use crate::cli::{Arguments, FailedNoErrorMessage, UsageError};
use std::process::ExitCode;
use utils::multithreading::set_thread_count;
use utils::multiversion::Version;

mod cli;

#[expect(clippy::print_stdout, clippy::print_stderr)]
fn main() -> ExitCode {
    let args = match Arguments::parse() {
        Ok(x) => x,
        Err(err) => {
            eprintln!("{err}");
            return UsageError::exit_code();
        }
    };

    if args.help {
        println!("{}", args.help_string());
        return ExitCode::SUCCESS;
    }

    if let Some(version) = args.version_override {
        Version::set_override(version);
    }
    if let Some(threads) = args.threads_override {
        set_thread_count(threads);
    }

    if let Err(err) = args.main_fn()(&args) {
        if err.downcast_ref::<FailedNoErrorMessage>().is_none() {
            eprintln!("{err}");
        }

        if err.downcast_ref::<UsageError>().is_some() {
            UsageError::exit_code()
        } else {
            ExitCode::FAILURE
        }
    } else {
        ExitCode::SUCCESS
    }
}
