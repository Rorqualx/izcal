//! The sandbox report between the launch contract and the wire (phase E,
//! E-R5, review M3): a child turns what its substrate found into
//! `process.hgidl`'s `SandboxReport`, and the broker turns a received
//! report's tier back to compare it with what its launcher applied.
//! Hand-written: the two enums are the same lists, kept in step by the
//! exhaustive matches below.

use hg_substrate::launch::{SandboxFeature as Feature, SandboxReached, SandboxTier as Tier};

use crate::process::{SandboxFeature, SandboxReport, SandboxTier};

/// The wire tier of `t`.
#[must_use]
pub const fn wire_tier(t: Tier) -> SandboxTier {
    match t {
        Tier::LinuxNoUserns => SandboxTier::LinuxNoUserns,
        Tier::LinuxRestrictedUserns => SandboxTier::LinuxRestrictedUserns,
        Tier::LinuxFull => SandboxTier::LinuxFull,
        Tier::MacosSbpl => SandboxTier::MacosSbpl,
        Tier::WindowsAppContainer => SandboxTier::WindowsAppContainer,
        Tier::Simulated => SandboxTier::Simulated,
    }
}

/// The launch contract's tier of wire tier `t`.
#[must_use]
pub const fn launch_tier(t: SandboxTier) -> Tier {
    match t {
        SandboxTier::LinuxNoUserns => Tier::LinuxNoUserns,
        SandboxTier::LinuxRestrictedUserns => Tier::LinuxRestrictedUserns,
        SandboxTier::LinuxFull => Tier::LinuxFull,
        SandboxTier::MacosSbpl => Tier::MacosSbpl,
        SandboxTier::WindowsAppContainer => Tier::WindowsAppContainer,
        SandboxTier::Simulated => Tier::Simulated,
    }
}

const fn wire_feature(f: Feature) -> SandboxFeature {
    match f {
        Feature::EmptyRoot => SandboxFeature::EmptyRoot,
        Feature::IdMap => SandboxFeature::IdMap,
        Feature::MountNamespace => SandboxFeature::MountNamespace,
        Feature::LandlockUdp => SandboxFeature::LandlockUdp,
        Feature::AlternateDesktop => SandboxFeature::AlternateDesktop,
        Feature::Win32kLockdown => SandboxFeature::Win32kLockdown,
        Feature::CodeIntegrityGuard => SandboxFeature::CodeIntegrityGuard,
    }
}

/// The report a child sends in `ready`. Features past the wire bound
/// (eight) are dropped; there are seven.
#[must_use]
pub fn wire_report(r: &SandboxReached) -> SandboxReport {
    let missing: Vec<SandboxFeature> = r.missing.iter().map(|&f| wire_feature(f)).take(8).collect();
    SandboxReport {
        tier: wire_tier(r.tier),
        missing: hg_ipc::BoundedVec::try_new(missing).unwrap_or_default(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tiers_round_trip() {
        for t in [
            Tier::LinuxNoUserns,
            Tier::LinuxRestrictedUserns,
            Tier::LinuxFull,
            Tier::MacosSbpl,
            Tier::WindowsAppContainer,
            Tier::Simulated,
        ] {
            assert_eq!(launch_tier(wire_tier(t)), t);
        }
    }
}
