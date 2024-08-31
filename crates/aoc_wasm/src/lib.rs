//! Simple WebAssembly interface without external libraries.

mod custom_sections;

use aoc::all_puzzles;
use aoc::utils::input::InputType;
use std::error::Error;
use std::ffi::CStr;
use std::ptr::addr_of_mut;

const BUFFER_LENGTH: usize = 1024 * 1024;

#[no_mangle]
static INPUT: [u8; BUFFER_LENGTH] = [0u8; BUFFER_LENGTH];
#[no_mangle]
static mut PART1: [u8; BUFFER_LENGTH] = [0u8; BUFFER_LENGTH];
#[no_mangle]
static mut PART2: [u8; BUFFER_LENGTH] = [0u8; BUFFER_LENGTH];

#[no_mangle]
extern "C" fn run_puzzle(
    year: u16,
    day: u8,
    is_example: bool,
    run_part1: bool,
    run_part2: bool,
) -> bool {
    let (success, part1, part2) = match run(year, day, is_example, run_part1, run_part2) {
        Ok((part1, part2)) => (true, part1, part2),
        Err(err) => (false, err.to_string(), String::new()),
    };

    // SAFETY: No other Rust code accesses these variables or creates references - they're only read
    // from JS.
    unsafe {
        write_string(addr_of_mut!(PART1).cast(), &part1);
        write_string(addr_of_mut!(PART2).cast(), &part2);
    }

    success
}

fn run(
    year: u16,
    day: u8,
    is_example: bool,
    run_part1: bool,
    run_part2: bool,
) -> Result<(String, String), Box<dyn Error>> {
    let input = CStr::from_bytes_until_nul(&INPUT)?.to_str()?;
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

unsafe fn write_string(buf: *mut u8, str: &str) {
    let len = str.len().min(BUFFER_LENGTH - 1);
    std::ptr::copy_nonoverlapping(str.as_ptr(), buf, len);
    *buf.add(len) = 0;
}
