//! `hg-build-id`: a build's identity (distribution and updates 13 item 5,
//! DU-M0a). Tools layer, tier T.
//!
//! `BuildId` is a **truncated digest of the build inputs, fixed before
//! compilation**: source commit, `specs.lock` and `downloads.lock`,
//! toolchain pin, build profile, target triple, channel, release version
//! and the `.hgidl` schema hash. This tool computes it; the build passes
//! it to the compiler as `IZCAL_BUILD_ID`, and `hg-types` compiles it
//! into every binary (`hg_types::BUILD_ID`), so every
//! process of one build reports the same value and a child of another
//! build cannot report the parent's. No build script is involved (coding
//! standard CS-ST-7).
//!
//! A build made without `IZCAL_BUILD_ID` (a plain `cargo build`) carries
//! the published [`unpinned`] value. Two such builds cannot tell each
//! other apart, so every distributed build is pinned.
//!
//! The digest is SHA-256 ([`sha256`]) over [`Inputs::encoding`]; the
//! `BuildId` is its first eight bytes as a little-endian `u64`.

use std::path::Path;
use std::process::Command;

pub mod sha256;

/// What a build is made from.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Inputs {
    /// The source commit, with a digest of uncommitted changes appended
    /// when the tree is dirty.
    pub commit: String,
    /// `specs.lock`'s bytes, if the file exists.
    pub specs_lock: Option<Vec<u8>>,
    /// `downloads.lock`'s bytes, if the file exists.
    pub downloads_lock: Option<Vec<u8>>,
    /// `rust-toolchain.toml`'s bytes.
    pub toolchain: Vec<u8>,
    /// The cargo profile.
    pub profile: String,
    /// The target triple.
    pub target: String,
    /// The channel (`dev` for development builds).
    pub channel: String,
    /// The release version (`0.0.0` for development builds).
    pub version: String,
    /// [`schema_hash`] of every `.hgidl` file.
    pub schema: [u8; 32],
}

impl Inputs {
    /// The canonical encoding: the tag `izcal-build-v1`, then each input
    /// in a fixed order as its name, a presence byte, a u64 little-endian
    /// length and its bytes.
    pub fn encoding(&self) -> Vec<u8> {
        let mut out = b"izcal-build-v1".to_vec();
        let mut put = |name: &str, value: Option<&[u8]>| {
            out.extend_from_slice(name.as_bytes());
            out.push(0);
            match value {
                None => out.push(0),
                Some(v) => {
                    out.push(1);
                    out.extend_from_slice(&(v.len() as u64).to_le_bytes());
                    out.extend_from_slice(v);
                }
            }
        };
        put("commit", Some(self.commit.as_bytes()));
        put("specs.lock", self.specs_lock.as_deref());
        put("downloads.lock", self.downloads_lock.as_deref());
        put("toolchain", Some(&self.toolchain));
        put("profile", Some(self.profile.as_bytes()));
        put("target", Some(self.target.as_bytes()));
        put("channel", Some(self.channel.as_bytes()));
        put("version", Some(self.version.as_bytes()));
        put("schema", Some(&self.schema));
        out
    }

    /// The `BuildId`.
    pub fn build_id(&self) -> u64 {
        truncate(&sha256::sha256(&self.encoding()))
    }
}

/// The first eight bytes of a digest as a little-endian `u64`.
pub fn truncate(d: &[u8; 32]) -> u64 {
    let mut a = [0u8; 8];
    a.copy_from_slice(&d[..8]);
    u64::from_le_bytes(a)
}

/// The inputs of a build made without `IZCAL_BUILD_ID`. Its `BuildId` is
/// the constant `hg-types` compiles in when the variable is unset.
pub fn unpinned() -> Inputs {
    Inputs {
        commit: "unpinned".into(),
        specs_lock: None,
        downloads_lock: None,
        toolchain: Vec::new(),
        profile: "unpinned".into(),
        target: "unpinned".into(),
        channel: "dev".into(),
        version: "0.0.0".into(),
        schema: [0; 32],
    }
}

/// SHA-256 over every `crates/*/idl/*.hgidl` under `root`, sorted by
/// repository path, each as its path, a NUL, a u64 little-endian length
/// and its bytes.
///
/// # Errors
/// A directory or file could not be read.
pub fn schema_hash(root: &Path) -> std::io::Result<[u8; 32]> {
    let mut files = Vec::new();
    for krate in std::fs::read_dir(root.join("crates"))? {
        let idl = krate?.path().join("idl");
        if !idl.is_dir() {
            continue;
        }
        for f in std::fs::read_dir(&idl)? {
            let p = f?.path();
            if p.extension().and_then(|e| e.to_str()) == Some("hgidl") {
                let rel = p
                    .strip_prefix(root)
                    .unwrap_or(&p)
                    .to_string_lossy()
                    .replace('\\', "/");
                files.push((rel, p));
            }
        }
    }
    files.sort();
    let mut all = Vec::new();
    for (rel, p) in files {
        let bytes = std::fs::read(&p)?;
        all.extend_from_slice(rel.as_bytes());
        all.push(0);
        all.extend_from_slice(&(bytes.len() as u64).to_le_bytes());
        all.extend_from_slice(&bytes);
    }
    Ok(sha256::sha256(&all))
}

/// Why the inputs could not be read.
#[derive(Debug)]
pub enum InputError {
    /// A file could not be read.
    Io(std::io::Error),
    /// `git` failed or is missing.
    Git(String),
}

impl std::fmt::Display for InputError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(e) => write!(f, "{e}"),
            Self::Git(e) => write!(f, "git: {e}"),
        }
    }
}

impl std::error::Error for InputError {}

fn git(root: &Path, args: &[&str]) -> Result<Vec<u8>, InputError> {
    let out = Command::new("git")
        .arg("-C")
        .arg(root)
        .args(args)
        .output()
        .map_err(|e| InputError::Git(e.to_string()))?;
    if !out.status.success() {
        return Err(InputError::Git(
            String::from_utf8_lossy(&out.stderr).trim().to_owned(),
        ));
    }
    Ok(out.stdout)
}

fn optional(p: &Path) -> Result<Option<Vec<u8>>, InputError> {
    match std::fs::read(p) {
        Ok(b) => Ok(Some(b)),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(e) => Err(InputError::Io(e)),
    }
}

/// What a changed or untracked path contributes beyond its path: a kind
/// byte, then a regular file's bytes (1), a symbolic link's target (2),
/// nothing for anything else, such as a nested checkout that git lists as
/// `dir/` (3), or nothing for a deleted file (0). The kind byte keeps the
/// four apart.
fn path_content(root: &Path, p: &[u8]) -> std::io::Result<Vec<u8>> {
    let path = root.join(path_from_git(p));
    let meta = match std::fs::symlink_metadata(&path) {
        Ok(m) => m,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(vec![0]),
        Err(e) => return Err(e),
    };
    if meta.is_file() {
        let mut v = vec![1];
        v.extend_from_slice(&std::fs::read(&path)?);
        Ok(v)
    } else if meta.file_type().is_symlink() {
        let mut v = vec![2];
        v.extend_from_slice(target_bytes(&std::fs::read_link(&path)?).as_slice());
        Ok(v)
    } else {
        Ok(vec![3])
    }
}

/// A path as git printed it. Git prints raw bytes on Unix, where any
/// bytes are a valid file name, and UTF-8 on Windows.
#[cfg(unix)]
fn path_from_git(p: &[u8]) -> std::path::PathBuf {
    use std::os::unix::ffi::OsStrExt as _;
    std::path::PathBuf::from(std::ffi::OsStr::from_bytes(p))
}

#[cfg(not(unix))]
fn path_from_git(p: &[u8]) -> std::path::PathBuf {
    std::path::PathBuf::from(String::from_utf8_lossy(p).into_owned())
}

/// A link target's bytes, exactly on Unix.
#[cfg(unix)]
fn target_bytes(t: &Path) -> Vec<u8> {
    use std::os::unix::ffi::OsStrExt as _;
    t.as_os_str().as_bytes().to_vec()
}

#[cfg(not(unix))]
fn target_bytes(t: &Path) -> Vec<u8> {
    t.to_string_lossy().as_bytes().to_vec()
}

/// Reads the inputs of the checkout at `root` for one build.
///
/// # Errors
/// [`InputError`] if a file or `git` fails.
pub fn collect(
    root: &Path,
    profile: &str,
    target: &str,
    channel: &str,
    version: &str,
) -> Result<Inputs, InputError> {
    let head = git(root, &["rev-parse", "HEAD"])?;
    let mut commit = String::from_utf8_lossy(&head).trim().to_owned();
    // Uncommitted changes make a different build from the same commit, so
    // they join the commit: each changed or untracked path, with what is
    // there now (its bytes, its link target, or that it is gone). Only the
    // list of paths comes from git, from commands whose output no git
    // setting changes: `status --porcelain` for tracked files (its format
    // is fixed, and the flags below override the settings that choose
    // what it lists) and `ls-files` with only the checkout's own
    // `.gitignore` files for untracked ones (not core.excludesFile or
    // .git/info/exclude, which differ between machines).
    let status = git(
        root,
        &[
            "status",
            "--porcelain=v1",
            "-z",
            "--untracked-files=no",
            "--no-renames",
            "--ignore-submodules=none",
        ],
    )?;
    let untracked = git(
        root,
        &[
            "ls-files",
            "--others",
            "--exclude-per-directory=.gitignore",
            "-z",
        ],
    )?;
    let mut paths: Vec<&[u8]> = status
        .split(|&b| b == 0)
        .filter_map(|e| e.get(3..))
        .chain(untracked.split(|&b| b == 0))
        .filter(|p| !p.is_empty())
        .collect();
    paths.sort_unstable();
    paths.dedup();
    let mut changes = Vec::new();
    for p in paths {
        changes.extend_from_slice(b"\0path\0");
        changes.extend_from_slice(p);
        changes.push(0);
        let content = path_content(root, p).map_err(InputError::Io)?;
        changes.extend_from_slice(&(content.len() as u64).to_le_bytes());
        changes.extend_from_slice(&content);
    }
    if !changes.is_empty() {
        commit.push_str("+dirty:");
        commit.push_str(&sha256::hex(&sha256::sha256(&changes)));
    }
    Ok(Inputs {
        commit,
        specs_lock: optional(&root.join("specs.lock"))?,
        downloads_lock: optional(&root.join("downloads.lock"))?,
        toolchain: std::fs::read(root.join("rust-toolchain.toml")).map_err(InputError::Io)?,
        profile: profile.to_owned(),
        target: target.to_owned(),
        channel: channel.to_owned(),
        version: version.to_owned(),
        schema: schema_hash(root).map_err(InputError::Io)?,
    })
}
