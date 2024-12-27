#![doc = include_str!("../README.md")]
#![cfg_attr(not(feature = "unsafe"), forbid(unsafe_code))]

mod knot_hash;

utils::year!(2017 => year2017, ${
    1 => day01::Day01<'_>,
    2 => day02::Day02,
    3 => day03::Day03,
    4 => day04::Day04<'_>,
    5 => day05::Day05,
    6 => day06::Day06,
    7 => day07::Day07<'_>,
    8 => day08::Day08,
    9 => day09::Day09,
    10 => day10::Day10<'_>,
    11 => day11::Day11,
    12 => day12::Day12,
    13 => day13::Day13,
    14 => day14::Day14,
    15 => day15::Day15,
    16 => day16::Day16,
    17 => day17::Day17,
    18 => day18::Day18,
    19 => day19::Day19,
    20 => day20::Day20,
    21 => day21::Day21,
    22 => day22::Day22,
    23 => day23::Day23,
});
