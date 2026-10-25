//! The install device's model (distribution and updates 13 item 3,
//! DU-M0a): an in-memory install tree shared by every simulated process,
//! so a DST scenario can replace a child binary or a whole build between
//! two launches. Every operation goes into the trace.
//!
//! Files are immutable byte strings shared by reference: an open file
//! holds the one it opened, so a later rename over its path does not
//! change what it reads, as on the real file system. Directories exist
//! where files are under them.

use std::collections::BTreeMap;
use std::sync::Arc;

use hg_substrate::install::{
    InstallDevice, InstallError, InstallLock, InstallPath, InstalledFile, LockMode,
};

use crate::world::Shared;

/// The tree: files by path, and each lock's holders.
#[derive(Default)]
pub(crate) struct InstallTree {
    files: BTreeMap<String, Arc<Vec<u8>>>,
    locks: BTreeMap<String, Holders>,
}

#[derive(Default, Clone, Copy)]
struct Holders {
    shared: u32,
    exclusive: bool,
}

impl InstallTree {
    fn is_dir(&self, p: &str) -> bool {
        let prefix = format!("{p}/");
        self.files
            .range(prefix.clone()..)
            .next()
            .is_some_and(|(k, _)| k.starts_with(&prefix))
    }

    /// Every file at or under `p`.
    fn under(&self, p: &str) -> Vec<String> {
        let prefix = format!("{p}/");
        self.files
            .keys()
            .filter(|k| *k == p || k.starts_with(&prefix))
            .cloned()
            .collect()
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
        if st.install.files.contains_key(p) || st.install.is_dir(p) {
            return Err(InstallError::AlreadyExists);
        }
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
        let moving = st.install.under(a);
        if moving.is_empty() {
            return Err(InstallError::NotFound);
        }
        if a == b {
            return Ok(());
        }
        // A directory at the target is refused, as the OS refuses to
        // rename over a non-empty one; a file there goes in the same step.
        if st.install.is_dir(b) {
            return Err(InstallError::Invalid);
        }
        st.install.files.remove(b);
        for old in moving {
            if let Some(content) = st.install.files.remove(&old) {
                let rest = old.strip_prefix(a).unwrap_or_default();
                st.install.files.insert(format!("{b}{rest}"), content);
            }
        }
        st.trace.record(format_args!("install rename {a} {b}"));
        Ok(())
    }

    fn delete_tree(&self, path: &InstallPath) -> Result<(), InstallError> {
        let mut st = self.shared.lock();
        let p = path.as_str();
        let gone = st.install.under(p);
        if gone.is_empty() {
            return Err(InstallError::NotFound);
        }
        for f in gone {
            st.install.files.remove(&f);
        }
        st.trace.record(format_args!("install delete {p}"));
        Ok(())
    }

    fn open_read(&self, path: &InstallPath) -> Result<Box<dyn InstalledFile>, InstallError> {
        let mut st = self.shared.lock();
        let p = path.as_str();
        let found = st.install.files.get(p).cloned();
        let is_dir = st.install.is_dir(p);
        st.trace.record(format_args!("install open {p}"));
        match found {
            Some(content) => Ok(Box::new(SimFile(content))),
            None if is_dir => Err(InstallError::Invalid),
            None => Err(InstallError::NotFound),
        }
    }

    fn lock(&self, path: &InstallPath, mode: LockMode) -> Result<InstallLock, InstallError> {
        let mut st = self.shared.lock();
        let p = path.as_str().to_owned();
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
