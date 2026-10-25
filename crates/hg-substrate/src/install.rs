//! The install device (distribution and updates 13 item 3, DU-M0a): the
//! file operations the launcher, broker and updater need on the install
//! tree, as one contract with two implementations, so DST can replace a
//! child binary or a whole build between two launches.
//!
//! The operations: create a file, rename atomically (replacing the
//! target), delete a tree, open read-only, open every entry of a build's
//! inventory, and take a shared or exclusive lock. Every path is an
//! [`InstallPath`], relative to the device's root, so no caller can name
//! a file outside the install tree.
//!
//! One property matters most and both implementations keep it: **a file
//! opened before a rename keeps its contents.** A broker that opened its
//! build's binaries at start launches from those handles, so an update
//! that renames a new build into place never changes what a running
//! build launches (DU-D1, G251).
//!
//! The production implementation is `hg-substrate-os`'s; the model is
//! `hg-substrate-sim`'s. Phase E wires the device into the launcher; this
//! is the contract and both halves.

use core::any::Any;

/// A path inside the install tree: relative, `/`-separated, each
/// component non-empty and neither `.` nor `..`, with no NUL, `\` or `:`
/// (which Windows reads as a drive or stream), at most
/// [`InstallPath::MAX_LEN`] bytes.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct InstallPath(String);

impl InstallPath {
    /// The longest path accepted, in bytes†.
    pub const MAX_LEN: usize = 1024;

    /// `s`, if it is a valid install path.
    ///
    /// # Errors
    /// [`InstallError::Invalid`].
    pub fn new(s: &str) -> Result<Self, InstallError> {
        let ok = !s.is_empty()
            && s.len() <= Self::MAX_LEN
            && !s.contains(['\0', '\\', ':'])
            && s.split('/').all(|c| !c.is_empty() && c != "." && c != "..");
        if ok {
            Ok(Self(s.to_owned()))
        } else {
            Err(InstallError::Invalid)
        }
    }

    /// The path.
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Its components.
    pub fn components(&self) -> impl Iterator<Item = &str> {
        self.0.split('/')
    }
}

/// A lock's mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LockMode {
    /// Many holders.
    Shared,
    /// One holder.
    Exclusive,
}

/// Why an install-device operation failed. Closed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InstallError {
    /// No such file or directory.
    NotFound,
    /// The file already exists.
    AlreadyExists,
    /// The lock is held in a conflicting mode; locks never wait.
    WouldBlock,
    /// An invalid path, or an operation on the wrong kind of entry.
    Invalid,
    /// The OS failed otherwise.
    Io,
}

impl core::fmt::Display for InstallError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(match self {
            Self::NotFound => "no such install file",
            Self::AlreadyExists => "the install file exists",
            Self::WouldBlock => "the install lock is held",
            Self::Invalid => "invalid install path or entry",
            Self::Io => "install device I/O failed",
        })
    }
}

impl std::error::Error for InstallError {}

/// An open, read-only install file. Its contents are the ones it had when
/// it was opened, whatever is renamed over its path later.
pub trait InstalledFile: Send + Sync {
    /// The whole file.
    ///
    /// # Errors
    /// [`InstallError::Io`].
    fn read_all(&self) -> Result<Vec<u8>, InstallError>;

    /// Its length in bytes.
    ///
    /// # Errors
    /// [`InstallError::Io`].
    fn byte_len(&self) -> Result<u64, InstallError>;
}

/// A held lock; dropping it releases the lock.
pub struct InstallLock {
    /// Held only for its `Drop`.
    _guard: Box<dyn Any + Send>,
}

impl InstallLock {
    /// Wraps the implementation's guard. For implementations only.
    pub fn new(guard: Box<dyn Any + Send>) -> Self {
        Self { _guard: guard }
    }
}

impl core::fmt::Debug for InstallLock {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str("InstallLock")
    }
}

/// The install tree's file operations.
pub trait InstallDevice: Send + Sync {
    /// Creates `path` with `bytes`, making missing parent directories,
    /// and makes it durable before returning.
    ///
    /// # Errors
    /// [`InstallError::AlreadyExists`], [`InstallError::Io`].
    fn write_new(&self, path: &InstallPath, bytes: &[u8]) -> Result<(), InstallError>;

    /// Renames `from` (a file or a directory) to `to` in one step,
    /// replacing `to` if it is a file; files already open at `to` keep
    /// their contents. A directory at `to` is refused, as the OS refuses
    /// to rename over a non-empty one: a new build goes to a new
    /// directory, and the small `current` file is what is replaced.
    /// Renaming a path to itself changes nothing.
    ///
    /// # Errors
    /// [`InstallError::NotFound`], [`InstallError::Invalid`] for a
    /// directory at `to`, [`InstallError::Io`].
    fn rename(&self, from: &InstallPath, to: &InstallPath) -> Result<(), InstallError>;

    /// Deletes `path` and everything under it.
    ///
    /// # Errors
    /// [`InstallError::NotFound`], [`InstallError::Io`].
    fn delete_tree(&self, path: &InstallPath) -> Result<(), InstallError>;

    /// Opens the file at `path` read-only.
    ///
    /// # Errors
    /// [`InstallError::NotFound`], [`InstallError::Invalid`] for a
    /// directory, [`InstallError::Io`].
    fn open_read(&self, path: &InstallPath) -> Result<Box<dyn InstalledFile>, InstallError>;

    /// Opens every file of a build's inventory, in order, or none: the
    /// first failure is the result. The broker does this once at start.
    ///
    /// # Errors
    /// As [`InstallDevice::open_read`], for the first entry that fails.
    fn open_inventory(
        &self,
        paths: &[InstallPath],
    ) -> Result<Vec<Box<dyn InstalledFile>>, InstallError> {
        paths.iter().map(|p| self.open_read(p)).collect()
    }

    /// Takes the lock at `path` (a lock file, created if missing) in
    /// `mode`, without waiting.
    ///
    /// # Errors
    /// [`InstallError::WouldBlock`] if held in a conflicting mode,
    /// [`InstallError::Io`].
    fn lock(&self, path: &InstallPath, mode: LockMode) -> Result<InstallLock, InstallError>;
}

#[cfg(test)]
mod tests {
    use super::InstallPath;

    #[test]
    fn install_paths_stay_inside_the_tree() {
        for good in ["a", "builds/1.0.0/izcal", "current", "a/b-c_d.e"] {
            assert!(InstallPath::new(good).is_ok(), "{good}");
        }
        for bad in [
            "", "/abs", "a/../b", "./a", "a/", "a//b", "a\\b", "a\0b", "..", "C:x", "a:b",
        ] {
            assert!(InstallPath::new(bad).is_err(), "{bad:?}");
        }
        assert!(InstallPath::new(&"a".repeat(InstallPath::MAX_LEN + 1)).is_err());
    }
}
