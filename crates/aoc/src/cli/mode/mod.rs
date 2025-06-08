use crate::cli::Arguments;
use std::error::Error;

pub mod default;
pub mod stdin;
#[cfg(feature = "test-runner")]
pub mod test;

pub type MainFn = fn(&Arguments) -> Result<(), Box<dyn Error>>;
