#![doc = include_str!("../README.md")]
#![cfg_attr(not(feature = "unsafe"), forbid(unsafe_code))]

utils::year!(2025 => year2025, ${
    1 => day01::Day01,
    2 => day02::Day02,
    3 => day03::Day03<'_>,
    4 => day04::Day04,
    5 => day05::Day05,
    6 => day06::Day06,
    7 => day07::Day07,
    8 => day08::Day08,
});
