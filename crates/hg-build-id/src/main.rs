//! Prints the `BuildId` of a build of this checkout, as the line the
//! build passes to the compiler:
//!
//! ```text
//! cargo run -p hg-build-id -- --profile release --target x86_64-unknown-linux-gnu
//! IZCAL_BUILD_ID=1f0e…
//! ```
//!
//! Options: `--profile P` and `--target T` (required), `--channel C`
//! (default `dev`), `--version V` (default `0.0.0`), `--root DIR` (default
//! this checkout). `--unpinned` prints the value of a build made without
//! the variable.

use std::path::PathBuf;
use std::process::ExitCode;

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let value = |flag: &str| {
        args.iter()
            .position(|a| a == flag)
            .and_then(|i| args.get(i + 1))
            .filter(|v| !v.starts_with("--"))
            .cloned()
    };
    if args.iter().any(|a| a == "--unpinned") {
        println!("IZCAL_BUILD_ID={:016x}", hg_build_id::unpinned().build_id());
        return ExitCode::SUCCESS;
    }
    let (Some(profile), Some(target)) = (value("--profile"), value("--target")) else {
        eprintln!(
            "usage: hg-build-id --profile P --target T [--channel C] [--version V] [--root DIR] | --unpinned"
        );
        return ExitCode::FAILURE;
    };
    let channel = value("--channel").unwrap_or_else(|| "dev".into());
    let version = value("--version").unwrap_or_else(|| "0.0.0".into());
    let root = value("--root").map_or_else(
        || PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../.."),
        PathBuf::from,
    );
    match hg_build_id::collect(&root, &profile, &target, &channel, &version) {
        Ok(inputs) => {
            println!("IZCAL_BUILD_ID={:016x}", inputs.build_id());
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("hg-build-id: {e}");
            ExitCode::FAILURE
        }
    }
}
