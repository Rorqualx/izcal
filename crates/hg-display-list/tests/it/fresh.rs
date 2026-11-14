//! CS-ST-17 (a), (e): the checked-in output is what the generator
//! makes now, byte for byte, on every OS — and a hand edit is caught.

use std::path::Path;

#[test]
#[expect(
    clippy::disallowed_methods,
    reason = "CS-ST-17: the freshness check reads the checked-in files"
)]
fn dl01_the_checked_in_output_is_fresh_and_a_hand_edit_is_caught() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let pairs = hg_ipc_codegen::pairs(root, "hg-display-list").unwrap();
    assert_eq!(pairs.len(), 1, "{pairs:?}");
    for p in pairs {
        let src = std::fs::read_to_string(&p.idl).unwrap();
        let want = hg_ipc_codegen::generate(&src, &p.label).unwrap();
        let have = std::fs::read_to_string(&p.rust).unwrap();
        assert!(
            have == want,
            "{} is stale: run `cargo run -p hg-ipc-codegen`",
            p.rust.display()
        );

        // The twin, which is what makes the check above worth having:
        // the same comparison against an edited copy must fail. A
        // freshness test that cannot fail is a test that would pass on
        // a hand-edited file, which is the thing G29 forbids.
        let edited = have.replacen("pub struct Point", "pub struct Pointt", 1);
        assert_ne!(edited, have, "the planted edit changed nothing");
        assert_ne!(edited, want, "a hand edit would not be caught");
    }
}
