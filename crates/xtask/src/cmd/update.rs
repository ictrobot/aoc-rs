use crate::common::{
    crate_dir_path, day_mod_name, day_struct_name, replace_in_file, year_create_name,
};
use std::error::Error;
use std::ffi::OsStr;
use std::fmt::Write;
use std::fs::{read_dir, read_to_string};
use std::path::Path;
use utils::date::{Day, Year};

pub fn main(args: impl Iterator<Item = String>) -> Result<(), Box<dyn Error>> {
    crate::ensure_no_args(args)?;

    let crates_dir = crate_dir_path();
    let years = find_years(&crates_dir)?;

    for &year in &years {
        let year_dir = crates_dir.join(year_create_name(year));
        update_year_lib_rs(&year_dir, year)?;
    }

    let aoc_dir = crates_dir.join("aoc");
    update_aoc_cargo_toml(&aoc_dir, &years)?;
    update_aoc_years_rs(&aoc_dir, &years)?;

    Ok(())
}

fn update_year_lib_rs(year_dir: &Path, year: Year) -> Result<(), Box<dyn Error>> {
    let src_dir = year_dir.join("src");
    let days = find_days(&src_dir)?;
    let lib_file = src_dir.join("lib.rs");

    let mut replacement = format!("{year:#} => {}, ${{\n", year_create_name(year));
    for &(day, uses_lifetime) in &days {
        writeln!(
            &mut replacement,
            "    {} => {}::{}{},",
            day.to_u8(),
            day_mod_name(day),
            day_struct_name(day),
            if uses_lifetime { "<'_>" } else { "" },
        )?;
    }
    replacement += "}";

    replace_in_file(&lib_file, "year!(", ")", &replacement)
}

fn update_aoc_cargo_toml(aoc_dir: &Path, years: &[Year]) -> Result<(), Box<dyn Error>> {
    let cargo_toml = aoc_dir.join("Cargo.toml");

    replace_in_file(&cargo_toml, "# xtask update dependencies", "\n\n", &{
        let mut replacement = String::new();
        for &year in years {
            let crate_name = year_create_name(year);
            write!(
                &mut replacement,
                "\n{crate_name} = {{ path = \"../{crate_name}\", optional = true }}"
            )?;
        }
        replacement
    })?;

    replace_in_file(&cargo_toml, "# xtask update features", "\n\n", &{
        let mut replacement = "\nall-years = [".to_string();
        replacement += &years
            .iter()
            .map(|&y| format!("\"{}\"", year_create_name(y)))
            .collect::<Vec<String>>()
            .join(", ");
        replacement += "]";
        replacement
    })?;

    Ok(())
}

fn update_aoc_years_rs(aoc_dir: &Path, years: &[Year]) -> Result<(), Box<dyn Error>> {
    let years_file = aoc_dir.join("src").join("puzzles.rs");

    replace_in_file(&years_file, "// xtask update all_puzzles", "\n\n", &{
        let mut replacement = String::new();
        for &year in years {
            write!(
                &mut replacement,
                "\n                $crate::puzzles::{},",
                year_create_name(year)
            )?;
        }
        replacement
    })?;

    replace_in_file(&years_file, "// xtask update re-exports", "\n\n", &{
        let mut year_reexports = String::new();
        let mut noop_reexports = String::new();
        for &year in years {
            let crate_name = year_create_name(year);

            write!(
                &mut year_reexports,
                r#"
#[cfg(feature = "{crate_name}")]
pub use ::{crate_name}::puzzles as {crate_name};"#
            )?;
            write!(
                &mut noop_reexports,
                r#"
#[cfg(not(feature = "{crate_name}"))]
pub use ::utils::puzzles_noop as {crate_name};"#
            )?;
        }

        // Order imports the same as rustfmt
        noop_reexports + &year_reexports
    })?;

    Ok(())
}

fn find_years(crates_dir: &Path) -> Result<Vec<Year>, Box<dyn Error>> {
    let mut years = Vec::new();
    for entry in read_dir(crates_dir)? {
        if let Some(year_num) = entry?
            .path()
            .file_name()
            .and_then(OsStr::to_str)
            .and_then(|s| s.strip_prefix("year"))
        {
            years.push(year_num.parse()?);
        }
    }
    years.sort_unstable();
    Ok(years)
}

fn find_days(src_dir: &Path) -> Result<Vec<(Day, bool)>, Box<dyn Error>> {
    let mut days = Vec::new();
    for entry in read_dir(src_dir)? {
        let path = entry?.path();
        if let Some(day_num) = path
            .file_name()
            .and_then(OsStr::to_str)
            .and_then(|s| s.strip_prefix("day"))
            .and_then(|s| s.strip_suffix(".rs"))
        {
            let day = day_num.parse()?;

            // Check if struct takes lifetime parameter
            let definition = format!("pub struct {}<'", day_struct_name(day));
            let contents = read_to_string(path)?;
            let uses_lifetime = contents.lines().any(|l| l.starts_with(&definition));

            days.push((day, uses_lifetime));
        }
    }
    days.sort_unstable();
    Ok(days)
}
