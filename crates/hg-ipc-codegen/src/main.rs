//! Regenerates the checked-in code of `hg-ipc-gen` and `hg-os-types` from
//! their `.hgidl` files.
//!
//! ```text
//! cargo run -p hg-ipc-codegen            # write src/generated/*.rs
//! cargo run -p hg-ipc-codegen -- --check # fail if any is stale
//! ```

use std::path::Path;
use std::process::ExitCode;

/// The crates whose `idl/*.hgidl` the generator writes into their
/// `src/generated/`.
const GENERATED_CRATES: [&str; 2] = ["hg-ipc-gen", "hg-os-types"];

fn main() -> ExitCode {
    let check = std::env::args().any(|a| a == "--check");
    let mut pairs = Vec::new();
    for krate in GENERATED_CRATES {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("..").join(krate);
        match hg_ipc_codegen::pairs(&root, krate) {
            Ok(p) => pairs.extend(p),
            Err(e) => {
                eprintln!("hg-ipc-codegen: {}: {e}", root.display());
                return ExitCode::FAILURE;
            }
        }
    }
    let mut stale = 0;
    for p in &pairs {
        let src = match std::fs::read_to_string(&p.idl) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("{}: {e}", p.label);
                return ExitCode::FAILURE;
            }
        };
        let out = match hg_ipc_codegen::generate(&src, &p.label) {
            Ok(o) => o,
            Err(e) => {
                eprintln!("{}:{e}", p.label);
                return ExitCode::FAILURE;
            }
        };
        let current = std::fs::read_to_string(&p.rust).unwrap_or_default();
        if current == out {
            continue;
        }
        if check {
            eprintln!(
                "stale: {} (run `cargo run -p hg-ipc-codegen`)",
                p.rust.display()
            );
            stale += 1;
        } else if let Err(e) = std::fs::write(&p.rust, out) {
            eprintln!("{}: {e}", p.rust.display());
            return ExitCode::FAILURE;
        } else {
            println!("wrote {}", p.rust.display());
        }
    }
    if stale > 0 {
        ExitCode::FAILURE
    } else {
        ExitCode::SUCCESS
    }
}
