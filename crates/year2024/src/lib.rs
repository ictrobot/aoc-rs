#![doc = include_str!("../README.md")]
#![cfg_attr(not(feature = "unsafe"), forbid(unsafe_code))]

utils::year!(2024 => year2024, ${
    1 => day01::Day01,
    2 => day02::Day02,
    3 => day03::Day03,
});