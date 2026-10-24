//! Regenerates `hg-ipc-gen`'s checked-in code from its `.hgidl` files.
//!
//! ```text
//! cargo run -p hg-ipc-codegen            # write src/generated/*.rs
//! cargo run -p hg-ipc-codegen -- --check # fail if any is stale
//! ```

use std::path::Path;
use std::process::ExitCode;

fn main() -> ExitCode {
    let check = std::env::args().any(|a| a == "--check");
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../hg-ipc-gen");
    let pairs = match hg_ipc_codegen::pairs(&root, "hg-ipc-gen") {
        Ok(p) => p,
        Err(e) => {
            eprintln!("hg-ipc-codegen: {}: {e}", root.display());
            return ExitCode::FAILURE;
        }
    };
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
