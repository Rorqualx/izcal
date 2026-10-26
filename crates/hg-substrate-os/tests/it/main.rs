//! Integration tests of `hg-substrate-os`. Their presence also makes
//! `cargo test` build the crate's binaries, which the launch tests run
//! (the spawner and its probe child).

#[test]
fn the_launch_binaries_are_built() {
    for exe in [
        env!("CARGO_BIN_EXE_hg-spawner-main"),
        env!("CARGO_BIN_EXE_hg-launch-probe"),
    ] {
        assert!(std::path::Path::new(exe).exists(), "{exe}");
    }
}
