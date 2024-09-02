use crate::common::{
    copy_dir, copy_file, create_dir, delete_dir, repo_dir_path, run_cargo, write_file,
};
use std::error::Error;
use std::fs;
use std::path::Path;
use std::str;
use utils::md5;

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

    create_dir(&output)?;

    let output_wasm = output.join("aoc.wasm");
    copy_file(
        repo_dir_path()
            .join("target")
            .join("wasm32-unknown-unknown")
            .join("release")
            .join("aoc_wasm.wasm"),
        &output_wasm,
    )?;

    let mut rewrites = vec![];
    add_rewrite(&mut rewrites, &output_wasm)?;

    let web = repo_dir_path().join("crates").join("aoc_wasm").join("web");
    for file in ["aoc.mjs", "worker.mjs", "web.mjs", "index.html"] {
        let src = web.join(file);
        let dst = output.join(file);

        let mut contents = fs::read_to_string(src)?;
        for (from, to) in &rewrites {
            let len = contents.len();
            contents = contents.replace(from, to);
            if len != contents.len() {
                println!("rewritten {from} to {to} in {file}");
            }
        }
        write_file(&dst, contents.as_bytes())?;

        add_rewrite(&mut rewrites, &dst)?;
    }

    copy_dir(
        repo_dir_path().join("target").join("doc"),
        output.join("doc"),
    )?;

    Ok(())
}

fn add_rewrite(rewrites: &mut Vec<(String, String)>, path: &Path) -> Result<(), Box<dyn Error>> {
    let bytes = fs::read(path)?;
    let hash_hex = md5::to_hex(md5::hash(&bytes));
    let hex_str = str::from_utf8(&hash_hex)?;

    let Some(file_name) = path.file_name().and_then(|x| x.to_str()) else {
        return Err("invalid file name".into());
    };
    let from = format!("\"./{file_name}\"");
    let to = format!("\"./{file_name}?hash={hex_str}\"");
    rewrites.push((from, to));

    Ok(())
}
