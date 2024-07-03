use crate::common::{
    crate_dir_path, create_dir, day_mod_name, day_struct_name, write_file, year_create_name,
};
use std::error::Error;

pub fn main(mut args: impl Iterator<Item = String>) -> Result<(), Box<dyn Error>> {
    let year = crate::year_arg(&mut args)?;
    let day = crate::day_arg(&mut args)?;
    crate::ensure_no_args(args)?;

    let crate_name = year_create_name(year);
    let crate_dir = crate_dir_path().join(&crate_name);
    let src_dir = crate_dir.join("src");

    if !crate_dir.exists() {
        create_dir(&crate_dir)?;

        write_file(
            crate_dir.join("README.md"),
            format!("Solutions for [Advent of Code {year:#}](https://adventofcode.com/{year:#})\n"),
        )?;

        write_file(
            crate_dir.join("Cargo.toml"),
            format!(
                r#"[package]
name = "{crate_name}"
authors = {{ workspace = true }}
edition = {{ workspace = true }}
license = {{ workspace = true }}
publish = {{ workspace = true }}
repository = {{ workspace = true }}

[dependencies]
utils = {{ path = "../utils" }}
"#
            ),
        )?;

        create_dir(&src_dir)?;

        write_file(
            src_dir.join("lib.rs"),
            format!(
                r#"#![doc = include_str!("../README.md")]

utils::year!({year:#} => {{}});"#
            ),
        )?;

        create_dir(crate_dir.join("examples"))?;
    }

    let mod_name = day_mod_name(day);
    let day_file = src_dir.join(mod_name.clone() + ".rs");
    assert!(
        !day_file.exists(),
        "{} already exists",
        day_file.to_string_lossy()
    );

    let struct_name = day_struct_name(day);
    write_file(
        day_file,
        format!(
            r#"use utils::prelude::*;

/// TODO
#[derive(Clone, Debug)]
pub struct {struct_name} {{
    input: String,
}}

impl {struct_name} {{
    pub fn new(input: &str, _: InputType) -> Result<Self, InvalidInputError> {{
        Ok(Self{{input: input.to_string()}})
    }}

    #[must_use]
    pub fn part1(&self) -> u64 {{
        0
    }}

    #[must_use]
    pub fn part2(&self) -> u64 {{
        0
    }}
}}

examples!({struct_name}<u64, u64> => [
    "..." part1=0 part2=0,
]);"#
        ),
    )?;

    super::update::main(None.into_iter())
}
