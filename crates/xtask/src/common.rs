use std::error::Error;
use std::fs::{create_dir_all, read_to_string, remove_dir_all, write};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::{env, fs};
use utils::date::{Day, Year};

pub fn create_dir(path: impl AsRef<Path>) -> Result<(), Box<dyn Error>> {
    println!(
        "creating {}",
        path.as_ref()
            .strip_prefix(repo_dir_path())?
            .to_string_lossy()
    );
    create_dir_all(path)?;
    Ok(())
}

pub fn delete_dir(path: impl AsRef<Path>) -> Result<(), Box<dyn Error>> {
    println!(
        "deleting {}",
        path.as_ref()
            .strip_prefix(repo_dir_path())?
            .to_string_lossy()
    );
    remove_dir_all(path)?;
    Ok(())
}

pub fn copy_dir(src: impl AsRef<Path>, dst: impl AsRef<Path>) -> Result<(), Box<dyn Error>> {
    create_dir(dst.as_ref())?;

    for entry_result in src.as_ref().read_dir()? {
        let entry = entry_result?;
        let file_type = entry.file_type()?;
        let src_child = entry.path();
        let dst_child = dst.as_ref().join(entry.file_name());
        if file_type.is_file() {
            copy_file(src_child, dst_child)?;
        } else if file_type.is_dir() {
            copy_dir(&src_child, &dst_child)?;
        }
    }

    Ok(())
}

pub fn copy_file(src: impl AsRef<Path>, dst: impl AsRef<Path>) -> Result<(), Box<dyn Error>> {
    println!(
        "copying {} to {}",
        src.as_ref()
            .strip_prefix(repo_dir_path())?
            .to_string_lossy(),
        dst.as_ref()
            .strip_prefix(repo_dir_path())?
            .to_string_lossy(),
    );

    fs::copy(src, dst)?;

    Ok(())
}

pub fn write_file(
    path: impl AsRef<Path>,
    contents: impl AsRef<[u8]>,
) -> Result<(), Box<dyn Error>> {
    println!(
        "writing {}",
        path.as_ref()
            .strip_prefix(repo_dir_path())?
            .to_string_lossy()
    );
    write(path, contents)?;
    Ok(())
}

pub fn replace_in_file(
    file: &Path,
    start: &str,
    end: &str,
    replacement: &str,
) -> Result<(), Box<dyn Error>> {
    let content = read_to_string(file)?.replace("\r\n", "\n");
    let (prefix, suffix) = content
        .split_once(start)
        .ok_or_else(|| format!("`{start}` not found in {}", file.to_string_lossy()))?;
    let (_, suffix) = suffix
        .split_once(end)
        .ok_or_else(|| format!("`{end}` not found in {}", file.to_string_lossy()))?;

    let mut new_content = prefix.to_string();
    new_content += start;
    new_content += replacement;
    new_content += end;
    new_content += suffix;

    if content != new_content {
        println!(
            "updating {} between {start:?} and {end:?}",
            file.strip_prefix(repo_dir_path())?.to_string_lossy()
        );
        write(file, new_content)?;
    }

    Ok(())
}

#[must_use]
pub fn crate_dir_path() -> PathBuf {
    let mut dir: PathBuf = cargo_var("CARGO_MANIFEST_DIR").into();
    assert!(dir.pop(), "expected manifest directory to have a parent");
    assert!(dir.is_dir(), "expected crates directory to be a directory");
    dir
}

#[must_use]
pub fn repo_dir_path() -> PathBuf {
    let mut dir = crate_dir_path();
    assert!(dir.pop(), "expected crates directory to have a parent");
    assert!(dir.is_dir(), "expected repo directory to be a directory");
    dir
}

#[must_use]
pub fn year_create_name(year: Year) -> String {
    format!("year{year:#}")
}

#[must_use]
pub fn day_mod_name(day: Day) -> String {
    format!("day{day:#}")
}

#[must_use]
pub fn day_struct_name(day: Day) -> String {
    format!("Day{day:#}")
}

// Check at runtime instead of compile time in case the workspace has been moved since compilation
// https://doc.rust-lang.org/cargo/reference/environment-variables.html#environment-variables-cargo-sets-for-crates
fn cargo_var(key: &str) -> String {
    env::var(key)
        .unwrap_or_else(|_| panic!("expected {key} environment variable to be set by cargo"))
}

pub fn run_cargo(args: &[&str]) -> Result<(), Box<dyn Error>> {
    println!("running cargo with {args:?}");
    let status = Command::new(cargo_var("CARGO")).args(args).status()?;
    if status.success() {
        Ok(())
    } else {
        Err(format!("command exited with {status}").into())
    }
}
