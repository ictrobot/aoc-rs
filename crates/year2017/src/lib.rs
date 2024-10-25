#![doc = include_str!("../README.md")]
#![cfg_attr(not(feature = "unsafe"), forbid(unsafe_code))]

utils::year!(2017 => year2017, ${
    1 => day01::Day01<'_>,
    2 => day02::Day02,
    3 => day03::Day03,
    4 => day04::Day04<'_>,
});
