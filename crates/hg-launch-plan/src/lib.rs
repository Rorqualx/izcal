//! `hg-launch-plan`: the launch plan as data (M1 phase E).
//!
//! Layer L1, tier P, no dependencies (design rules E-R1, B2; owner decision 26 Oct). Shared by
//! the broker and the Linux spawner (`hg-substrate-os`'s
//! `hg-spawner-main` binary), so both read one table:
//!
//! - [`Kind`]: the process kinds the broker launches at M1, and for each
//!   its executable ([`Kind::exe`]) and its sandbox floor
//!   ([`Kind::floor`]). The request names only the kind; the plan and the
//!   executable are derived from it here (review minor, "Spawn{kind}").
//! - [`Tier`]: what a Linux launch achieved, in order. The spawner decides
//!   the session's tier once at its start; a spawn that cannot reach it
//!   fails, it never downgrades (review M3).
//! - The spawner's wire records ([`Request`], [`Reply`]): fixed-layout,
//!   little-endian, one record per `SOCK_SEQPACKET` datagram, with the
//!   number of attached descriptors each record must carry
//!   ([`Request::FDS`], [`Reply::fds`]). The spawner links no `hg-ipc`
//!   and speaks no `.hgidl` (security scope 6.3: "no argument parsing
//!   beyond a fixed request struct").
//!
//! Threading: pure functions, any thread.

#![no_std]

/// The process kinds the broker launches at M1 (Stage 1 8.1 M1; networking
/// NW-D3). The browser itself and the spawner are not launched by kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Kind {
    /// One site's renderer.
    Renderer,
    /// The network system service.
    NetworkSystem,
    /// A container's network service (unpartitioned cookies, DNS transport).
    NetworkContainer,
    /// A (top-level site, container)'s network service.
    NetworkPartition,
}

/// The executables the spawner holds an O_PATH descriptor for, opened at
/// its own start (G251; review M1: one descriptor per executable, no
/// build-directory descriptor).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Exe {
    /// `hg-renderer-main`.
    Renderer,
    /// `hg-net-main`.
    Net,
}

impl Exe {
    /// Every executable, in table order.
    pub const ALL: [Self; 2] = [Self::Renderer, Self::Net];

    /// The file name inside the build.
    #[must_use]
    pub const fn file_name(self) -> &'static str {
        match self {
            Self::Renderer => "hg-renderer-main",
            Self::Net => "hg-net-main",
        }
    }
}

/// What a Linux launch reached, weakest first. Every tier has seccomp and
/// Landlock; there is no tier without them (review M2: below the floor a
/// launch is refused).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Tier {
    /// Seccomp and Landlock, no namespaces: the host refuses unprivileged
    /// user namespaces.
    NoUserns,
    /// Seccomp, Landlock, and new user, PID and network namespaces, but
    /// no mount namespace and no id map: the host moves the child into a
    /// restricted user-namespace profile (launch facts E1–E4, E22). A
    /// **non-reference configuration** (review M4).
    RestrictedUserns,
    /// The security scope 6.7 row: plus a mount namespace with an empty
    /// root and id maps. **Unclaimed** until measured (E-Q1).
    Full,
}

impl Tier {
    /// Every tier, weakest first.
    pub const ALL: [Self; 3] = [Self::NoUserns, Self::RestrictedUserns, Self::Full];

    const fn code(self) -> u8 {
        match self {
            Self::NoUserns => 1,
            Self::RestrictedUserns => 2,
            Self::Full => 3,
        }
    }

    const fn from_code(c: u8) -> Option<Self> {
        match c {
            1 => Some(Self::NoUserns),
            2 => Some(Self::RestrictedUserns),
            3 => Some(Self::Full),
            _ => None,
        }
    }

    /// The tier's name in logs and the sandbox report.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::NoUserns => "linux-no-userns",
            Self::RestrictedUserns => "linux-restricted-userns",
            Self::Full => "linux-full",
        }
    }
}

impl Kind {
    /// Every kind, in code order.
    pub const ALL: [Self; 4] = [
        Self::Renderer,
        Self::NetworkSystem,
        Self::NetworkContainer,
        Self::NetworkPartition,
    ];

    const fn code(self) -> u8 {
        match self {
            Self::Renderer => 1,
            Self::NetworkSystem => 2,
            Self::NetworkContainer => 3,
            Self::NetworkPartition => 4,
        }
    }

    const fn from_code(c: u8) -> Option<Self> {
        match c {
            1 => Some(Self::Renderer),
            2 => Some(Self::NetworkSystem),
            3 => Some(Self::NetworkContainer),
            4 => Some(Self::NetworkPartition),
            _ => None,
        }
    }

    /// The executable this kind runs.
    #[must_use]
    pub const fn exe(self) -> Exe {
        match self {
            Self::Renderer => Exe::Renderer,
            Self::NetworkSystem | Self::NetworkContainer | Self::NetworkPartition => Exe::Net,
        }
    }

    /// Everything a launch of this kind is derived from, in one place:
    /// the executable, the sandbox floor and the argument token (E-R7
    /// disposition, 26 Oct). The spawner and every launcher take all of
    /// them from this one value, so no plan and token can disagree.
    #[must_use]
    pub const fn spec(self) -> Spec {
        Spec {
            kind: self,
            exe: self.exe(),
            floor: self.floor(),
            token: self.token(),
        }
    }

    /// The kind's argument token: the launched child's `argv[1]`, its only
    /// argument (one executable serves the three network kinds, and the
    /// child must name its kind in `ready` before `start` arrives).
    #[must_use]
    pub const fn token(self) -> &'static str {
        match self {
            Self::Renderer => "izk1",
            Self::NetworkSystem => "izk2",
            Self::NetworkContainer => "izk3",
            Self::NetworkPartition => "izk4",
        }
    }

    /// The kind whose token is exactly `t`, if `exe` runs it: a network
    /// token is refused by the renderer executable and the reverse.
    #[must_use]
    pub fn from_token(exe: Exe, t: &[u8]) -> Option<Self> {
        Self::ALL
            .into_iter()
            .find(|k| k.token().as_bytes() == t && k.exe() == exe)
    }

    /// The weakest tier this kind may run at: seccomp and Landlock are
    /// mandatory for every kind (review M2), which [`Tier::NoUserns`]
    /// already has. A spawner whose session tier is below a kind's floor
    /// refuses that kind.
    #[must_use]
    pub const fn floor(self) -> Tier {
        match self {
            Self::Renderer
            | Self::NetworkSystem
            | Self::NetworkContainer
            | Self::NetworkPartition => Tier::NoUserns,
        }
    }
}

/// One kind's launch, derived by [`Kind::spec`] and only there: its
/// fields are private, so a spec pairing one kind's executable or plan
/// with another kind's token cannot be built.
///
/// ```compile_fail
/// use hg_launch_plan::{Exe, Kind, Spec, Tier};
/// let _ = Spec { kind: Kind::Renderer, exe: Exe::Net, floor: Tier::NoUserns, token: "izk2" };
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Spec {
    kind: Kind,
    exe: Exe,
    floor: Tier,
    token: &'static str,
}

impl Spec {
    /// The kind.
    #[must_use]
    pub const fn kind(&self) -> Kind {
        self.kind
    }
    /// The executable.
    #[must_use]
    pub const fn exe(&self) -> Exe {
        self.exe
    }
    /// The sandbox floor.
    #[must_use]
    pub const fn floor(&self) -> Tier {
        self.floor
    }
    /// The child's `argv[1]`.
    #[must_use]
    pub const fn token(&self) -> &'static str {
        self.token
    }
}

/// Why a record was refused.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WireError {
    /// Not exactly the record's length.
    Length,
    /// Wrong magic or version.
    Header,
    /// An unknown operation.
    Op,
    /// An unknown kind, tier or stage.
    Value,
    /// A reserved byte is not zero.
    Reserved,
}

/// The broker's request magic, `IZSQ`.
pub const REQUEST_MAGIC: [u8; 4] = *b"IZSQ";
/// The spawner's reply magic, `IZSR`.
pub const REPLY_MAGIC: [u8; 4] = *b"IZSR";
/// The record format version.
pub const VERSION: u8 = 1;
/// A request's length in bytes.
pub const REQUEST_LEN: usize = 16;
/// A reply's length in bytes.
pub const REPLY_LEN: usize = 24;

/// Broker to spawner.
///
/// Layout: magic `IZSQ`; version u8; op u8 (1 = spawn); kind u8; reserved
/// u8 = 0; request id u64.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Request {
    /// Start one process of `kind`. Carries exactly [`Request::FDS`]
    /// descriptors: the child's two bootstrap ends, in that order (sender
    /// to the broker, then receiver from it).
    Spawn {
        /// Echoed in the reply.
        id: u64,
        /// What to start.
        kind: Kind,
    },
}

impl Request {
    /// Descriptors a spawn request carries.
    pub const FDS: usize = 2;

    /// The record.
    #[must_use]
    pub fn encode(self) -> [u8; REQUEST_LEN] {
        let Self::Spawn { id, kind } = self;
        let [m0, m1, m2, m3] = REQUEST_MAGIC;
        let [i0, i1, i2, i3, i4, i5, i6, i7] = id.to_le_bytes();
        [
            m0,
            m1,
            m2,
            m3,
            VERSION,
            1,
            kind.code(),
            0,
            i0,
            i1,
            i2,
            i3,
            i4,
            i5,
            i6,
            i7,
        ]
    }

    /// Parses a record.
    ///
    /// # Errors
    /// [`WireError`] for anything but an exact, known record.
    pub fn decode(bytes: &[u8]) -> Result<Self, WireError> {
        let b: &[u8; REQUEST_LEN] = bytes.try_into().map_err(|_e| WireError::Length)?;
        let [
            m0,
            m1,
            m2,
            m3,
            ver,
            op,
            kind,
            rsv,
            i0,
            i1,
            i2,
            i3,
            i4,
            i5,
            i6,
            i7,
        ] = *b;
        if [m0, m1, m2, m3] != REQUEST_MAGIC || ver != VERSION {
            return Err(WireError::Header);
        }
        if op != 1 {
            return Err(WireError::Op);
        }
        if rsv != 0 {
            return Err(WireError::Reserved);
        }
        let kind = Kind::from_code(kind).ok_or(WireError::Value)?;
        Ok(Self::Spawn {
            id: u64::from_le_bytes([i0, i1, i2, i3, i4, i5, i6, i7]),
            kind,
        })
    }
}

/// Where a failed spawn stopped.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Stage {
    /// The request was refused before any process was made (wrong
    /// descriptors, truncated control data, a kind above the session).
    Request,
    /// `clone3` failed.
    Clone,
    /// Placing the bootstrap ends or closing the rest failed.
    Descriptors,
    /// Landlock failed.
    Landlock,
    /// Installing the launch seccomp filter failed.
    Seccomp,
    /// The exec failed.
    Exec,
    /// The child reported a stage the spawner does not know.
    Unknown,
}

impl Stage {
    const fn code(self) -> u8 {
        match self {
            Self::Request => 1,
            Self::Clone => 2,
            Self::Descriptors => 3,
            Self::Landlock => 4,
            Self::Seccomp => 5,
            Self::Exec => 6,
            Self::Unknown => 7,
        }
    }

    const fn from_code(c: u8) -> Option<Self> {
        match c {
            1 => Some(Self::Request),
            2 => Some(Self::Clone),
            3 => Some(Self::Descriptors),
            4 => Some(Self::Landlock),
            5 => Some(Self::Seccomp),
            6 => Some(Self::Exec),
            7 => Some(Self::Unknown),
            _ => None,
        }
    }
}

/// Spawner to broker.
///
/// Layout: magic `IZSR`; version u8; op u8; a u8; reserved u8 = 0; id
/// u64; w u32; x u32. `Spawned`: op 1, a = tier, w = outer pid, x = pid in
/// the child's namespace. `Failed`: op 2, a = stage, w = errno, x = 0.
/// `Ready`: op 3, a = the session tier, id = 0, w = x = 0.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Reply {
    /// The spawner started and decided the session's tier (review M3).
    Ready {
        /// What every spawn of this session applies.
        tier: Tier,
    },
    /// The child exec'd. Carries one descriptor: its pidfd.
    Spawned {
        /// The request's id.
        id: u64,
        /// The tier the spawner applied (review M3: the child's own report
        /// is a cross-check only).
        tier: Tier,
        /// The child's pid in the broker's namespace.
        outer_pid: u32,
        /// The child's pid in its own namespace (1 with a PID namespace).
        ns_pid: u32,
    },
    /// No child runs. Carries no descriptor.
    Failed {
        /// The request's id.
        id: u64,
        /// Where it stopped.
        stage: Stage,
        /// The OS error, 0 when there is none.
        errno: u32,
    },
}

impl Reply {
    /// Descriptors this reply carries.
    #[must_use]
    pub const fn fds(&self) -> usize {
        match self {
            Self::Spawned { .. } => 1,
            Self::Ready { .. } | Self::Failed { .. } => 0,
        }
    }

    /// The record.
    #[must_use]
    pub fn encode(self) -> [u8; REPLY_LEN] {
        let (op, a, id, w, x) = match self {
            Self::Ready { tier } => (3, tier.code(), 0, 0, 0),
            Self::Spawned {
                id,
                tier,
                outer_pid,
                ns_pid,
            } => (1, tier.code(), id, outer_pid, ns_pid),
            Self::Failed { id, stage, errno } => (2, stage.code(), id, errno, 0),
        };
        let [m0, m1, m2, m3] = REPLY_MAGIC;
        let [i0, i1, i2, i3, i4, i5, i6, i7] = u64::to_le_bytes(id);
        let [w0, w1, w2, w3] = u32::to_le_bytes(w);
        let [x0, x1, x2, x3] = u32::to_le_bytes(x);
        [
            m0, m1, m2, m3, VERSION, op, a, 0, i0, i1, i2, i3, i4, i5, i6, i7, w0, w1, w2, w3, x0,
            x1, x2, x3,
        ]
    }

    /// Parses a record.
    ///
    /// # Errors
    /// [`WireError`] for anything but an exact, known record.
    pub fn decode(bytes: &[u8]) -> Result<Self, WireError> {
        let b: &[u8; REPLY_LEN] = bytes.try_into().map_err(|_e| WireError::Length)?;
        let [
            m0,
            m1,
            m2,
            m3,
            ver,
            op,
            a,
            rsv,
            i0,
            i1,
            i2,
            i3,
            i4,
            i5,
            i6,
            i7,
            w0,
            w1,
            w2,
            w3,
            x0,
            x1,
            x2,
            x3,
        ] = *b;
        if [m0, m1, m2, m3] != REPLY_MAGIC || ver != VERSION {
            return Err(WireError::Header);
        }
        if rsv != 0 {
            return Err(WireError::Reserved);
        }
        let id = u64::from_le_bytes([i0, i1, i2, i3, i4, i5, i6, i7]);
        let w = u32::from_le_bytes([w0, w1, w2, w3]);
        let x = u32::from_le_bytes([x0, x1, x2, x3]);
        match op {
            1 => Ok(Self::Spawned {
                id,
                tier: Tier::from_code(a).ok_or(WireError::Value)?,
                outer_pid: w,
                ns_pid: x,
            }),
            2 if x == 0 => Ok(Self::Failed {
                id,
                stage: Stage::from_code(a).ok_or(WireError::Value)?,
                errno: w,
            }),
            3 if id == 0 && w == 0 && x == 0 => Ok(Self::Ready {
                tier: Tier::from_code(a).ok_or(WireError::Value)?,
            }),
            2 | 3 => Err(WireError::Reserved),
            _ => Err(WireError::Op),
        }
    }
}

#[cfg(kani)]
mod proofs {
    use super::{Exe, Kind, REPLY_LEN, REQUEST_LEN, Reply, Request, Tier};

    /// The request decoder is total, and whatever it accepts re-encodes to
    /// the same bytes (the layout has no slack).
    #[kani::proof]
    fn request_decode_is_total_and_exact() {
        let b: [u8; REQUEST_LEN] = kani::any();
        if let Ok(r) = Request::decode(&b) {
            assert!(r.encode() == b);
            kani::cover!(true, "a request decodes");
        }
    }

    /// The reply decoder is total and exact.
    #[kani::proof]
    fn reply_decode_is_total_and_exact() {
        let b: [u8; REPLY_LEN] = kani::any();
        if let Ok(r) = Reply::decode(&b) {
            assert!(r.encode() == b);
            kani::cover!(
                matches!(r, Reply::Spawned { .. }),
                "a spawned reply decodes"
            );
            kani::cover!(matches!(r, Reply::Failed { .. }), "a failed reply decodes");
            kani::cover!(matches!(r, Reply::Ready { .. }), "a ready reply decodes");
        }
    }

    /// Any other length is refused.
    #[kani::proof]
    fn wrong_lengths_are_refused() {
        let b: [u8; 25] = kani::any();
        let n: usize = kani::any();
        kani::assume(n <= 25 && n != REQUEST_LEN);
        assert!(Request::decode(b.get(..n).unwrap_or(&[])).is_err());
        let m: usize = kani::any();
        kani::assume(m <= 25 && m != REPLY_LEN);
        assert!(Reply::decode(b.get(..m).unwrap_or(&[])).is_err());
    }

    /// Every encodable request decodes back to itself.
    #[kani::proof]
    fn requests_round_trip() {
        let k: u8 = kani::any();
        kani::assume(usize::from(k) < Kind::ALL.len());
        let kind = Kind::ALL
            .get(usize::from(k))
            .copied()
            .unwrap_or(Kind::Renderer);
        let r = Request::Spawn {
            id: kani::any(),
            kind,
        };
        assert!(Request::decode(&r.encode()) == Ok(r));
    }

    /// E-R7 disposition: kind → token → kind round-trips through the
    /// kind's own executable, never through the other one, and the tokens
    /// are pairwise distinct.
    #[kani::proof]
    #[kani::unwind(6)]
    fn tokens_round_trip_and_are_distinct() {
        let a: u8 = kani::any();
        let b: u8 = kani::any();
        kani::assume(usize::from(a) < Kind::ALL.len() && usize::from(b) < Kind::ALL.len());
        let ka = Kind::ALL
            .get(usize::from(a))
            .copied()
            .unwrap_or(Kind::Renderer);
        let kb = Kind::ALL
            .get(usize::from(b))
            .copied()
            .unwrap_or(Kind::Renderer);
        assert!(Kind::from_token(ka.exe(), ka.token().as_bytes()) == Some(ka));
        let other = if ka.exe() == Exe::Renderer {
            Exe::Net
        } else {
            Exe::Renderer
        };
        assert!(Kind::from_token(other, ka.token().as_bytes()).is_none());
        if ka != kb {
            assert!(ka.token() != kb.token());
        }
        let s = ka.spec();
        assert!(s.kind() == ka && s.exe() == ka.exe() && s.token() == ka.token());
    }

    /// The kind-to-executable map and the floors (review minor: the plan
    /// and executable derive from the kind): network kinds run
    /// `hg-net-main`, the renderer `hg-renderer-main`, and no kind's floor
    /// is above the weakest tier with seccomp and Landlock.
    #[kani::proof]
    fn kinds_map_to_their_executables() {
        let k: u8 = kani::any();
        kani::assume(usize::from(k) < Kind::ALL.len());
        let kind = Kind::ALL
            .get(usize::from(k))
            .copied()
            .unwrap_or(Kind::Renderer);
        let exe = kind.exe();
        assert!((kind == Kind::Renderer) == (exe == Exe::Renderer));
        assert!(kind.floor() == Tier::NoUserns);
    }
}
