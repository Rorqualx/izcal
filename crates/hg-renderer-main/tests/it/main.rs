//! Its presence makes `cargo test` build this executable, which the
//! launch tool's OS round trip (`hg-launch-tool`) starts from the build
//! directory.

#[test]
fn the_executable_is_built() {
    assert!(std::path::Path::new(env!("CARGO_BIN_EXE_hg-renderer-main")).exists());
}
