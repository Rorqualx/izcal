//! The install device's model (distribution and updates 13 item 3,
//! DU-M0a): an in-memory install tree shared by every simulated process,
//! so a DST scenario can replace a child binary or a whole build between
//! two launches. Every operation goes into the trace.
//!
//! Files are immutable byte strings shared by reference: an open file
//! holds the one it opened, so a later rename over its path does not
//! change what it reads, as on the real file system. Directories are
//! entries of their own: made for a file's parents, and left, empty, when
//! the last file under one moves away, as on the OS. Every refusal is the
//! one [`InstallDevice`] names, which the OS device gives too (the
//! conformance test runs one script against both).

use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

use hg_substrate::install::{
    InstallDevice, InstallError, InstallLock, InstallPath, InstalledFile, LockMode,
};

use crate::world::Shared;

/// The tree: files and directories by path, and each lock's holders.
#[derive(Default)]
pub(crate) struct InstallTree {
    files: BTreeMap<String, Arc<Vec<u8>>>,
    dirs: BTreeSet<String>,
    locks: BTreeMap<String, Holders>,
}

#[derive(Default, Clone, Copy)]
struct Holders {
    shared: u32,
    exclusive: bool,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Kind {
    File,
    Dir,
}

/// The proper prefixes of `p`: `a`, `a/b` for `a/b/c`.
fn ancestors(p: &str) -> impl Iterator<Item = &str> {
    p.match_indices('/')
        .map(move |(i, _)| p.get(..i).unwrap_or(p))
}

/// Whether `p` is `root` or under it.
fn at_or_under(p: &str, root: &str) -> bool {
    p == root
        || p.strip_prefix(root)
            .is_some_and(|rest| rest.starts_with('/'))
}

impl InstallTree {
    fn kind(&self, p: &str) -> Option<Kind> {
        if self.files.contains_key(p) {
            Some(Kind::File)
        } else if self.dirs.contains(p) {
            Some(Kind::Dir)
        } else {
            None
        }
    }

    /// Whether a component of `p` above it is a file.
    fn file_above(&self, p: &str) -> bool {
        ancestors(p).any(|a| self.files.contains_key(a))
    }

    fn make_parents(&mut self, p: &str) {
        for a in ancestors(p) {
            self.dirs.insert(a.to_owned());
        }
    }

    /// Moves every entry at or under `a` to the same place under `b`.
    fn move_tree(&mut self, a: &str, b: &str) {
        let files: Vec<String> = self
            .files
            .keys()
            .filter(|k| at_or_under(k, a))
            .cloned()
            .collect();
        for old in files {
            if let Some(content) = self.files.remove(&old) {
                let rest = old.get(a.len()..).unwrap_or_default();
                self.files.insert(format!("{b}{rest}"), content);
            }
        }
        let dirs: Vec<String> = self
            .dirs
            .iter()
            .filter(|k| at_or_under(k, a))
            .cloned()
            .collect();
        for old in dirs {
            self.dirs.remove(&old);
            let rest = old.get(a.len()..).unwrap_or_default();
            self.dirs.insert(format!("{b}{rest}"));
        }
    }

    fn remove_tree(&mut self, p: &str) {
        self.files.retain(|k, _| !at_or_under(k, p));
        self.dirs.retain(|k| !at_or_under(k, p));
    }
}

/// The world's install device.
pub(crate) struct SimInstallDevice {
    pub(crate) shared: Arc<Shared>,
}

struct SimFile(Arc<Vec<u8>>);

impl InstalledFile for SimFile {
    fn read_all(&self) -> Result<Vec<u8>, InstallError> {
        Ok(self.0.as_ref().clone())
    }

    fn byte_len(&self) -> Result<u64, InstallError> {
        u64::try_from(self.0.len()).map_err(|_e| InstallError::Io)
    }
}

/// Releases a lock when dropped.
struct LockGuard {
    shared: Arc<Shared>,
    path: String,
    mode: LockMode,
}

impl Drop for LockGuard {
    fn drop(&mut self) {
        let mut st = self.shared.lock();
        if let Some(h) = st.install.locks.get_mut(&self.path) {
            match self.mode {
                LockMode::Shared => h.shared = h.shared.saturating_sub(1),
                LockMode::Exclusive => h.exclusive = false,
            }
        }
        st.trace
            .record(format_args!("install unlock {} {:?}", self.path, self.mode));
    }
}

impl InstallDevice for SimInstallDevice {
    fn write_new(&self, path: &InstallPath, bytes: &[u8]) -> Result<(), InstallError> {
        let mut st = self.shared.lock();
        let p = path.as_str();
        if st.install.file_above(p) {
            return Err(InstallError::Invalid);
        }
        if st.install.kind(p).is_some() {
            return Err(InstallError::AlreadyExists);
        }
        st.install.make_parents(p);
        st.install
            .files
            .insert(p.to_owned(), Arc::new(bytes.to_vec()));
        st.trace
            .record(format_args!("install write {p} {} bytes", bytes.len()));
        Ok(())
    }

    fn rename(&self, from: &InstallPath, to: &InstallPath) -> Result<(), InstallError> {
        let mut st = self.shared.lock();
        let (a, b) = (from.as_str(), to.as_str());
        let t = &mut st.install;
        let Some(moving) = t.kind(a) else {
            return Err(InstallError::NotFound);
        };
        if a == b {
            return Ok(());
        }
        let target = t.kind(b);
        let refused = t.file_above(b)
            || target == Some(Kind::Dir)
            || (moving == Kind::Dir && target == Some(Kind::File))
            || at_or_under(b, a);
        if refused {
            return Err(InstallError::Invalid);
        }
        t.make_parents(b);
        t.files.remove(b);
        t.move_tree(a, b);
        st.trace.record(format_args!("install rename {a} {b}"));
        Ok(())
    }

    fn delete_tree(&self, path: &InstallPath) -> Result<(), InstallError> {
        let mut st = self.shared.lock();
        let p = path.as_str();
        if st.install.kind(p).is_none() {
            return Err(InstallError::NotFound);
        }
        st.install.remove_tree(p);
        st.trace.record(format_args!("install delete {p}"));
        Ok(())
    }

    fn open_read(&self, path: &InstallPath) -> Result<Box<dyn InstalledFile>, InstallError> {
        let mut st = self.shared.lock();
        let p = path.as_str();
        let found = st.install.files.get(p).cloned();
        let kind = st.install.kind(p);
        st.trace.record(format_args!("install open {p}"));
        match (found, kind) {
            (Some(content), _) => Ok(Box::new(SimFile(content))),
            (None, Some(Kind::Dir)) => Err(InstallError::Invalid),
            (None, _) => Err(InstallError::NotFound),
        }
    }

    fn lock(&self, path: &InstallPath, mode: LockMode) -> Result<InstallLock, InstallError> {
        let mut st = self.shared.lock();
        let p = path.as_str().to_owned();
        if st.install.file_above(&p) || st.install.kind(&p) == Some(Kind::Dir) {
            return Err(InstallError::Invalid);
        }
        // The lock file, made empty if missing, as the OS device makes it.
        if st.install.kind(&p).is_none() {
            st.install.make_parents(&p);
            st.install.files.insert(p.clone(), Arc::new(Vec::new()));
        }
        let h = st.install.locks.entry(p.clone()).or_default();
        let free = match mode {
            LockMode::Shared => !h.exclusive,
            LockMode::Exclusive => !h.exclusive && h.shared == 0,
        };
        if !free {
            st.trace
                .record(format_args!("install lock {p} {mode:?} would-block"));
            return Err(InstallError::WouldBlock);
        }
        match mode {
            LockMode::Shared => h.shared = h.shared.saturating_add(1),
            LockMode::Exclusive => h.exclusive = true,
        }
        st.trace.record(format_args!("install lock {p} {mode:?}"));
        drop(st);
        Ok(InstallLock::new(Box::new(LockGuard {
            shared: Arc::clone(&self.shared),
            path: p,
            mode,
        })))
    }
}
