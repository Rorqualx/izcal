//! CS-ST-17 (a), (e): the checked-in output is what the generator makes
//! now, byte for byte, on every OS.

use std::path::Path;

#[test]
#[expect(
    clippy::disallowed_methods,
    reason = "CS-ST-17: the freshness check reads the checked-in files"
)]
fn generated_code_is_fresh() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let pairs = hg_ipc_codegen::pairs(root, "hg-ipc-gen").unwrap();
    assert!(pairs.len() >= 2, "{pairs:?}");
    for p in pairs {
        let src = std::fs::read_to_string(&p.idl).unwrap();
        let want = hg_ipc_codegen::generate(&src, &p.label).unwrap();
        let have = std::fs::read_to_string(&p.rust).unwrap();
        assert!(
            have == want,
            "{} is stale: run `cargo run -p hg-ipc-codegen`",
            p.rust.display()
        );
    }
}
