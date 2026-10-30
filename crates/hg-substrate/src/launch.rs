//! Launching child processes (M1 phase E; footprint FP-D1, security scope
//! 6.3 and 6.7, distribution G251).
//!
//! Only the broker has a [`Launcher`] (`Substrate::launcher` is `Some`
//! there and `None` everywhere else). It starts one process of a
//! [`ChildKind`] from the broker's own build, in that kind's sandbox, and
//! returns the broker's ends of the child's bootstrap channels plus the
//! child's identity. A child cannot launch anything.
//!
//! **Identity (design review M6, M8).** [`Launched::peer`] is the OS
//! identity the broker binds the child's channel to
//! ([`crate::Ipc::listen_bound`]): on Linux the pid as the broker sees
//! it, on macOS the audit token's pid and version, on Windows the
//! process. Its generation part is the launcher's own label, a number
//! that never repeats in one broker's life: **the kernel never attests
//! it** (Linux pidfds have no unique inode on 6.8, launch facts E24). It
//! keeps two launches apart in logs and in simulation; the kernel-attested
//! part is the pid (and, on macOS, the audit token's version).
//!
//! On the child's side, identity is always the **channel binding the
//! broker issued**: a child in a PID namespace reads every other process
//! as pid 0 (launch facts E12, E13, E23), so it never attests a peer by
//! pid. Its bootstrap channel came to it at birth and only the broker
//! holds the other end; every other channel a child holds was granted by
//! the broker, bound to the peer the broker named.

use crate::ipc::{ChannelReceiver, ChannelSender, OsPeer};

/// The kinds the broker launches at M1 (Stage 1 design M1 exit;
/// networking NW-D3). The same list as `hg_launch_plan::Kind`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ChildKind {
    /// One site's renderer.
    Renderer,
    /// The network system service.
    NetworkSystem,
    /// A container's network service.
    NetworkContainer,
    /// A (top-level site, container)'s network service.
    NetworkPartition,
}

impl ChildKind {
    /// Every kind.
    pub const ALL: [Self; 4] = [
        Self::Renderer,
        Self::NetworkSystem,
        Self::NetworkContainer,
        Self::NetworkPartition,
    ];
}

/// The sandbox a child runs in, by OS (security scope 6.7; SEC-8). The
/// same list as `process.hgidl`'s `SandboxTier`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SandboxTier {
    /// Linux: seccomp and Landlock, no namespaces.
    LinuxNoUserns,
    /// Linux: seccomp, Landlock, user, PID and network namespaces, no
    /// mount namespace (a non-reference configuration).
    LinuxRestrictedUserns,
    /// Linux: the security scope 6.7 row.
    LinuxFull,
    /// macOS: the deny-default SBPL profile.
    MacosSbpl,
    /// Windows: AppContainer, lockdown token and job (the M1 subset).
    WindowsAppContainer,
    /// Under the simulator: no OS sandbox exists.
    Simulated,
}

/// A part of the 6.7 row a child did not reach. The same list as
/// `process.hgidl`'s `SandboxFeature`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SandboxFeature {
    /// No empty root (Linux).
    EmptyRoot,
    /// No user-namespace id map (Linux).
    IdMap,
    /// No mount namespace (Linux).
    MountNamespace,
    /// No Landlock UDP rules (Linux).
    LandlockUdp,
    /// No alternate desktop (Windows, M5).
    AlternateDesktop,
    /// No win32k lockdown (Windows, M5).
    Win32kLockdown,
    /// No code integrity guard (Windows, M5).
    CodeIntegrityGuard,
}

/// What a child's own black-box probes found after lockdown (E-R5): a
/// cross-check of what its launcher applied (design review M3).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SandboxReached {
    /// The tier the probes show.
    pub tier: SandboxTier,
    /// What the 6.7 row has that this tier lacks.
    pub missing: Vec<SandboxFeature>,
}

/// A child that started.
#[derive(Debug)]
pub struct Launched {
    /// The broker's receiver of the child's bootstrap sender.
    pub from_child: ChannelReceiver,
    /// The broker's sender to the child's bootstrap receiver.
    pub to_child: ChannelSender,
    /// The child's identity, for [`crate::Ipc::listen_bound`].
    pub peer: OsPeer,
    /// The sandbox tier the launcher applied (design review M3): the
    /// child's own report must name the same tier, else it is killed.
    pub sandbox: SandboxTier,
}

/// Why no child started.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LaunchError {
    /// This process cannot launch (only the broker can), or this kind is
    /// not launchable on this OS or substrate.
    Unsupported,
    /// The launch would run below the kind's sandbox floor, or below the
    /// session's tier (design review M2, M3): refused, never downgraded.
    BelowFloor,
    /// The OS refused (process creation, sandbox setup or exec failed).
    Failed,
    /// The launcher is in the restart-required state (distribution DU-3):
    /// no further launches of any kind.
    RestartRequired,
}

/// Starts child processes. The broker's only.
pub trait Launcher: Send + Sync {
    /// Starts one process of `kind`. The child's bootstrap channels hold
    /// up to `capacity` undelivered messages each.
    ///
    /// # Errors
    /// [`LaunchError`].
    fn launch(&self, kind: ChildKind, capacity: u32) -> Result<Launched, LaunchError>;

    /// Ends the child this launcher started as `peer` ([`Launched::peer`])
    /// and releases what the launcher held for it. The broker calls it for
    /// a child that missed its `ready` deadline (phase E rule E-R3: the
    /// child is killed and the spawn is a failure) or that it gives up
    /// on. A `peer` this launcher did not start, or one already gone, is
    /// left alone: the call never reaches another process.
    fn kill(&self, peer: OsPeer);

    /// Enters the restart-required state (distribution DU-3, 13 item 6): a
    /// child reported another build. Every later [`Launcher::launch`]
    /// returns [`LaunchError::RestartRequired`]; there is no way out but a
    /// restart of the browser.
    fn restart_required(&self);
}
