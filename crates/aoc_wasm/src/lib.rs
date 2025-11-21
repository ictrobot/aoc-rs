//! Simple WebAssembly interface without external libraries.
mod custom_sections;
#[cfg(feature = "multithreading")]
mod multithreading;

use aoc::all_puzzles;
use aoc::utils::input::{InputType, strip_final_newline};
use std::error::Error;
use std::ffi::CStr;
use std::sync::Once;

const BUFFER_LENGTH: usize = 1024 * 1024;

#[unsafe(no_mangle)]
static INPUT: [u8; BUFFER_LENGTH] = [0u8; BUFFER_LENGTH];
#[unsafe(no_mangle)]
static mut PART1: [u8; BUFFER_LENGTH] = [0u8; BUFFER_LENGTH];
#[unsafe(no_mangle)]
static mut PART2: [u8; BUFFER_LENGTH] = [0u8; BUFFER_LENGTH];

static ONCE_PANIC_HANDLER: Once = Once::new();

#[unsafe(no_mangle)]
extern "C" fn run_puzzle(
    year: u16,
    day: u8,
    is_example: bool,
    run_part1: bool,
    run_part2: bool,
) -> bool {
    ONCE_PANIC_HANDLER.call_once(|| {
        std::panic::set_hook(Box::new(|info| {
            let Some(error) = info.payload_as_str() else {
                return;
            };

            let location = if let Some(location) = info.location() {
                // Trying to format the location may cause another panic, particularly on allocation
                // failure, but it should work on most explicit panics (e.g. "no solution found")
                &format!(
                    "{}:{}:{}",
                    location.file(),
                    location.line(),
                    location.column()
                )
            } else {
                ""
            };

            write_results(error, location);
        }));
    });

    // Clear result buffers before running the solution, so if it panics, the buffers either contain
    // the panic payload and location or are empty.
    write_results("", "");

    let (success, part1, part2) = match run(year, day, is_example, run_part1, run_part2) {
        Ok((part1, part2)) => (true, part1, part2),
        Err(err) => (false, err.to_string(), String::new()),
    };

    write_results(&part1, &part2);

    success
}

fn run(
    year: u16,
    day: u8,
    is_example: bool,
    run_part1: bool,
    run_part2: bool,
) -> Result<(String, String), Box<dyn Error>> {
    let input = strip_final_newline(CStr::from_bytes_until_nul(&INPUT)?.to_str()?);
    let input_type = if is_example {
        InputType::Example
    } else {
        InputType::Real
    };

    macro_rules! matcher {
        ($(
            $y:literal => $year:ident{$(
                $d:literal => $day:ident,
            )*}
        )*) => {
            match (year, day) {$($(
                ($y, $d) => {
                    let solution = aoc::$year::$day::new(input, input_type)?;
                    let part1 = if run_part1 { solution.part1().to_string() } else { String::new() };
                    let part2 = if run_part2 { solution.part2().to_string() } else { String::new() };
                    Ok((part1, part2))
                }
            )*)*
                _ => Err("unsupported puzzle".into()),
            }
        };
    }
    all_puzzles! {matcher}
}

fn write_results(part1: &str, part2: &str) {
    // SAFETY: No other Rust code accesses these variables or creates references - they're only read
    // from JS.
    unsafe {
        write_string((&raw mut PART1).cast(), part1);
        write_string((&raw mut PART2).cast(), part2);
    }
}

unsafe fn write_string(buf: *mut u8, str: &str) {
    let len = str.len().min(BUFFER_LENGTH - 1);
    unsafe {
        std::ptr::copy_nonoverlapping(str.as_ptr(), buf, len);
        *buf.add(len) = 0;
    }
}
