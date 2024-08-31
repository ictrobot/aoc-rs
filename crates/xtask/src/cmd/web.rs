use crate::common::{copy_dir, copy_file, delete_dir, repo_dir_path, run_cargo};
use std::error::Error;

pub fn main(args: impl Iterator<Item = String>) -> Result<(), Box<dyn Error>> {
    crate::ensure_no_args(args)?;

    let mut output = repo_dir_path();
    output.push("target");
    output.push("web");

    if output.exists() {
        delete_dir(&output)?;
    }

    run_cargo(&["clean", "--doc"])?;
    run_cargo(&["doc", "--no-deps"])?;
    run_cargo(&[
        "build",
        "-p",
        "aoc_wasm",
        "--lib",
        "--target=wasm32-unknown-unknown",
        "--release",
    ])?;

    copy_dir(
        repo_dir_path().join("crates").join("aoc_wasm").join("web"),
        &output,
    )?;
    copy_file(
        repo_dir_path()
            .join("target")
            .join("wasm32-unknown-unknown")
            .join("release")
            .join("aoc_wasm.wasm"),
        output.join("aoc_wasm.wasm"),
    )?;
    copy_dir(
        repo_dir_path().join("target").join("doc"),
        output.join("doc"),
    )?;

    Ok(())
}
