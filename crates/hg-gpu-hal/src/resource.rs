//! Resources and the states they are in.
//!
//! §6.6 and GR-M0's review item: **resource type states**. A texture
//! is not one thing a back end does what it likes with; it is in one
//! state at a time, and a transition between two states is a barrier
//! the back end has to emit. Getting that wrong is the commonest
//! class of GPU bug and the hardest to see, because the wrong state
//! usually works on the machine it was written on.
//!
//! So the state is a value here, the legal transitions are a function
//! of it, and a back end that forgets a barrier fails a test rather
//! than a user's machine.

/// What a resource is.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ResourceKind {
    /// A linear buffer.
    Buffer,
    /// A two-dimensional texture.
    Texture2d,
}

/// A pixel format. Integer formats only at GR-M0: GR-D3's canonical
/// path has no float, and a format nothing writes is a format nobody
/// has had to keep.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Format {
    /// Eight-bit single channel: coverage masks and A8 glyph atlases.
    R8,
    /// Eight-bit RGBA, premultiplied, the page working encoding.
    Rgba8,
    /// Sixteen-bit single channel, for intermediate coverage.
    R16,
}

impl Format {
    /// Bytes per texel.
    #[must_use]
    pub const fn bytes(self) -> u32 {
        match self {
            Self::R8 => 1,
            Self::R16 => 2,
            Self::Rgba8 => 4,
        }
    }
}

/// Which resource, in which epoch.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ResourceId {
    /// Which device.
    pub device: u32,
    /// Which epoch of that device.
    pub epoch: u64,
    /// Which resource.
    pub index: u32,
}

/// How a pass uses a resource.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Usage {
    /// Read in a shader.
    Sample,
    /// Written as a render target.
    RenderTarget,
    /// Read and written by a compute shader.
    Storage,
    /// The source of a copy.
    CopySource,
    /// The destination of a copy.
    CopyDestination,
    /// Presented to a surface.
    Present,
}

/// The state a resource is in between passes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum State {
    /// Just created, or just transitioned to nothing in particular.
    Undefined,
    /// Readable by shaders.
    ShaderRead,
    /// Writable as a render target.
    RenderTarget,
    /// Readable and writable by compute.
    Storage,
    /// The source of a copy.
    CopySource,
    /// The destination of a copy.
    CopyDestination,
    /// Handed to the presentation engine.
    Present,
}

impl State {
    /// The state a usage needs.
    #[must_use]
    pub const fn for_usage(usage: Usage) -> Self {
        match usage {
            Usage::Sample => Self::ShaderRead,
            Usage::RenderTarget => Self::RenderTarget,
            Usage::Storage => Self::Storage,
            Usage::CopySource => Self::CopySource,
            Usage::CopyDestination => Self::CopyDestination,
            Usage::Present => Self::Present,
        }
    }

    /// Whether the state may be read by more than one pass at once.
    ///
    /// Read states may; write states may not, and that is the whole
    /// of why a barrier is needed between a write and anything else.
    #[must_use]
    pub const fn is_read_only(self) -> bool {
        matches!(self, Self::ShaderRead | Self::CopySource | Self::Present)
    }

    /// Whether moving from `self` to `to` needs a barrier.
    ///
    /// Two reads in a row do not. Everything else does, including a
    /// write followed by the same write: two passes writing the same
    /// resource have to be ordered, and the barrier is the ordering.
    #[must_use]
    pub const fn needs_barrier_to(self, to: Self) -> bool {
        if matches!(self, Self::Undefined) {
            return true;
        }
        !(self.is_read_only() && to.is_read_only() && self_eq(self, to))
    }
}

/// `==` for `State` in a `const fn`, which `PartialEq` is not yet.
const fn self_eq(a: State, b: State) -> bool {
    matches!(
        (a, b),
        (State::Undefined, State::Undefined)
            | (State::ShaderRead, State::ShaderRead)
            | (State::RenderTarget, State::RenderTarget)
            | (State::Storage, State::Storage)
            | (State::CopySource, State::CopySource)
            | (State::CopyDestination, State::CopyDestination)
            | (State::Present, State::Present)
    )
}
