//! The trusted region (JS spec 6.1, JS-D16): one per process, outside the
//! cage. It holds what the engine relies on without re-checking: verified
//! bytecode and its constant kinds, and later IC programs, frame states,
//! stack maps and code-table entries. It is read-only except inside a
//! write window, which only [`TrustedRegion::insert`] opens; entries are
//! reached by a [`TrustedId`] that is bounds-, kind- and generation-checked
//! on every use, so an index read from the cage (untrusted, H-2) can name
//! another live entry of the same kind at worst, never other memory.
//!
//! **Verification condition:** every read returns a slice inside the
//! committed part of the region's reservation, of an entry that was fully
//! written before the window closed; every write lies inside the
//! committed part and happens only while the window is open.
#![expect(
    unsafe_code,
    reason = "G5: trusted (the trusted region's reads and writes)"
)]

use std::sync::Arc;

use hg_substrate::{Access, Memory, MemoryError, MemoryKind, Region};

/// What a trusted entry holds. The kind is checked on every read.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TrustedKind {
    /// A verified bytecode unit (JS spec 6.3).
    BytecodeUnit,
    /// A unit's constant kinds (design review R0-B4).
    ConstantKinds,
    /// An IC program (JS-M1).
    IcProgram,
    /// A frame state for deopt (part 2 of the design review).
    FrameState,
    /// A stack map (part 2).
    StackMap,
    /// A code-table entry (JS spec 6.1, mechanism 4).
    CodeEntry,
}

/// A reference to a trusted entry: an index and the generation it was
/// made with. It may be stored in the cage as [`TrustedId::to_bits`] and
/// read back with [`TrustedId::from_bits`]; any bits are safe to look up.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TrustedId {
    index: u32,
    generation: u32,
}

impl TrustedId {
    /// The id as 64 bits, for a raw field.
    pub const fn to_bits(self) -> u64 {
        ((self.generation as u64) << 32) | self.index as u64
    }

    /// The id from 64 bits, such as a raw field read from the cage.
    pub const fn from_bits(bits: u64) -> Self {
        Self {
            index: (bits & 0xFFFF_FFFF) as u32,
            generation: (bits >> 32) as u32,
        }
    }
}

/// Why a trusted operation failed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrustedError {
    /// The region is full.
    Full,
    /// The reservation, a commit or a protection change failed.
    Memory(MemoryError),
}

#[derive(Debug, Clone, Copy)]
struct Entry {
    kind: TrustedKind,
    generation: u32,
    offset: u64,
    len: u64,
    live: bool,
}

/// The process's trusted region.
pub struct TrustedRegion {
    mem: Arc<dyn Memory>,
    region: Option<Region>,
    base: *mut u8,
    len: u64,
    page: u64,
    committed: u64,
    cursor: u64,
    entries: Vec<Entry>,
    free: Vec<u32>,
    windows: u64,
    /// A protection change failed and the committed pages may still be
    /// writable; every insert first puts them back to read-only.
    exposed: bool,
}

impl core::fmt::Debug for TrustedRegion {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("TrustedRegion")
            .field("len", &self.len)
            .field("used", &self.cursor)
            .field("entries", &self.entries.len())
            .finish_non_exhaustive()
    }
}

fn to_usize(n: u64) -> Result<usize, TrustedError> {
    usize::try_from(n).map_err(|_| TrustedError::Full)
}

impl TrustedRegion {
    /// Reserves a trusted region of `bytes`, charged to the process part.
    ///
    /// # Errors
    /// [`TrustedError::Memory`] if the reservation fails.
    pub fn new(mem: Arc<dyn Memory>, bytes: usize) -> Result<Self, TrustedError> {
        let region = mem
            .reserve(bytes, MemoryKind::ProcessPart)
            .map_err(TrustedError::Memory)?;
        let page = u64::try_from(mem.page_size()).map_err(|_| TrustedError::Full)?;
        let len = u64::try_from(region.len()).map_err(|_| TrustedError::Full)?;
        let base = std::ptr::with_exposed_provenance_mut::<u8>(region.base());
        Ok(Self {
            mem,
            region: Some(region),
            base,
            len,
            page,
            committed: 0,
            cursor: 0,
            entries: Vec::new(),
            free: Vec::new(),
            windows: 0,
            exposed: false,
        })
    }

    /// Write windows opened so far (each is two protection flips).
    pub const fn windows(&self) -> u64 {
        self.windows
    }

    /// Copies `data` into the region as a new entry of `kind`, inside a
    /// write window, and returns its id.
    ///
    /// No error leaves the region writable: if a protection change fails,
    /// the committed pages are put back to read-only, and while that too
    /// fails every insert refuses.
    ///
    /// # Errors
    /// [`TrustedError::Full`] or [`TrustedError::Memory`].
    pub fn insert(&mut self, kind: TrustedKind, data: &[u8]) -> Result<TrustedId, TrustedError> {
        if self.exposed {
            self.seal()?;
        }
        let r = self.write(data);
        if r.is_err() {
            // New pages are committed read/write; a failed window may have
            // left them so. Errors from sealing are kept in `exposed`.
            let _ = self.seal();
        }
        let (at, end) = r?;
        self.cursor = end;
        let entry = Entry {
            kind,
            generation: 0,
            offset: at,
            len: u64::try_from(data.len()).map_err(|_| TrustedError::Full)?,
            live: true,
        };
        let index = if let Some(i) = self.free.pop() {
            let slot = self
                .entries
                .get_mut(to_usize(u64::from(i))?)
                .ok_or(TrustedError::Full)?;
            let generation = slot.generation.wrapping_add(1);
            *slot = Entry {
                generation,
                ..entry
            };
            i
        } else {
            self.entries.push(entry);
            u32::try_from(self.entries.len() - 1).map_err(|_| TrustedError::Full)?
        };
        let generation = self
            .entries
            .get(to_usize(u64::from(index))?)
            .map_or(0, |e| e.generation);
        Ok(TrustedId { index, generation })
    }

    /// Commits what an entry of `data` needs and copies it in, inside a
    /// write window; returns where it starts and where the next entry may.
    /// The cursor is not moved.
    fn write(&mut self, data: &[u8]) -> Result<(u64, u64), TrustedError> {
        let region = self.region.as_ref().ok_or(TrustedError::Full)?;
        let size = u64::try_from(data.len())
            .map_err(|_| TrustedError::Full)?
            .next_multiple_of(8)
            .max(8);
        let at = self.cursor;
        let end = at.checked_add(size).ok_or(TrustedError::Full)?;
        if end > self.len {
            return Err(TrustedError::Full);
        }
        // Commit what the entry needs; new pages start read/write.
        let need = end.next_multiple_of(self.page).min(self.len);
        if need > self.committed {
            let (o, l) = (to_usize(self.committed)?, to_usize(need - self.committed)?);
            self.mem
                .commit(region, o, l, MemoryKind::ProcessPart)
                .map_err(TrustedError::Memory)?;
            self.committed = need;
        }
        // The window: the pages the entry touches.
        let first = at - at % self.page;
        let (wo, wl) = (to_usize(first)?, to_usize(need - first)?);
        self.mem
            .protect(region, wo, wl, Access::ReadWrite)
            .map_err(TrustedError::Memory)?;
        self.windows += 1;
        let dst = self.base.wrapping_add(to_usize(at)?);
        // SAFETY: [at, at + data.len()) lies inside the committed part
        // (end <= need <= committed), which the window just made writable;
        // `data` is a separate Rust allocation, so the ranges cannot overlap.
        unsafe { std::ptr::copy_nonoverlapping(data.as_ptr(), dst, data.len()) };
        self.mem
            .protect(region, wo, wl, Access::Read)
            .map_err(TrustedError::Memory)?;
        Ok((at, end))
    }

    /// Makes every committed page read-only, and records whether that
    /// failed.
    fn seal(&mut self) -> Result<(), TrustedError> {
        let region = self.region.as_ref().ok_or(TrustedError::Full)?;
        let r = if self.committed == 0 {
            Ok(())
        } else {
            to_usize(self.committed).and_then(|l| {
                self.mem
                    .protect(region, 0, l, Access::Read)
                    .map_err(TrustedError::Memory)
            })
        };
        self.exposed = r.is_err();
        r
    }

    /// The bytes of entry `id`, if it is live, of `kind`, and of the
    /// generation `id` names.
    pub fn get(&self, id: TrustedId, kind: TrustedKind) -> Option<&[u8]> {
        let e = self.entries.get(usize::try_from(id.index).ok()?)?;
        if !e.live || e.kind != kind || e.generation != id.generation {
            return None;
        }
        let end = e.offset.checked_add(e.len)?;
        if end > self.committed {
            return None;
        }
        let p = self.base.wrapping_add(usize::try_from(e.offset).ok()?);
        let len = usize::try_from(e.len).ok()?;
        // SAFETY: [offset, offset + len) is inside the committed part
        // (checked above) and was written in full before its window
        // closed; the region changes only through `&mut self`, so the
        // bytes cannot change while this shared borrow lives.
        Some(unsafe { std::slice::from_raw_parts(p.cast_const(), len) })
    }

    /// Ends entry `id`: later lookups with it fail, and its index may be
    /// reused under a new generation. The bytes are not reused at JS-M0
    /// (the region only grows); flushing comes with JS-M1's code aging.
    pub fn remove(&mut self, id: TrustedId) -> bool {
        let Ok(i) = usize::try_from(id.index) else {
            return false;
        };
        match self.entries.get_mut(i) {
            Some(e) if e.live && e.generation == id.generation => {
                e.live = false;
                self.free.push(id.index);
                true
            }
            _ => false,
        }
    }
}

impl Drop for TrustedRegion {
    fn drop(&mut self) {
        if let Some(r) = self.region.take() {
            self.mem.release(r);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cage::test_memory::TestMemory;

    #[test]
    fn entries_are_kind_and_generation_checked() {
        let mem = Arc::new(TestMemory::default());
        let mut t = TrustedRegion::new(Arc::clone(&mem) as Arc<dyn Memory>, 1 << 20).unwrap();
        let a = t
            .insert(TrustedKind::BytecodeUnit, b"bytecode one")
            .unwrap();
        let b = t.insert(TrustedKind::ConstantKinds, &[1, 2, 3]).unwrap();
        assert_eq!(
            t.get(a, TrustedKind::BytecodeUnit),
            Some(&b"bytecode one"[..])
        );
        assert_eq!(t.get(b, TrustedKind::ConstantKinds), Some(&[1u8, 2, 3][..]));
        // The wrong kind, a stale generation, or any bits fail.
        assert_eq!(t.get(a, TrustedKind::ConstantKinds), None);
        assert!(t.remove(a));
        assert!(!t.remove(a));
        assert_eq!(t.get(a, TrustedKind::BytecodeUnit), None);
        let c = t.insert(TrustedKind::BytecodeUnit, b"two").unwrap();
        assert_eq!(c.index, a.index);
        assert_ne!(c.generation, a.generation);
        assert_eq!(t.get(a, TrustedKind::BytecodeUnit), None);
        assert_eq!(t.get(c, TrustedKind::BytecodeUnit), Some(&b"two"[..]));
        for bits in [0, u64::MAX, 0xFFFF_FFFF, 1 << 32] {
            let _ = t.get(TrustedId::from_bits(bits), TrustedKind::BytecodeUnit);
        }
        assert_eq!(TrustedId::from_bits(c.to_bits()), c);
        // Each insert opened one window: read/write, then read-only again.
        assert_eq!(t.windows(), 3);
        let flips = mem.protects();
        assert_eq!(flips.len(), 6);
        for pair in flips.chunks(2) {
            assert_eq!(pair[0].2, Access::ReadWrite);
            assert_eq!(pair[1].2, Access::Read);
            assert_eq!((pair[0].0, pair[0].1), (pair[1].0, pair[1].1));
        }
    }

    /// A failed protection change never leaves the region writable: the
    /// committed pages go back to read-only, and while that fails too,
    /// inserts refuse until it succeeds.
    #[test]
    fn a_failed_window_leaves_the_region_read_only() {
        let mem = Arc::new(TestMemory::default());
        let mut t = TrustedRegion::new(Arc::clone(&mem) as Arc<dyn Memory>, 8 * 4096).unwrap();
        let a = t.insert(TrustedKind::BytecodeUnit, b"one").unwrap();

        // The window cannot open: the new pages are sealed again.
        mem.refuse_protect(Some(Access::ReadWrite));
        let big = vec![1u8; 5000];
        assert!(matches!(
            t.insert(TrustedKind::StackMap, &big),
            Err(TrustedError::Memory(_))
        ));
        assert_eq!(mem.protects().last(), Some(&(0, 2 * 4096, Access::Read)));
        mem.refuse_protect(None);

        // The window cannot close: nothing is sealed, so inserts refuse.
        mem.refuse_protect(Some(Access::Read));
        assert!(t.insert(TrustedKind::StackMap, b"two").is_err());
        assert!(t.insert(TrustedKind::StackMap, b"three").is_err());
        let before = mem.protects().len();
        mem.refuse_protect(None);

        // Once sealing works, the next insert seals first, then goes on.
        let c = t.insert(TrustedKind::StackMap, b"four").unwrap();
        assert_eq!(mem.protects()[before], (0, 2 * 4096, Access::Read));
        assert_eq!(mem.protects().last().map(|p| p.2), Some(Access::Read));
        assert_eq!(t.get(a, TrustedKind::BytecodeUnit), Some(&b"one"[..]));
        assert_eq!(t.get(c, TrustedKind::StackMap), Some(&b"four"[..]));
    }

    #[test]
    fn a_full_region_refuses_without_writing() {
        let mem = Arc::new(TestMemory::default());
        let mut t = TrustedRegion::new(Arc::clone(&mem) as Arc<dyn Memory>, 8192).unwrap();
        let big = vec![7u8; 8000];
        let a = t.insert(TrustedKind::StackMap, &big).unwrap();
        assert_eq!(
            t.insert(TrustedKind::StackMap, &big),
            Err(TrustedError::Full)
        );
        assert_eq!(t.get(a, TrustedKind::StackMap).map(<[u8]>::len), Some(8000));
    }
}
