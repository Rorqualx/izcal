//! `hg-spawner-main`: Izcal's Linux spawner (security scope 6.3, SEC-SP;
//! launch design E-R2, review B2). A `[[bin]]` of `hg-substrate-os`,
//! static-pie like every Izcal binary on Linux. The broker starts it once;
//! it parses no arguments and reads no environment. The logic is
//! `hg_substrate_os::launch::linux::spawner`.

#[cfg(target_os = "linux")]
fn main() {
    let code = hg_substrate_os::launch::linux::spawner::run();
    std::process::exit(code);
}

/// The spawner exists on Linux only.
#[cfg(not(target_os = "linux"))]
fn main() {
    eprintln!("hg-spawner-main: Linux only");
    std::process::exit(2);
}
