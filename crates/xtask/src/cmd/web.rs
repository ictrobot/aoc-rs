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

    create_dir(&output)?;

    run_cargo(&["clean", "--doc"], &[])?;
    run_cargo(&["doc", "--no-deps"], &[])?;

    let mut rewrites = vec![];
    for (env, extra_args, name) in [
        (&[][..], &[][..], "aoc.wasm"),
        (
            &[("RUSTFLAGS", "-C target_feature=+simd128")],
            &[],
            "aoc-simd128.wasm",
        ),
        // Experimental wasm threads support. This relies on a number of unstable and/or
        // undocumented features, including `target_feature=+`atomics`, `-Z build-std` and
        // `RUSTC_BOOTSTRAP=1` to re-use the same stable rust toolchain.
        (
            &[
                ("RUSTC_BOOTSTRAP", "1"),
                (
                    "RUSTFLAGS",
                    "-C target_feature=+atomics,+bulk-memory,+mutable-globals,+simd128 \
                        -C link-args=--export=__stack_pointer",
                ),
            ],
            &[
                "--features",
                "multithreading",
                "-Z",
                "build-std=panic_abort,std",
            ],
            "aoc-threads.wasm",
        ),
    ] {
        let mut args = vec![
            "build",
            "-p",
            "aoc_wasm",
            "--lib",
            "--target=wasm32-unknown-unknown",
            "--release",
        ];
        args.extend_from_slice(extra_args);

        run_cargo(&args, env)?;

        let output_wasm = output.join(name);
        copy_file(
            repo_dir_path()
                .join("target")
                .join("wasm32-unknown-unknown")
                .join("release")
                .join("aoc_wasm.wasm"),
            &output_wasm,
        )?;

        add_rewrite(&mut rewrites, &output_wasm)?;
    }

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

    // Don't rewrite service worker path
    copy_file(
        web.join("cross-origin-isolation-service-worker.js"),
        output.join("cross-origin-isolation-service-worker.js"),
    )?;

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
