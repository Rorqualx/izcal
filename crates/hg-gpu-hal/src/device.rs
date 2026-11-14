//! Capabilities, sticky validation, and device loss with an epoch.

use core::fmt;

/// What a device can do. **No field names a vendor.**
///
/// §6.6: "devices advertise capabilities, never vendors". A driver
/// workaround is a capability reported absent, which the rest of the
/// engine already knows how to handle; a vendor check is a claim
/// about every device that vendor will ever make.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[non_exhaustive]
pub struct Capabilities {
    /// Compute shaders with integer atomics, which the canonical GPU
    /// raster needs (GR-D3: "commutative integer accumulation only").
    pub integer_atomics: bool,
    /// Timeline semaphores, rather than per-submission fences.
    pub timeline_semaphores: bool,
    /// Textures may be sampled while another queue writes them.
    pub concurrent_sampling: bool,
    /// The largest 2D texture dimension.
    pub max_texture_2d: u32,
    /// The largest number of bytes one buffer may hold.
    pub max_buffer_bytes: u64,
    /// The largest workgroup size in each dimension.
    pub max_workgroup: [u32; 3],
}

impl Capabilities {
    /// A device that advertises nothing.
    ///
    /// The starting point for the builders below, and the honest
    /// answer for a back end that has not probed yet. Everything is
    /// absent, so a caller that forgets to probe falls back rather
    /// than assuming a capability it never asked about.
    pub const NONE: Self = Self {
        integer_atomics: false,
        timeline_semaphores: false,
        concurrent_sampling: false,
        max_texture_2d: 0,
        max_buffer_bytes: 0,
        max_workgroup: [0, 0, 0],
    };

    /// Reports integer atomics.
    #[must_use]
    pub const fn with_integer_atomics(mut self, yes: bool) -> Self {
        self.integer_atomics = yes;
        self
    }

    /// Reports timeline semaphores.
    #[must_use]
    pub const fn with_timeline_semaphores(mut self, yes: bool) -> Self {
        self.timeline_semaphores = yes;
        self
    }

    /// Reports sampling while another queue writes.
    #[must_use]
    pub const fn with_concurrent_sampling(mut self, yes: bool) -> Self {
        self.concurrent_sampling = yes;
        self
    }

    /// Reports the largest 2D texture dimension.
    #[must_use]
    pub const fn with_max_texture_2d(mut self, n: u32) -> Self {
        self.max_texture_2d = n;
        self
    }

    /// Reports the largest buffer.
    #[must_use]
    pub const fn with_max_buffer_bytes(mut self, n: u64) -> Self {
        self.max_buffer_bytes = n;
        self
    }

    /// Reports the largest workgroup in each dimension.
    #[must_use]
    pub const fn with_max_workgroup(mut self, n: [u32; 3]) -> Self {
        self.max_workgroup = n;
        self
    }

    /// Whether this device can run the canonical integer raster.
    ///
    /// The one derived question the engine asks, written here so it
    /// is asked the same way everywhere. GR-D3's canonical GPU passes
    /// use commutative integer accumulation only, which needs integer
    /// atomics; without them the device raster falls back to the CPU
    /// and the bits are the same either way.
    #[must_use]
    pub const fn can_raster_canonically(&self) -> bool {
        self.integer_atomics && self.max_texture_2d >= 4_096
    }
}

/// Why a command was refused.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum ValidationError {
    /// A resource was used in a state it is not in.
    WrongState,
    /// A size or count above what the device advertises.
    AboveLimit,
    /// A resource that does not belong to this device.
    ForeignResource,
    /// A pass that reads what no pass writes.
    UnwrittenRead,
    /// A graph with a cycle.
    Cycle,
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Self::WrongState => "a resource was used in a state it is not in",
            Self::AboveLimit => "above the device's advertised limit",
            Self::ForeignResource => "a resource from another device",
            Self::UnwrittenRead => "a pass reads what no pass writes",
            Self::Cycle => "the render graph has a cycle",
        };
        f.write_str(s)
    }
}

impl core::error::Error for ValidationError {}

/// Validation state, which is **sticky** (§6.6).
///
/// The first error is kept and every later one is counted. Sticky
/// rather than returned-and-forgotten because GPU work is submitted
/// in batches and a caller that checked only the last command would
/// miss the one that actually went wrong — and because an error that
/// clears itself is one that shows up in a bug report as "sometimes
/// nothing renders".
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Validation {
    first: Option<ValidationError>,
    count: u64,
}

impl Validation {
    /// No errors.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            first: None,
            count: 0,
        }
    }

    /// Records an error.
    pub const fn record(&mut self, e: ValidationError) {
        if self.first.is_none() {
            self.first = Some(e);
        }
        self.count = self.count.saturating_add(1);
    }

    /// The first error, which is the one that explains the rest.
    #[must_use]
    pub const fn first(&self) -> Option<ValidationError> {
        self.first
    }

    /// How many errors there have been.
    #[must_use]
    pub const fn count(&self) -> u64 {
        self.count
    }

    /// Whether anything has gone wrong.
    #[must_use]
    pub const fn is_clean(&self) -> bool {
        self.first.is_none()
    }

    /// Clears the state, which only rebuilding the device may do.
    ///
    /// Takes the [`Epoch`] it is rebuilding into, so that clearing is
    /// something that happens as part of a device's life rather than
    /// something a caller can do to make a symptom go away.
    pub const fn reset_for(&mut self, _epoch: Epoch) {
        self.first = None;
        self.count = 0;
    }
}

/// Which generation of a device something belongs to.
///
/// Monotonic. A resource from an older epoch is a
/// [`ValidationError::ForeignResource`], which is how a stale handle
/// is caught rather than used.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct Epoch(pub u64);

/// Device loss, resolved exactly once (§6.6).
///
/// A lost device is discovered by every caller at once and each of
/// them wants to rebuild. Without this they all rebuild and the
/// rebuilds race; with it, [`Self::resolve`] says yes to one caller
/// per epoch and no to the rest, who then wait for the new device.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DeviceLoss {
    epoch: Epoch,
    lost: bool,
    resolved: bool,
}

impl DeviceLoss {
    /// A device that is alive, at epoch 0.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            epoch: Epoch(0),
            lost: false,
            resolved: false,
        }
    }

    /// The current epoch.
    #[must_use]
    pub const fn epoch(&self) -> Epoch {
        self.epoch
    }

    /// Whether the device is lost.
    #[must_use]
    pub const fn is_lost(&self) -> bool {
        self.lost
    }

    /// Records that the device was lost. Repeated reports of the same
    /// loss do nothing, which is what lets every caller report it.
    pub const fn lose(&mut self) {
        if !self.lost {
            self.lost = true;
            self.resolved = false;
        }
    }

    /// Claims the rebuild.
    ///
    /// `Some(epoch)` for the one caller that gets it, `None` for
    /// every other — including a caller that asks before the device
    /// is lost at all. The epoch returned is the **new** one, so the
    /// caller builds into it and every resource from before it is
    /// foreign.
    pub const fn resolve(&mut self) -> Option<Epoch> {
        if !self.lost || self.resolved {
            return None;
        }
        self.resolved = true;
        self.epoch = Epoch(self.epoch.0.saturating_add(1));
        self.lost = false;
        Some(self.epoch)
    }
}

impl Default for DeviceLoss {
    fn default() -> Self {
        Self::new()
    }
}
