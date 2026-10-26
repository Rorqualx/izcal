//! Shared-memory regions (security scope IPC-R1, IPC-R7; M1 phase D6).
//!
//! The broker allocates every region ([`super::Ipc::allocate_region`]) and
//! hands it out as a [`RegionWriter`]: fresh, one holder. The holder fills
//! it and either seals it ([`super::Ipc::seal_region`]), which gives a
//! [`ReadOnlyRegion`] nobody can write, or keeps writing
//! ([`super::Ipc::share_region`]), which gives a [`ProducerRegion`] that
//! never leaves the process and hands out [`ConsumerRegion`]s. Nothing goes
//! back to writable.
//!
//! A region crosses a channel with a declared length, which the receiver
//! maps exactly. Bytes in shared memory are read only by copying them out
//! ([`Mapping::read_into`]) or as single ordered words
//! ([`Mapping::load_u32_acquire`] and the like), never as a `&[u8]`, so
//! another process writing at the same time cannot break Rust's rules for
//! this process (IPC-R7: copy, then validate).
//!
//! What "read-only" guarantees depends on the OS (the D6 facts in
//! `research_notes/M1 D6 shared memory/shm_facts.md`):
//! [`RegionObject::immutable`] says whether the receiving backend attests
//! that the contents can no longer change (always on Linux, by the seal; on
//! macOS for a region that crossed as `immutable`, by a snapshot; on
//! Windows when the broker filled it).

use core::ptr::NonNull;
use core::sync::atomic::Ordering;

use super::{Handle, HandleKind, IpcError};

/// Why an access to a mapping failed. Every case is the caller's to
/// handle; none is a panic.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AccessError {
    /// The range runs past the mapping's end, or its end overflows.
    OutOfBounds,
    /// A word access at an offset not a multiple of the word's size.
    Misaligned,
    /// A write to a read-only mapping.
    ReadOnly,
}

impl core::fmt::Display for AccessError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(match self {
            Self::OutOfBounds => "the access runs past the mapping",
            Self::Misaligned => "a word access at an unaligned offset",
            Self::ReadOnly => "a write to a read-only mapping",
        })
    }
}

impl std::error::Error for AccessError {}

/// Checks that `n` bytes at `off` lie inside a mapping of `len` bytes.
/// Every backend calls this before it touches memory.
///
/// # Errors
/// [`AccessError::OutOfBounds`].
pub const fn check_range(off: usize, n: usize, len: usize) -> Result<(), AccessError> {
    match off.checked_add(n) {
        Some(end) if end <= len => Ok(()),
        _ => Err(AccessError::OutOfBounds),
    }
}

/// Checks a word access of `size` bytes (4 or 8) at `off` in a mapping of
/// `len` bytes: in bounds, and `off` a multiple of `size`. Mappings start
/// on a page boundary, so an aligned offset is an aligned address.
///
/// # Errors
/// [`AccessError::OutOfBounds`], [`AccessError::Misaligned`].
pub const fn check_word(off: usize, size: usize, len: usize) -> Result<(), AccessError> {
    if let Err(e) = check_range(off, size, len) {
        return Err(e);
    }
    match off.checked_rem(size) {
        Some(0) => Ok(()),
        _ => Err(AccessError::Misaligned),
    }
}

/// A backend's mapping of a region. Every method checks its own range with
/// [`check_range`] or [`check_word`] before touching memory, so no input
/// reaches outside the mapping.
pub trait MappingObject: Send + Sync + core::fmt::Debug {
    /// Its length in bytes: the region's declared length.
    fn len(&self) -> usize;
    /// Whether it has no bytes (never, for a region's mapping).
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
    /// Whether it may be written.
    fn writable(&self) -> bool;
    /// Copies `out.len()` bytes from `off`, byte by byte with no ordering
    /// (another process may be writing them).
    ///
    /// # Errors
    /// [`AccessError::OutOfBounds`].
    fn read(&self, off: usize, out: &mut [u8]) -> Result<(), AccessError>;
    /// Copies `data` to `off`.
    ///
    /// # Errors
    /// [`AccessError::OutOfBounds`], [`AccessError::ReadOnly`].
    fn write(&self, off: usize, data: &[u8]) -> Result<(), AccessError>;
    /// An atomic 32-bit load at `off`.
    ///
    /// # Errors
    /// [`AccessError::OutOfBounds`], [`AccessError::Misaligned`].
    fn load_u32(&self, off: usize, order: Ordering) -> Result<u32, AccessError>;
    /// An atomic 32-bit store at `off`.
    ///
    /// # Errors
    /// As [`MappingObject::load_u32`], and [`AccessError::ReadOnly`].
    fn store_u32(&self, off: usize, v: u32, order: Ordering) -> Result<(), AccessError>;
    /// An atomic 64-bit load at `off`.
    ///
    /// # Errors
    /// [`AccessError::OutOfBounds`], [`AccessError::Misaligned`].
    fn load_u64(&self, off: usize, order: Ordering) -> Result<u64, AccessError>;
    /// An atomic 64-bit store at `off`.
    ///
    /// # Errors
    /// As [`MappingObject::load_u64`], and [`AccessError::ReadOnly`].
    fn store_u64(&self, off: usize, v: u64, order: Ordering) -> Result<(), AccessError>;
    /// The first byte, for [`Mapping::raw_base`].
    fn base(&self) -> NonNull<u8>;
}

/// A mapping of a region into this process. Dropping it unmaps the region.
///
/// Mappings are made when needed and not kept idle: each one is a mapping
/// counted against the footprint spec's budget of 64 in an idle process.
#[derive(Debug)]
pub struct Mapping(Box<dyn MappingObject>);

impl Mapping {
    /// Wraps a backend's mapping. For backends only.
    pub fn new(object: Box<dyn MappingObject>) -> Self {
        Self(object)
    }

    /// Its length in bytes.
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Whether it is empty (never, for a region's mapping).
    pub fn is_empty(&self) -> bool {
        self.0.len() == 0
    }

    /// Whether it may be written.
    pub fn writable(&self) -> bool {
        self.0.writable()
    }

    /// Copies `out.len()` bytes at `off` into `out` (IPC-R7: the copy the
    /// receiver then validates). Another process may be writing them, so
    /// the copy may be torn; a seqlock built from the word accessors
    /// detects that.
    ///
    /// # Errors
    /// [`AccessError::OutOfBounds`].
    pub fn read_into(&self, off: usize, out: &mut [u8]) -> Result<(), AccessError> {
        self.0.read(off, out)
    }

    /// Copies `data` to `off`.
    ///
    /// # Errors
    /// [`AccessError::OutOfBounds`], [`AccessError::ReadOnly`].
    pub fn write_from(&self, off: usize, data: &[u8]) -> Result<(), AccessError> {
        self.0.write(off, data)
    }

    /// A 32-bit load with acquire ordering at a 4-aligned `off`.
    ///
    /// # Errors
    /// [`AccessError::OutOfBounds`], [`AccessError::Misaligned`].
    pub fn load_u32_acquire(&self, off: usize) -> Result<u32, AccessError> {
        self.0.load_u32(off, Ordering::Acquire)
    }

    /// A 32-bit load with no ordering at a 4-aligned `off`.
    ///
    /// # Errors
    /// As [`Mapping::load_u32_acquire`].
    pub fn load_u32_relaxed(&self, off: usize) -> Result<u32, AccessError> {
        self.0.load_u32(off, Ordering::Relaxed)
    }

    /// A 32-bit store with release ordering at a 4-aligned `off`.
    ///
    /// # Errors
    /// As [`Mapping::load_u32_acquire`], and [`AccessError::ReadOnly`].
    pub fn store_u32_release(&self, off: usize, v: u32) -> Result<(), AccessError> {
        self.0.store_u32(off, v, Ordering::Release)
    }

    /// A 64-bit load with acquire ordering at an 8-aligned `off`.
    ///
    /// # Errors
    /// [`AccessError::OutOfBounds`], [`AccessError::Misaligned`].
    pub fn load_u64_acquire(&self, off: usize) -> Result<u64, AccessError> {
        self.0.load_u64(off, Ordering::Acquire)
    }

    /// A 64-bit load with no ordering at an 8-aligned `off`.
    ///
    /// # Errors
    /// As [`Mapping::load_u64_acquire`].
    pub fn load_u64_relaxed(&self, off: usize) -> Result<u64, AccessError> {
        self.0.load_u64(off, Ordering::Relaxed)
    }

    /// A 64-bit store with release ordering at an 8-aligned `off`.
    ///
    /// # Errors
    /// As [`Mapping::load_u64_acquire`], and [`AccessError::ReadOnly`].
    pub fn store_u64_release(&self, off: usize, v: u64) -> Result<(), AccessError> {
        self.0.store_u64(off, v, Ordering::Release)
    }

    /// The mapping's first byte, for an opaque buffer handed to the GPU
    /// upload path only (IPC-R7's exemption for sampled buffers). Reading
    /// through it is the caller's `unsafe` and outside deterministic
    /// simulation: the simulator models only the checked accessors.
    pub fn raw_base(&self) -> NonNull<u8> {
        self.0.base()
    }
}

/// What a region handle knows beyond its kind.
pub trait RegionObject: Send + Sync {
    /// The declared length in bytes: what the receiver maps.
    fn len(&self) -> usize;
    /// Whether it has no bytes (never: regions are not empty).
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
    /// Whether the receiving backend attests that the contents can no
    /// longer change (see the module documentation).
    fn immutable(&self) -> bool;
    /// Asks the transport to carry this read-only region as `immutable`
    /// (the IDL attribute): on Windows the broker then copies a region a
    /// sandboxed process sealed into one it fills itself; on macOS the
    /// receiver maps a snapshot.
    fn want_immutable(&self);
}

/// Gives a region kind's typed wrapper `from_handle`, `into_handle`,
/// `handle`, `len` and `immutable`.
macro_rules! region_handle {
    ($t:ident, $kind:ident, $doc:literal) => {
        #[doc = $doc]
        #[derive(Debug)]
        pub struct $t(Handle);

        impl $t {
            /// Wraps a handle of this kind.
            ///
            /// # Errors
            /// Gives the handle back if it is another kind, or not a region.
            pub fn from_handle(h: Handle) -> Result<Self, Handle> {
                if h.kind() == HandleKind::$kind && h.region().is_some() {
                    Ok(Self(h))
                } else {
                    Err(h)
                }
            }

            /// The untyped handle, to send it.
            pub fn into_handle(self) -> Handle {
                self.0
            }

            /// The untyped handle. For backends only.
            pub const fn handle(&self) -> &Handle {
                &self.0
            }

            /// The declared length in bytes.
            pub fn len(&self) -> usize {
                self.0.region().map_or(0, RegionObject::len)
            }

            /// Whether it has no bytes (never: regions are not empty).
            pub fn is_empty(&self) -> bool {
                self.len() == 0
            }

            /// Whether this process's backend attests the contents can no
            /// longer change.
            pub fn immutable(&self) -> bool {
                self.0.region().is_some_and(RegionObject::immutable)
            }
        }
    };
}

region_handle!(
    RegionWriter,
    RegionWriter,
    "A fresh region, writable by its one holder; it crosses only from the broker."
);
region_handle!(
    ReadOnlyRegion,
    ReadOnlyRegion,
    "A sealed region: nobody can write it (whether its contents are attested unchanging is [`ReadOnlyRegion::immutable`])."
);
region_handle!(
    ConsumerRegion,
    ConsumerRegion,
    "A read-only view of a region whose producer keeps writing it."
);

impl ReadOnlyRegion {
    /// Asks the transport to carry it as `immutable` (see
    /// [`RegionObject::want_immutable`]); the generated code calls this for
    /// fields with the IDL attribute.
    pub fn want_immutable(&self) {
        if let Some(r) = self.0.region() {
            r.want_immutable();
        }
    }
}

/// A region this process keeps writing: made from a [`RegionWriter`] by
/// [`super::Ipc::share_region`]. It holds its one writable mapping for its
/// life (counted against the mapping budget) and never crosses a process
/// boundary: on Linux no new writable mapping can be made once it exists
/// (`F_SEAL_FUTURE_WRITE`). Two producers, one each way, make a ring.
#[derive(Debug)]
pub struct ProducerRegion {
    handle: Handle,
    mapping: Mapping,
}

impl ProducerRegion {
    /// Wraps a backend's producer object and its mapping. For backends
    /// only.
    ///
    /// # Errors
    /// Gives both back if the handle is not a producer.
    pub fn new(handle: Handle, mapping: Mapping) -> Result<Self, (Handle, Mapping)> {
        if handle.kind() == HandleKind::ProducerRegion && handle.region().is_some() {
            Ok(Self { handle, mapping })
        } else {
            Err((handle, mapping))
        }
    }

    /// The writable mapping.
    pub const fn mapping(&self) -> &Mapping {
        &self.mapping
    }

    /// The backend object. For backends only.
    pub const fn handle(&self) -> &Handle {
        &self.handle
    }

    /// The declared length in bytes.
    pub fn len(&self) -> usize {
        self.mapping.len()
    }

    /// Whether it has no bytes (never).
    pub fn is_empty(&self) -> bool {
        self.mapping.is_empty()
    }
}

/// A seal or share the backend refused, with the writer given back.
#[derive(Debug)]
pub struct RegionRefused {
    /// The writer, unchanged.
    pub writer: RegionWriter,
    /// Why: [`IpcError::Busy`] while a mapping of it is live, or a
    /// backend's failure.
    pub error: IpcError,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ranges_and_words_are_checked() {
        assert_eq!(check_range(0, 4096, 4096), Ok(()));
        assert_eq!(check_range(1, 4096, 4096), Err(AccessError::OutOfBounds));
        assert_eq!(
            check_range(usize::MAX, 2, 4096),
            Err(AccessError::OutOfBounds)
        );
        assert_eq!(check_range(4096, 0, 4096), Ok(()));
        assert_eq!(check_word(8, 8, 16), Ok(()));
        assert_eq!(check_word(4, 8, 16), Err(AccessError::Misaligned));
        assert_eq!(check_word(12, 8, 16), Err(AccessError::OutOfBounds));
        assert_eq!(check_word(12, 4, 16), Ok(()));
        assert_eq!(check_word(0, 0, 16), Err(AccessError::Misaligned));
    }
}

#[cfg(kani)]
mod proofs {
    use super::*;

    /// An accepted range lies inside the mapping, for every input
    /// (D6 minor 3: `read_into`/`write_from` overflow).
    #[kani::proof]
    fn accepted_ranges_are_inside() {
        let (off, n, len): (usize, usize, usize) = (kani::any(), kani::any(), kani::any());
        match check_range(off, n, len) {
            Ok(()) => {
                assert!(off <= len);
                assert!(n <= len - off);
            }
            Err(_) => assert!(off.checked_add(n).is_none_or(|e| e > len)),
        }
        kani::cover!(check_range(off, n, len).is_ok() && n > 0);
        kani::cover!(off.checked_add(n).is_none());
    }

    /// An accepted word is inside the mapping and aligned to its size
    /// (D6 M3).
    #[kani::proof]
    fn accepted_words_are_inside_and_aligned() {
        let (off, len): (usize, usize) = (kani::any(), kani::any());
        let size: usize = if kani::any() { 4 } else { 8 };
        if check_word(off, size, len).is_ok() {
            assert!(off % size == 0);
            assert!(off <= len && size <= len - off);
        }
        kani::cover!(check_word(off, size, len) == Err(AccessError::Misaligned));
        kani::cover!(check_word(off, size, len).is_ok() && size == 8);
    }
}
