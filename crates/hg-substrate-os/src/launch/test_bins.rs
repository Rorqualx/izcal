//! Tests only: the crate's own binaries that the launch tests run (the
//! spawner and the `hg-launch-probe` child), built by the tests
//! themselves.
//!
//! Cargo builds a package's `[[bin]]` targets for `cargo test` only when
//! an integration test or bench of the package is selected. `cargo test
//! --lib` selects neither, so the lib's launch tests found no probe in a
//! target directory where it was never built (M1 phase G: three
//! `launch::windows` tests failed at the probe lookup in every
//! `--lib` suite on rig253), or ran a stale probe from an older build.
//! So the first launch test of a process runs `cargo build` for exactly
//! those binaries, in this test binary's own target directory, target
//! and profile, with this build's features. Cargo does not hold its
//! build lock while tests run (checked), and an up-to-date build is a
//! no-op.

use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::OnceLock;

/// The binaries the launch tests run.
pub(crate) const BINS: [&str; 2] = ["hg-spawner-main", "hg-launch-probe"];

/// Where a test binary sits: `<root>/[<triple>/]<profile dir>/deps/<exe>`.
#[derive(Debug, PartialEq, Eq)]
pub(crate) struct Layout {
    /// The target directory (the one holding `CACHEDIR.TAG`).
    pub(crate) root: PathBuf,
    /// The target triple, for a `--target` build.
    pub(crate) triple: Option<OsString>,
    /// The profile directory's name (`debug` for `dev` and `test`).
    pub(crate) profile_dir: OsString,
}

impl Layout {
    /// The layout of `test_exe`; `is_root` says whether a directory is a
    /// Cargo target directory.
    pub(crate) fn of(test_exe: &Path, is_root: impl Fn(&Path) -> bool) -> Option<Self> {
        let deps = test_exe.parent()?;
        let profile = deps.parent()?;
        let above = profile.parent()?;
        let (root, triple) = if is_root(above) {
            (above.to_path_buf(), None)
        } else {
            let root = above.parent()?;
            if !is_root(root) {
                return None;
            }
            (root.to_path_buf(), Some(above.file_name()?.to_os_string()))
        };
        Some(Self {
            root,
            triple,
            profile_dir: profile.file_name()?.to_os_string(),
        })
    }

    /// The binaries' directory.
    pub(crate) fn bin_dir(&self) -> PathBuf {
        let mut d = self.root.clone();
        if let Some(t) = &self.triple {
            d.push(t);
        }
        d.push(&self.profile_dir);
        d
    }

    /// `cargo build`'s arguments for [`BINS`] into this layout.
    pub(crate) fn build_args(&self, features: &[&str]) -> Vec<OsString> {
        let mut a: Vec<OsString> = vec!["build".into(), "-p".into(), "hg-substrate-os".into()];
        for b in BINS {
            a.extend(["--bin".into(), b.into()]);
        }
        // `dev` and `test` both build into `debug`.
        let profile = if self.profile_dir == "debug" {
            OsString::from("dev")
        } else {
            self.profile_dir.clone()
        };
        a.extend(["--profile".into(), profile]);
        a.extend(["--target-dir".into(), self.root.clone().into_os_string()]);
        if let Some(t) = &self.triple {
            a.extend(["--target".into(), t.clone()]);
        }
        if !features.is_empty() {
            a.extend(["--features".into(), features.join(",").into()]);
        }
        a
    }
}

/// This build's features of the crate, passed on to the binaries.
fn features() -> Vec<&'static str> {
    let mut f = Vec::new();
    if cfg!(feature = "measure-launch") {
        f.push("measure-launch");
    }
    f
}

/// Builds [`BINS`] once per test process and returns their directory.
pub(crate) fn bin_dir() -> PathBuf {
    static DIR: OnceLock<PathBuf> = OnceLock::new();
    DIR.get_or_init(|| {
        let exe = std::env::current_exe().unwrap();
        let layout = Layout::of(&exe, |d| d.join("CACHEDIR.TAG").is_file())
            .unwrap_or_else(|| panic!("{} is not in a Cargo target directory", exe.display()));
        let cargo = std::env::var_os("CARGO").unwrap_or_else(|| env!("CARGO").into());
        let out = Command::new(cargo)
            .args(layout.build_args(&features()))
            .current_dir(env!("CARGO_MANIFEST_DIR"))
            .output()
            .unwrap();
        assert!(
            out.status.success(),
            "building the launch tests' binaries failed:\n{}",
            String::from_utf8_lossy(&out.stderr)
        );
        layout.bin_dir()
    })
    .clone()
}

/// A binary of [`BINS`], built.
pub(crate) fn bin(name: &str) -> PathBuf {
    let mut p = bin_dir().join(name);
    if cfg!(windows) {
        p.set_extension("exe");
    }
    p
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_layout_is_read_from_the_test_binarys_own_path() {
        let roots = [Path::new("/w/target"), Path::new("/x/t2")];
        let is_root = |d: &Path| roots.contains(&d);
        let host =
            Layout::of(Path::new("/w/target/debug/deps/hg_substrate_os-1"), is_root).unwrap();
        assert_eq!(host.root, Path::new("/w/target"));
        assert_eq!(host.triple, None);
        assert_eq!(host.bin_dir(), Path::new("/w/target/debug"));
        let args = host.build_args(&[]);
        let s: Vec<&str> = args.iter().map(|a| a.to_str().unwrap()).collect();
        assert_eq!(
            s,
            [
                "build",
                "-p",
                "hg-substrate-os",
                "--bin",
                "hg-spawner-main",
                "--bin",
                "hg-launch-probe",
                "--profile",
                "dev",
                "--target-dir",
                "/w/target"
            ]
        );
        let cross = Layout::of(
            Path::new("/x/t2/x86_64-pc-windows-msvc/release/deps/t"),
            is_root,
        )
        .unwrap();
        assert_eq!(
            cross.bin_dir(),
            Path::new("/x/t2/x86_64-pc-windows-msvc/release")
        );
        let args = cross.build_args(&["measure-launch"]);
        let s: Vec<&str> = args.iter().map(|a| a.to_str().unwrap()).collect();
        assert!(s.ends_with(&[
            "--profile",
            "release",
            "--target-dir",
            "/x/t2",
            "--target",
            "x86_64-pc-windows-msvc",
            "--features",
            "measure-launch"
        ]));
        assert_eq!(
            Layout::of(Path::new("/elsewhere/debug/deps/t"), is_root),
            None
        );
    }

    /// The cause pinned: the lib's own tests get the binaries whatever
    /// targets the run selected. Under `cargo test --lib` in a target
    /// directory where they were never built, this failed before (no
    /// binary), and the launch tests with it.
    #[test]
    fn the_lib_tests_build_the_binaries_they_run() {
        for b in BINS {
            let p = bin(b);
            assert!(p.is_file(), "{}", p.display());
        }
    }
}
