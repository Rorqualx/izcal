//! The shared recording and crash-report header (`RecordingHeader`).
//!
//! One definition, owned by FI-11 and placed here, crypto-free, so every
//! producer and reader shares it (enterprise policy 6.21 item 1; OS
//! integration OI-D26; compliance 9.4; Stage 1 8.1's M1 exit:
//! "recordings carry the engine build ID, the data versions and the
//! decision-point log"). Its fields:
//!
//! | Field | Recordings | Crash reports |
//! |---|---|---|
//! | Header schema version | yes | yes |
//! | Engine build ID (keys the signed build manifest: version, release line, channel, platform, ISA, tier ceiling, SBOM digest) | yes | yes |
//! | Data versions | yes | yes |
//! | Decision-point log reference | yes | no |
//! | OS version | yes | yes |
//! | Process kind | yes | yes |
//! | Sandbox tier (the one achieved) | yes | yes |
//! | Policy: [`PolicyDigest`] in recordings, [`PolicyFlags`] in crash reports | digest | flags |
//! | Profile identity, replaced on every export | yes | no |
//!
//! Channel is not a field: the build manifest gives it. A header without
//! the fields its kind requires is refused when built and when decoded.
//!
//! FI-11 has no spec yet, so the value types below are this crate's:
//! where a spec fixes a size (the 32-byte digest, the u64 flag set, the
//! u64 build ID) it is used; the other bounds are marked †.
//!
//! # Encoding
//!
//! Little-endian; [`RecordingHeader::encode`] and
//! [`RecordingHeader::decode`]:
//!
//! | Bytes | Field |
//! |---|---|
//! | 4 | magic `IZRH` |
//! | 2 | schema version ([`HEADER_SCHEMA_VERSION`]) |
//! | 1 | kind: 1 recording, 2 crash report |
//! | 8 | build ID |
//! | 1 + … | data versions: a count (at most [`MAX_DATA_VERSIONS`]), each a name (u8 length, at most [`MAX_NAME`] bytes) and a u64 version |
//! | 1 + 16 | decision-point log: presence, then trace hash u64 and entry count u64 |
//! | 1 + … | OS version: u8 length (at most [`MAX_NAME`] bytes) and text |
//! | 1 | process kind ([`ProcessKind`]) |
//! | 1 | sandbox tier ([`SandboxTier`]) |
//! | 1 + … | policy: 1 and 32 digest bytes, or 2 and a u64 flag set |
//! | 1 + 16 | profile identity: presence, then 16 bytes |
//!
//! Names and the OS version are printable ASCII.

pub use text::Text;

/// The schema version this crate writes and reads.
pub const HEADER_SCHEMA_VERSION: u16 = 1;
/// `IZRH`.
const MAGIC: [u8; 4] = *b"IZRH";
/// The most data versions in one header†.
pub const MAX_DATA_VERSIONS: usize = 32;
/// The most bytes in a data-set name or the OS version†.
pub const MAX_NAME: usize = 64;

/// `SHA-256(canonical_policy_encoding)` of the effective policy
/// (enterprise policy 6.21: `izcal-policy-v1`), computed by `hg-policy`
/// with `hg-crypto` in the broker. This crate only carries the bytes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PolicyDigest(pub [u8; 32]);

/// The digest of the empty entry set with the catalogue version fixed
/// at 0: what an unmanaged machine's header always carries, and every
/// M1 recording (M1 has no policy sources). It is the SHA-256 of the
/// tag `izcal-policy-v1` followed by the catalogue version as a u32
/// little-endian, with no entries; `hg-build-id`'s test recomputes it.
pub const NO_POLICY: PolicyDigest = PolicyDigest([
    0x9c, 0xee, 0xe9, 0x08, 0x7a, 0x63, 0xf0, 0xce, 0x85, 0xb1, 0x2b, 0x63, 0xe2, 0x3f, 0xbf, 0x9e,
    0xdd, 0x43, 0x59, 0xd4, 0xab, 0xe8, 0x50, 0xb4, 0xf4, 0xdc, 0xa7, 0x97, 0x92, 0x97, 0xc7, 0x7f,
]);

/// Coarse policy flags for crash reports: one bit per class of
/// risk-relevant policy that is set (enterprise policy 6.21, the
/// catalogue's `risk_flag`). All zero with no policy.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PolicyFlags(pub u64);

/// The policy field: the digest in recordings, the flags in crash
/// reports.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum HeaderPolicy {
    /// Recordings.
    Digest(PolicyDigest),
    /// Crash reports.
    Flags(PolicyFlags),
}

/// Which process wrote the recording or crashed. Closed; a new kind is a
/// schema version.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ProcessKind {
    /// The browser process (the broker).
    Browser,
    /// The Linux spawner.
    Spawner,
    /// A renderer.
    Renderer,
    /// A network service.
    Network,
    /// Every simulated process at once: a DST run's recording.
    Simulation,
}

/// The sandbox tier the process achieved (security scope 6.7: reported,
/// never silently lowered). Closed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SandboxTier {
    /// Not sandboxed: the broker and the spawner.
    Unsandboxed,
    /// The renderer's tier: zero ambient authority.
    ZeroAmbient,
    /// The GPU tier.
    Gpu,
    /// The network tier.
    Network,
    /// The strictest tier.
    Strictest,
    /// A lower tier than the kind's, where the OS lacked a mechanism.
    Reduced,
    /// The simulator: no OS sandbox applies.
    Simulated,
}

/// One data set's name and version (a list, a model, a snapshot).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DataVersion {
    /// The data set.
    pub name: Text,
    /// Its version.
    pub version: u64,
}

/// Where the decision-point log is: in M1, the DST trace it hashes to
/// and its length.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct DecisionLogRef {
    /// The log's 64-bit trace hash.
    pub trace_hash: u64,
    /// How many entries it holds.
    pub entries: u64,
}

/// A per-installation random value, replaced on every export or upload,
/// so it never leaves the device ([`RecordingHeader::for_export`]).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ProfileIdentity(pub [u8; 16]);

/// What a header is for.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum HeaderKind {
    /// A recording.
    Recording,
    /// A crash report.
    CrashReport,
}

/// The shared header.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RecordingHeader {
    kind: HeaderKind,
    build_id: u64,
    data_versions: Vec<DataVersion>,
    decision_log: Option<DecisionLogRef>,
    os_version: Text,
    process: ProcessKind,
    sandbox: SandboxTier,
    policy: HeaderPolicy,
    profile: Option<ProfileIdentity>,
}

/// Why a header was refused.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum HeaderError {
    /// It does not decode, or an unknown code.
    Malformed,
    /// Another schema version.
    Version,
    /// Over a bound.
    TooLarge,
    /// A recording without its data versions, decision-point log
    /// reference, policy digest or profile identity; a crash report with
    /// a decision log, a digest or a profile identity.
    WrongFields,
}

/// The common fields of both kinds.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Common {
    /// This build's ID (`crate::BUILD_ID` for the running build).
    pub build_id: u64,
    /// The data sets the process used, at most [`MAX_DATA_VERSIONS`];
    /// empty when it used none.
    pub data_versions: Vec<DataVersion>,
    /// The OS version.
    pub os_version: Text,
    /// The process.
    pub process: ProcessKind,
    /// Its achieved sandbox tier.
    pub sandbox: SandboxTier,
}

impl RecordingHeader {
    /// A recording's header.
    ///
    /// # Errors
    /// [`HeaderError::TooLarge`] for too many data versions.
    pub fn recording(
        common: Common,
        decision_log: DecisionLogRef,
        policy: PolicyDigest,
        profile: ProfileIdentity,
    ) -> Result<Self, HeaderError> {
        Self::checked(Self {
            kind: HeaderKind::Recording,
            build_id: common.build_id,
            data_versions: common.data_versions,
            decision_log: Some(decision_log),
            os_version: common.os_version,
            process: common.process,
            sandbox: common.sandbox,
            policy: HeaderPolicy::Digest(policy),
            profile: Some(profile),
        })
    }

    /// A crash report's header.
    ///
    /// # Errors
    /// [`HeaderError::TooLarge`] for too many data versions.
    pub fn crash_report(common: Common, policy: PolicyFlags) -> Result<Self, HeaderError> {
        Self::checked(Self {
            kind: HeaderKind::CrashReport,
            build_id: common.build_id,
            data_versions: common.data_versions,
            decision_log: None,
            os_version: common.os_version,
            process: common.process,
            sandbox: common.sandbox,
            policy: HeaderPolicy::Flags(policy),
            profile: None,
        })
    }

    fn checked(h: Self) -> Result<Self, HeaderError> {
        if h.data_versions.len() > MAX_DATA_VERSIONS {
            return Err(HeaderError::TooLarge);
        }
        let ok = match h.kind {
            HeaderKind::Recording => {
                h.decision_log.is_some()
                    && matches!(h.policy, HeaderPolicy::Digest(_))
                    && h.profile.is_some()
            }
            HeaderKind::CrashReport => {
                h.decision_log.is_none()
                    && matches!(h.policy, HeaderPolicy::Flags(_))
                    && h.profile.is_none()
            }
        };
        if ok {
            Ok(h)
        } else {
            Err(HeaderError::WrongFields)
        }
    }

    /// The same header with the profile identity replaced by `fresh`, a
    /// new random value, as every export or upload requires.
    #[must_use]
    pub fn for_export(&self, fresh: ProfileIdentity) -> Self {
        let mut h = self.clone();
        if h.profile.is_some() {
            h.profile = Some(fresh);
        }
        h
    }

    /// The kind.
    pub const fn kind(&self) -> HeaderKind {
        self.kind
    }
    /// The build ID.
    pub const fn build_id(&self) -> u64 {
        self.build_id
    }
    /// The data versions.
    pub fn data_versions(&self) -> &[DataVersion] {
        &self.data_versions
    }
    /// The decision-point log, for a recording.
    pub const fn decision_log(&self) -> Option<DecisionLogRef> {
        self.decision_log
    }
    /// The OS version.
    pub const fn os_version(&self) -> &Text {
        &self.os_version
    }
    /// The process kind.
    pub const fn process(&self) -> ProcessKind {
        self.process
    }
    /// The sandbox tier.
    pub const fn sandbox(&self) -> SandboxTier {
        self.sandbox
    }
    /// The policy field.
    pub const fn policy(&self) -> HeaderPolicy {
        self.policy
    }
    /// The profile identity, for a recording.
    pub const fn profile(&self) -> Option<ProfileIdentity> {
        self.profile
    }

    /// The encoding (module docs).
    pub fn encode(&self) -> Vec<u8> {
        let mut o = Vec::new();
        o.extend_from_slice(&MAGIC);
        o.extend_from_slice(&HEADER_SCHEMA_VERSION.to_le_bytes());
        o.push(match self.kind {
            HeaderKind::Recording => 1,
            HeaderKind::CrashReport => 2,
        });
        o.extend_from_slice(&self.build_id.to_le_bytes());
        // `checked` bounds the count by MAX_DATA_VERSIONS (32).
        o.push(u8::try_from(self.data_versions.len()).unwrap_or(u8::MAX));
        for d in &self.data_versions {
            d.name.put(&mut o);
            o.extend_from_slice(&d.version.to_le_bytes());
        }
        match self.decision_log {
            None => o.push(0),
            Some(l) => {
                o.push(1);
                o.extend_from_slice(&l.trace_hash.to_le_bytes());
                o.extend_from_slice(&l.entries.to_le_bytes());
            }
        }
        self.os_version.put(&mut o);
        o.push(match self.process {
            ProcessKind::Browser => 1,
            ProcessKind::Spawner => 2,
            ProcessKind::Renderer => 3,
            ProcessKind::Network => 4,
            ProcessKind::Simulation => 5,
        });
        o.push(match self.sandbox {
            SandboxTier::Unsandboxed => 1,
            SandboxTier::ZeroAmbient => 2,
            SandboxTier::Gpu => 3,
            SandboxTier::Network => 4,
            SandboxTier::Strictest => 5,
            SandboxTier::Reduced => 6,
            SandboxTier::Simulated => 7,
        });
        match self.policy {
            HeaderPolicy::Digest(d) => {
                o.push(1);
                o.extend_from_slice(&d.0);
            }
            HeaderPolicy::Flags(f) => {
                o.push(2);
                o.extend_from_slice(&f.0.to_le_bytes());
            }
        }
        match self.profile {
            None => o.push(0),
            Some(p) => {
                o.push(1);
                o.extend_from_slice(&p.0);
            }
        }
        o
    }

    /// Decodes a header that must fill `bytes` exactly.
    ///
    /// # Errors
    /// [`HeaderError`]; a recording missing a required field is
    /// [`HeaderError::WrongFields`].
    pub fn decode(bytes: &[u8]) -> Result<Self, HeaderError> {
        let mut r = Reader { b: bytes };
        if r.take::<4>()? != MAGIC {
            return Err(HeaderError::Malformed);
        }
        if u16::from_le_bytes(r.take()?) != HEADER_SCHEMA_VERSION {
            return Err(HeaderError::Version);
        }
        let kind = match r.byte()? {
            1 => HeaderKind::Recording,
            2 => HeaderKind::CrashReport,
            _ => return Err(HeaderError::Malformed),
        };
        let build_id = u64::from_le_bytes(r.take()?);
        let n = usize::from(r.byte()?);
        if n > MAX_DATA_VERSIONS {
            return Err(HeaderError::TooLarge);
        }
        let mut data_versions = Vec::with_capacity(n);
        for _ in 0..n {
            let name = Text::get(&mut r)?;
            let version = u64::from_le_bytes(r.take()?);
            data_versions.push(DataVersion { name, version });
        }
        let decision_log = match r.byte()? {
            0 => None,
            1 => Some(DecisionLogRef {
                trace_hash: u64::from_le_bytes(r.take()?),
                entries: u64::from_le_bytes(r.take()?),
            }),
            _ => return Err(HeaderError::Malformed),
        };
        let os_version = Text::get(&mut r)?;
        let process = match r.byte()? {
            1 => ProcessKind::Browser,
            2 => ProcessKind::Spawner,
            3 => ProcessKind::Renderer,
            4 => ProcessKind::Network,
            5 => ProcessKind::Simulation,
            _ => return Err(HeaderError::Malformed),
        };
        let sandbox = match r.byte()? {
            1 => SandboxTier::Unsandboxed,
            2 => SandboxTier::ZeroAmbient,
            3 => SandboxTier::Gpu,
            4 => SandboxTier::Network,
            5 => SandboxTier::Strictest,
            6 => SandboxTier::Reduced,
            7 => SandboxTier::Simulated,
            _ => return Err(HeaderError::Malformed),
        };
        let policy = match r.byte()? {
            1 => HeaderPolicy::Digest(PolicyDigest(r.take()?)),
            2 => HeaderPolicy::Flags(PolicyFlags(u64::from_le_bytes(r.take()?))),
            _ => return Err(HeaderError::Malformed),
        };
        let profile = match r.byte()? {
            0 => None,
            1 => Some(ProfileIdentity(r.take()?)),
            _ => return Err(HeaderError::Malformed),
        };
        if !r.b.is_empty() {
            return Err(HeaderError::Malformed);
        }
        Self::checked(Self {
            kind,
            build_id,
            data_versions,
            decision_log,
            os_version,
            process,
            sandbox,
            policy,
            profile,
        })
    }
}

/// Reads the encoding front to back.
struct Reader<'a> {
    b: &'a [u8],
}

impl Reader<'_> {
    fn take<const N: usize>(&mut self) -> Result<[u8; N], HeaderError> {
        let (head, rest) = self
            .b
            .split_first_chunk::<N>()
            .ok_or(HeaderError::Malformed)?;
        self.b = rest;
        Ok(*head)
    }

    fn byte(&mut self) -> Result<u8, HeaderError> {
        let [b] = self.take::<1>()?;
        Ok(b)
    }

    fn bytes(&mut self, n: usize) -> Result<&[u8], HeaderError> {
        let (head, rest) = self.b.split_at_checked(n).ok_or(HeaderError::Malformed)?;
        self.b = rest;
        Ok(head)
    }
}

/// Bounded printable ASCII text, for names and the OS version.
mod text {
    use super::{HeaderError, MAX_NAME, Reader};

    /// At most [`MAX_NAME`] bytes of printable ASCII (0x20 to 0x7E).
    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    pub struct Text(String);

    impl Text {
        /// `s`, if it is printable ASCII of at most [`MAX_NAME`] bytes.
        ///
        /// # Errors
        /// [`HeaderError::TooLarge`] if longer; [`HeaderError::Malformed`]
        /// for any other byte.
        pub fn new(s: &str) -> Result<Self, HeaderError> {
            if s.len() > MAX_NAME {
                return Err(HeaderError::TooLarge);
            }
            if !s.bytes().all(|b| (0x20..=0x7e).contains(&b)) {
                return Err(HeaderError::Malformed);
            }
            Ok(Self(s.to_owned()))
        }

        /// The text.
        pub fn as_str(&self) -> &str {
            &self.0
        }

        pub(super) fn put(&self, o: &mut Vec<u8>) {
            // `new` bounds the length by MAX_NAME (64).
            o.push(u8::try_from(self.0.len()).unwrap_or(u8::MAX));
            o.extend_from_slice(self.0.as_bytes());
        }

        pub(super) fn get(r: &mut Reader<'_>) -> Result<Self, HeaderError> {
            let n = usize::from(r.byte()?);
            let b = r.bytes(n)?;
            let s = core::str::from_utf8(b).map_err(|_e| HeaderError::Malformed)?;
            Self::new(s)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn common() -> Common {
        Common {
            build_id: crate::BUILD_ID,
            data_versions: vec![DataVersion {
                name: Text::new("psl").unwrap(),
                version: 3,
            }],
            os_version: Text::new("Linux 6.8.0").unwrap(),
            process: ProcessKind::Renderer,
            sandbox: SandboxTier::ZeroAmbient,
        }
    }

    fn log() -> DecisionLogRef {
        DecisionLogRef {
            trace_hash: 0x0123_4567_89ab_cdef,
            entries: 508,
        }
    }

    #[test]
    fn both_kinds_round_trip() {
        let r = RecordingHeader::recording(common(), log(), NO_POLICY, ProfileIdentity([9; 16]))
            .unwrap();
        assert_eq!(RecordingHeader::decode(&r.encode()), Ok(r));
        let c = RecordingHeader::crash_report(common(), PolicyFlags(0)).unwrap();
        assert_eq!(RecordingHeader::decode(&c.encode()), Ok(c));
    }

    #[test]
    fn a_recording_missing_a_required_field_is_refused() {
        let r = RecordingHeader::recording(common(), log(), NO_POLICY, ProfileIdentity([9; 16]))
            .unwrap();
        let good = r.encode();
        // The decision-log presence byte follows the magic (4), version
        // (2), kind (1), build ID (8), one data version (1 + 1 + 3 + 8).
        let at = 4 + 2 + 1 + 8 + 1 + 1 + 3 + 8;
        assert_eq!(good[at], 1);
        let mut no_log: Vec<u8> = good[..at].to_vec();
        no_log.push(0);
        no_log.extend_from_slice(&good[at + 17..]);
        assert_eq!(
            RecordingHeader::decode(&no_log),
            Err(HeaderError::WrongFields)
        );
        // A recording with the crash-report policy.
        let crash = RecordingHeader::crash_report(common(), PolicyFlags(1)).unwrap();
        let mut mixed = crash.encode();
        mixed[6] = 1;
        assert_eq!(
            RecordingHeader::decode(&mixed),
            Err(HeaderError::WrongFields)
        );
    }

    #[test]
    fn export_replaces_the_profile_identity() {
        let r = RecordingHeader::recording(common(), log(), NO_POLICY, ProfileIdentity([9; 16]))
            .unwrap();
        let e = r.for_export(ProfileIdentity([1; 16]));
        assert_eq!(e.profile(), Some(ProfileIdentity([1; 16])));
        assert_eq!(e.build_id(), r.build_id());
    }

    #[test]
    fn bounds_and_codes_are_checked() {
        assert_eq!(Text::new(&"a".repeat(65)), Err(HeaderError::TooLarge));
        assert_eq!(Text::new("a\nb"), Err(HeaderError::Malformed));
        let mut c = common();
        c.data_versions = vec![
            DataVersion {
                name: Text::new("x").unwrap(),
                version: 0,
            };
            33
        ];
        assert_eq!(
            RecordingHeader::crash_report(c, PolicyFlags(0)),
            Err(HeaderError::TooLarge)
        );
        let good = RecordingHeader::crash_report(common(), PolicyFlags(0))
            .unwrap()
            .encode();
        let mut v2 = good.clone();
        v2[4] = 2;
        assert_eq!(RecordingHeader::decode(&v2), Err(HeaderError::Version));
        let mut long = good.clone();
        long.push(0);
        assert_eq!(RecordingHeader::decode(&long), Err(HeaderError::Malformed));
        for n in 0..good.len() {
            assert!(RecordingHeader::decode(&good[..n]).is_err());
        }
    }
}
