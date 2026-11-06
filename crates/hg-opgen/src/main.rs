//! The `hg-opgen` command line (tier T).
//!
//! ```text
//! cargo run -p hg-opgen -- generate   # check the descriptions and write every output
//! cargo run -p hg-opgen -- check      # fail if regenerating would change anything (OC-16)
//! ```

use std::path::PathBuf;
use std::process::ExitCode;

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    match hg_opgen::cli::run(&root, &args) {
        Ok(()) => ExitCode::SUCCESS,
        Err(msg) => {
            eprintln!("{msg}");
            ExitCode::FAILURE
        }
    }
}
