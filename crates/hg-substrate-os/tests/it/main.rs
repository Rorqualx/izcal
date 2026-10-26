//! Integration tests of `hg-substrate-os`: the binaries the launch tests
//! run (the spawner and its probe child) are built by `cargo test`. The
//! lib's launch tests no longer rely on this target being selected; they
//! build the binaries themselves (`launch::test_bins`), so `--lib` runs
//! work too.

#[test]
fn the_launch_binaries_are_built() {
    for exe in [
        env!("CARGO_BIN_EXE_hg-spawner-main"),
        env!("CARGO_BIN_EXE_hg-launch-probe"),
    ] {
        assert!(std::path::Path::new(exe).exists(), "{exe}");
    }
}
