//! The install device's conformance test (distribution and updates 13
//! item 3): one script of operations against the model and against the
//! real file system, which must give the same result at every step. A
//! DST scenario is only worth what the model's agreement with the OS is.

use std::sync::Arc;

use hg_substrate::install::{InstallDevice, InstallError, InstallLock, InstallPath, LockMode};
use hg_substrate_os::install::OsInstallDevice;
use hg_substrate_sim::{SimConfig, World};

enum Op {
    Write(&'static str, &'static str),
    Rename(&'static str, &'static str),
    Delete(&'static str),
    Read(&'static str),
    Lock(&'static str, LockMode),
    Unlock,
}

/// Each step and what it must give.
const SCRIPT: &[(Op, &str)] = &[
    (Op::Write("a/f", "one"), "ok"),
    // Under a file, and over a directory.
    (Op::Write("a/f/x", "two"), "Invalid"),
    (Op::Write("a", "three"), "AlreadyExists"),
    (Op::Write("d/g", "four"), "ok"),
    // A directory onto a file, into its own subtree; a file under itself.
    (Op::Rename("d", "a/f"), "Invalid"),
    (Op::Rename("d", "d/sub"), "Invalid"),
    (Op::Rename("a/f", "a/f/y"), "Invalid"),
    (Op::Read("a/f"), "one"),
    (Op::Read("d/g"), "four"),
    // A lock file is created, empty, and is a file like any other.
    // (Read only once unlocked: Windows locks are mandatory, so a locked
    // file cannot be read there, where Unix reads it.)
    (Op::Lock("l/lk", LockMode::Exclusive), "ok"),
    (Op::Lock("l/lk", LockMode::Shared), "WouldBlock"),
    (Op::Unlock, "ok"),
    (Op::Read("l/lk"), ""),
    (Op::Write("l/lk", "five"), "AlreadyExists"),
    (Op::Lock("a", LockMode::Shared), "Invalid"),
    (Op::Lock("a/f/z", LockMode::Shared), "Invalid"),
    (Op::Lock("d/g", LockMode::Shared), "ok"),
    (Op::Unlock, "ok"),
    (Op::Read("d/g"), "four"),
    // Nothing is found under a file.
    (Op::Read("a/f/x"), "NotFound"),
    (Op::Delete("a/f/x"), "NotFound"),
    (Op::Rename("a/f/x", "z"), "NotFound"),
    // The directory a file leaves stays, empty, and is still a directory.
    (Op::Rename("a/f", "e/f"), "ok"),
    (Op::Read("a"), "Invalid"),
    (Op::Write("a", "six"), "AlreadyExists"),
    (Op::Rename("d", "a"), "Invalid"),
    (Op::Read("e/f"), "one"),
    (Op::Rename("missing", "y"), "NotFound"),
    // A whole directory moves to a new name, and goes away when deleted.
    (Op::Rename("d", "n/d"), "ok"),
    (Op::Read("n/d/g"), "four"),
    (Op::Read("d/g"), "NotFound"),
    (Op::Delete("a"), "ok"),
    (Op::Read("a"), "NotFound"),
    (Op::Write("a/f", "seven"), "ok"),
    (Op::Read("a/f"), "seven"),
];

fn run(d: &dyn InstallDevice) -> Vec<String> {
    let p = |s: &str| InstallPath::new(s).unwrap();
    let mut held: Option<InstallLock> = None;
    let show = |r: Result<(), InstallError>| match r {
        Ok(()) => "ok".to_owned(),
        Err(e) => format!("{e:?}"),
    };
    SCRIPT
        .iter()
        .map(|(op, _)| match op {
            Op::Write(path, text) => show(d.write_new(&p(path), text.as_bytes())),
            Op::Rename(a, b) => show(d.rename(&p(a), &p(b))),
            Op::Delete(path) => show(d.delete_tree(&p(path))),
            Op::Read(path) => match d.open_read(&p(path)).and_then(|f| f.read_all()) {
                Ok(bytes) => String::from_utf8(bytes).unwrap(),
                Err(e) => format!("{e:?}"),
            },
            Op::Lock(path, mode) => show(d.lock(&p(path), *mode).map(|l| held = Some(l))),
            Op::Unlock => show(held.take().map(drop).ok_or(InstallError::Io)),
        })
        .collect()
}

#[test]
#[expect(
    clippy::disallowed_methods,
    reason = "the OS device needs a scratch install tree on the real file system"
)]
fn the_model_and_the_file_system_agree() {
    let expected: Vec<&str> = SCRIPT.iter().map(|(_, want)| *want).collect();
    let world = World::new(SimConfig::new(11));
    let model: Arc<dyn InstallDevice> = world.install_device();
    assert_eq!(run(model.as_ref()), expected, "the model");
    let root = std::env::temp_dir().join(format!("hg-install-conf-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(&root).unwrap();
    let os = OsInstallDevice::new(root.clone());
    assert_eq!(run(&os), expected, "the file system");
    std::fs::remove_dir_all(&root).unwrap();
}
