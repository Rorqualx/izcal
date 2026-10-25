//! The install device on the real file system (distribution and updates
//! 13 item 3, DU-M0a): [`hg_substrate::install::InstallDevice`] rooted at
//! an install directory.
//!
//! - **Atomic rename** is `std::fs::rename`, which replaces the target in
//!   one step on every supported OS. On Unix a file already open keeps its
//!   inode, so its contents; on Windows the standard library opens files
//!   with delete sharing, so a rename over an open file succeeds and the
//!   open handle keeps the old file. The parent directory is then synced
//!   where the OS allows opening a directory (Unix), so the rename is
//!   durable.
//! - **Locks** are the standard library's advisory file locks
//!   (`File::try_lock`, `File::try_lock_shared`), never waiting.
//! - Paths are joined component by component under the root, so an
//!   [`InstallPath`] cannot leave it.

use std::fs::{File, OpenOptions};
use std::io::{ErrorKind, Read as _, Seek as _, SeekFrom, Write as _};
use std::path::{Path, PathBuf};
use std::sync::{Mutex, PoisonError};

use hg_substrate::install::{
    InstallDevice, InstallError, InstallLock, InstallPath, InstalledFile, LockMode,
};

/// The install device rooted at a directory.
#[derive(Debug, Clone)]
pub struct OsInstallDevice {
    root: PathBuf,
}

impl OsInstallDevice {
    /// The device for the install tree at `root`.
    pub fn new(root: PathBuf) -> Self {
        Self { root }
    }

    fn path(&self, p: &InstallPath) -> PathBuf {
        let mut out = self.root.clone();
        for c in p.components() {
            out.push(c);
        }
        out
    }

    /// Whether a component of `p` above it is a file (or anything but a
    /// directory). Checked first, so each OS gives the contract's error
    /// rather than its own (Unix says "not a directory", Windows "not
    /// found").
    fn file_above(&self, p: &InstallPath) -> bool {
        let mut at = self.root.clone();
        let mut parts = p.components().peekable();
        while let Some(c) = parts.next() {
            if parts.peek().is_none() {
                break;
            }
            at.push(c);
            if std::fs::symlink_metadata(&at).is_ok_and(|m| !m.is_dir()) {
                return true;
            }
        }
        false
    }
}

/// Whether `p` is `root` or under it.
fn at_or_under(p: &InstallPath, root: &InstallPath) -> bool {
    let (p, root) = (p.as_str(), root.as_str());
    p == root
        || p.strip_prefix(root)
            .is_some_and(|rest| rest.starts_with('/'))
}

fn map(e: &std::io::Error) -> InstallError {
    match e.kind() {
        ErrorKind::NotFound => InstallError::NotFound,
        ErrorKind::AlreadyExists => InstallError::AlreadyExists,
        _ => InstallError::Io,
    }
}

/// Makes a directory entry durable where the OS lets a directory be
/// opened and synced (Unix); elsewhere the rename is already durable when
/// it returns, or the OS gives no way to ask.
fn sync_dir(dir: &Path) {
    if let Ok(d) = File::open(dir) {
        let _ = d.sync_all();
    }
}

/// An open install file.
struct OsInstalledFile(Mutex<File>);

impl InstalledFile for OsInstalledFile {
    fn read_all(&self) -> Result<Vec<u8>, InstallError> {
        let mut f = self.0.lock().unwrap_or_else(PoisonError::into_inner);
        f.seek(SeekFrom::Start(0)).map_err(|e| map(&e))?;
        let mut out = Vec::new();
        f.read_to_end(&mut out).map_err(|e| map(&e))?;
        Ok(out)
    }

    fn byte_len(&self) -> Result<u64, InstallError> {
        let f = self.0.lock().unwrap_or_else(PoisonError::into_inner);
        f.metadata().map(|m| m.len()).map_err(|e| map(&e))
    }
}

impl InstallDevice for OsInstallDevice {
    fn write_new(&self, path: &InstallPath, bytes: &[u8]) -> Result<(), InstallError> {
        let full = self.path(path);
        if self.file_above(path) {
            return Err(InstallError::Invalid);
        }
        if std::fs::symlink_metadata(&full).is_ok() {
            return Err(InstallError::AlreadyExists);
        }
        if let Some(parent) = full.parent() {
            std::fs::create_dir_all(parent).map_err(|e| map(&e))?;
        }
        let mut f = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&full)
            .map_err(|e| map(&e))?;
        f.write_all(bytes).map_err(|e| map(&e))?;
        f.sync_all().map_err(|e| map(&e))?;
        if let Some(parent) = full.parent() {
            sync_dir(parent);
        }
        Ok(())
    }

    fn rename(&self, from: &InstallPath, to: &InstallPath) -> Result<(), InstallError> {
        let (a, b) = (self.path(from), self.path(to));
        if self.file_above(from) {
            return Err(InstallError::NotFound);
        }
        let moving = std::fs::symlink_metadata(&a).map_err(|e| map(&e))?;
        if from == to {
            return Ok(());
        }
        let target = std::fs::symlink_metadata(&b).ok();
        let refused = self.file_above(to)
            || target.as_ref().is_some_and(std::fs::Metadata::is_dir)
            || (moving.is_dir() && target.is_some())
            || at_or_under(to, from);
        if refused {
            return Err(InstallError::Invalid);
        }
        if let Some(parent) = b.parent() {
            std::fs::create_dir_all(parent).map_err(|e| map(&e))?;
        }
        std::fs::rename(&a, &b).map_err(|e| map(&e))?;
        if let Some(parent) = b.parent() {
            sync_dir(parent);
        }
        Ok(())
    }

    fn delete_tree(&self, path: &InstallPath) -> Result<(), InstallError> {
        let full = self.path(path);
        if self.file_above(path) {
            return Err(InstallError::NotFound);
        }
        let meta = std::fs::symlink_metadata(&full).map_err(|e| map(&e))?;
        if meta.is_dir() {
            std::fs::remove_dir_all(&full).map_err(|e| map(&e))
        } else {
            std::fs::remove_file(&full).map_err(|e| map(&e))
        }
    }

    fn open_read(&self, path: &InstallPath) -> Result<Box<dyn InstalledFile>, InstallError> {
        let full = self.path(path);
        if self.file_above(path) {
            return Err(InstallError::NotFound);
        }
        // Checked before opening: Windows refuses to open a directory
        // (access denied) where Unix opens it.
        if std::fs::metadata(&full).map_err(|e| map(&e))?.is_dir() {
            return Err(InstallError::Invalid);
        }
        let f = File::open(&full).map_err(|e| map(&e))?;
        Ok(Box::new(OsInstalledFile(Mutex::new(f))))
    }

    fn lock(&self, path: &InstallPath, mode: LockMode) -> Result<InstallLock, InstallError> {
        let full = self.path(path);
        if self.file_above(path) || std::fs::metadata(&full).is_ok_and(|m| m.is_dir()) {
            return Err(InstallError::Invalid);
        }
        if let Some(parent) = full.parent() {
            std::fs::create_dir_all(parent).map_err(|e| map(&e))?;
        }
        let f = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .open(&full)
            .map_err(|e| map(&e))?;
        let taken = match mode {
            LockMode::Shared => f.try_lock_shared(),
            LockMode::Exclusive => f.try_lock(),
        };
        match taken {
            Ok(()) => Ok(InstallLock::new(Box::new(f))),
            Err(std::fs::TryLockError::WouldBlock) => Err(InstallError::WouldBlock),
            Err(std::fs::TryLockError::Error(e)) => Err(map(&e)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn device(tag: &str) -> (OsInstallDevice, PathBuf) {
        let root = std::env::temp_dir().join(format!("hg-install-{tag}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).unwrap();
        (OsInstallDevice::new(root.clone()), root)
    }

    fn p(s: &str) -> InstallPath {
        InstallPath::new(s).unwrap()
    }

    #[test]
    fn an_open_file_keeps_its_contents_across_a_rename() {
        let (d, root) = device("rename");
        d.write_new(&p("builds/1/child"), b"build one").unwrap();
        let old = d.open_read(&p("builds/1/child")).unwrap();
        // An update stages the new binary and renames it into place.
        d.write_new(&p("builds/1/child.new"), b"build two!")
            .unwrap();
        d.rename(&p("builds/1/child.new"), &p("builds/1/child"))
            .unwrap();
        assert_eq!(old.read_all().unwrap(), b"build one");
        assert_eq!(
            d.open_read(&p("builds/1/child"))
                .unwrap()
                .read_all()
                .unwrap(),
            b"build two!"
        );
        drop(old);
        // As in the model: a directory target is refused, a self-rename
        // changes nothing, and a new build goes to a new directory.
        d.write_new(&p("staging/child"), b"build three").unwrap();
        assert_eq!(
            d.rename(&p("staging"), &p("builds/1")).err(),
            Some(InstallError::Invalid)
        );
        d.rename(&p("staging"), &p("builds/2")).unwrap();
        d.rename(&p("builds/2"), &p("builds/2")).unwrap();
        assert_eq!(
            d.open_read(&p("builds/2/child"))
                .unwrap()
                .read_all()
                .unwrap(),
            b"build three"
        );
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn create_delete_and_open_errors() {
        let (d, root) = device("errors");
        d.write_new(&p("a/b"), b"x").unwrap();
        assert_eq!(
            d.write_new(&p("a/b"), b"y").err(),
            Some(InstallError::AlreadyExists)
        );
        assert_eq!(d.open_read(&p("a")).err(), Some(InstallError::Invalid));
        assert_eq!(d.open_read(&p("nope")).err(), Some(InstallError::NotFound));
        let all = d.open_inventory(&[p("a/b")]).unwrap();
        assert_eq!(all.len(), 1);
        assert_eq!(
            d.open_inventory(&[p("a/b"), p("a/c")]).err(),
            Some(InstallError::NotFound)
        );
        d.delete_tree(&p("a")).unwrap();
        assert_eq!(d.open_read(&p("a/b")).err(), Some(InstallError::NotFound));
        assert_eq!(d.delete_tree(&p("a")).err(), Some(InstallError::NotFound));
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn locks_never_wait() {
        let (d, root) = device("locks");
        let lock = p("build.lock");
        let s1 = d.lock(&lock, LockMode::Shared).unwrap();
        let s2 = d.lock(&lock, LockMode::Shared).unwrap();
        assert_eq!(
            d.lock(&lock, LockMode::Exclusive).err(),
            Some(InstallError::WouldBlock)
        );
        drop((s1, s2));
        let x = d.lock(&lock, LockMode::Exclusive).unwrap();
        assert_eq!(
            d.lock(&lock, LockMode::Shared).err(),
            Some(InstallError::WouldBlock)
        );
        drop(x);
        std::fs::remove_dir_all(root).unwrap();
    }
}
