//! Shared-memory regions on macOS (M1 phase D6): Mach memory entries.
//! Written from Apple's kernel reference pages and the D6 probes
//! (research notes "M1 D6 shared memory", facts M1 to M10); no header,
//! kernel source or other project's code is a source.
//!
//! **Kinds.** The broker makes every region: a read-write entry. A
//! region crosses as one send right to its entry.
//! - sealing a writer makes a read-only sub-entry and drops the writer's
//!   entry; a live mapping of the writer here refuses the seal
//!   (`IpcError::Busy`, as on Linux). The kernel cannot stop a mapping
//!   made earlier elsewhere from writing on, so a sealed region is
//!   filler-trusted on macOS (review Q1);
//! - sharing a writer maps it read-write once and keeps its entry; a
//!   consumer is a read-only sub-entry of it;
//! - the receiver checks the entry with the kernel (`probe_entry`) and
//!   decides with `ipc::mach::accept::region_rule`; a read-only region
//!   sent with the `immutable` attribute is mapped as a `copy=1`
//!   snapshot at acceptance and held (review M2), so it cannot change
//!   whatever its filler does: only those are attested immutable.
//!
//! **Names.** The kernel gives one entry one name in a task and counts
//! send references on it, so two handles to one region may share a name.
//! Each handle owns exactly one reference and releases it when dropped
//! (review minor 5).

use core::any::Any;
use core::ptr::NonNull;
use core::sync::atomic::Ordering as MemOrdering;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

use hg_substrate::ipc::{
    AccessError, ConsumerRegion, Handle, HandleKind, HandleObject, IpcError, Mapping,
    MappingObject, OsPeer, ProducerRegion, ReadOnlyRegion, RegionObject, RegionRefused,
    RegionWriter,
};

use super::{Inner, MacIpc, sys};
use crate::ipc::common::codec::{HandleDesc, REGION_IMMUTABLE, WireKind};
use crate::ipc::mach::accept::MapMode;
use crate::ipc::shm::OsMapping;

/// A snapshot shared by every `Mapping` made of one region handle.
type Snapshot = Arc<OsMapping<sys::VmRange>>;

/// A region handle: one send reference on its entry's name.
pub(super) struct MacRegion {
    inner: Arc<Inner>,
    pub(super) name: u32,
    len: usize,
    kind: HandleKind,
    /// The snapshot taken at acceptance, for an `immutable` read-only
    /// region; its presence is the attestation.
    snapshot: Option<Snapshot>,
    /// Carry it with the `immutable` flag (the IDL attribute).
    want: AtomicBool,
    /// Live mappings made from this handle (a seal waits for none).
    live: Arc<AtomicUsize>,
}

impl core::fmt::Debug for MacRegion {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("MacRegion")
            .field("name", &self.name)
            .field("kind", &self.kind)
            .field("len", &self.len)
            .finish_non_exhaustive()
    }
}

impl Drop for MacRegion {
    fn drop(&mut self) {
        sys::deallocate(self.name);
    }
}

impl HandleObject for MacRegion {
    fn kind(&self) -> HandleKind {
        self.kind
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn into_any(self: Box<Self>) -> Box<dyn Any + Send> {
        self
    }
    fn region(&self) -> Option<&dyn RegionObject> {
        Some(self)
    }
}

impl RegionObject for MacRegion {
    fn len(&self) -> usize {
        self.len
    }
    fn immutable(&self) -> bool {
        self.snapshot.is_some()
    }
    fn want_immutable(&self) {
        self.want.store(true, Ordering::Relaxed);
    }
}

impl MacRegion {
    /// The name to copy into a message and its record entry, for the
    /// region kinds that may cross from this process; `None` for any
    /// other.
    pub(super) fn to_send(&self, from_broker: bool) -> Option<(u32, HandleDesc)> {
        let len = u32::try_from(self.len).ok()?;
        let desc = match self.kind {
            // A writer crosses only from the broker (review M1).
            HandleKind::RegionWriter if from_broker => {
                HandleDesc::region(WireKind::RegionWriter, len, 0)
            }
            HandleKind::ReadOnlyRegion => {
                let flags = if self.want.load(Ordering::Relaxed) || self.snapshot.is_some() {
                    REGION_IMMUTABLE
                } else {
                    0
                };
                HandleDesc::region(WireKind::ReadOnlyRegion, len, flags)
            }
            HandleKind::ConsumerRegion => HandleDesc::region(WireKind::ConsumerRegion, len, 0),
            // A producer never crosses (review B3).
            _ => return None,
        };
        Some((self.name, desc))
    }
}

fn new_handle(
    inner: &Arc<Inner>,
    name: u32,
    len: usize,
    kind: HandleKind,
    snapshot: Option<Snapshot>,
) -> Handle {
    Handle::new(Box::new(MacRegion {
        inner: Arc::clone(inner),
        name,
        len,
        kind,
        snapshot,
        want: AtomicBool::new(false),
        live: Arc::new(AtomicUsize::new(0)),
    }))
}

/// A received region whose right and entry were accepted: `name` holds
/// the reference the message brought. For [`MapMode::Snapshot`] the
/// snapshot is taken now.
///
/// # Errors
/// The name back (still holding its reference) if the snapshot could
/// not be mapped.
pub(super) fn received(
    inner: &Arc<Inner>,
    name: u32,
    kind: HandleKind,
    len: u32,
    mode: MapMode,
) -> Result<Handle, u32> {
    let len = usize::try_from(len).map_err(|_e| name)?;
    let snapshot = if mode == MapMode::Snapshot {
        Some(Arc::new(
            sys::map_entry(name, len, false, true, None).map_err(|_e| name)?,
        ))
    } else {
        None
    };
    Ok(new_handle(inner, name, len, kind, snapshot))
}

/// A `Mapping` of a held snapshot.
#[derive(Debug)]
struct SnapshotView(Snapshot);

impl MappingObject for SnapshotView {
    fn len(&self) -> usize {
        self.0.len()
    }
    fn writable(&self) -> bool {
        false
    }
    fn read(&self, off: usize, out: &mut [u8]) -> Result<(), AccessError> {
        self.0.read(off, out)
    }
    fn write(&self, off: usize, data: &[u8]) -> Result<(), AccessError> {
        self.0.write(off, data)
    }
    fn load_u32(&self, off: usize, order: MemOrdering) -> Result<u32, AccessError> {
        self.0.load_u32(off, order)
    }
    fn store_u32(&self, off: usize, v: u32, order: MemOrdering) -> Result<(), AccessError> {
        self.0.store_u32(off, v, order)
    }
    fn load_u64(&self, off: usize, order: MemOrdering) -> Result<u64, AccessError> {
        self.0.load_u64(off, order)
    }
    fn store_u64(&self, off: usize, v: u64, order: MemOrdering) -> Result<(), AccessError> {
        self.0.store_u64(off, v, order)
    }
    fn base(&self) -> NonNull<u8> {
        self.0.base()
    }
}

fn len64(len: usize) -> u64 {
    u64::try_from(len).unwrap_or(u64::MAX)
}

impl MacIpc {
    /// The region `h` names, if it is one of this transport's.
    fn own_region<'h>(&self, h: &'h Handle) -> Option<&'h MacRegion> {
        let r = h.object().downcast_ref::<MacRegion>()?;
        Arc::ptr_eq(&r.inner, &self.inner).then_some(r)
    }

    /// For `send`: a region handle's name and record entry, or `Err` for
    /// one that may not cross; `Ok(None)` for a non-region.
    pub(super) fn region_to_send(&self, h: &Handle) -> Result<Option<(u32, HandleDesc)>, IpcError> {
        if h.region().is_none() {
            return Ok(None);
        }
        let r = self.own_region(h).ok_or(IpcError::Refused)?;
        r.to_send(self.inner.is_broker())
            .map(Some)
            .ok_or(IpcError::Refused)
    }

    pub(super) fn allocate(&self, len: usize, owner: OsPeer) -> Result<RegionWriter, IpcError> {
        if !self.inner.is_broker() {
            return Err(IpcError::Unsupported);
        }
        if len == 0 || u32::try_from(len).is_err() {
            return Err(IpcError::Refused);
        }
        let charge = len64(len);
        if !self.inner.ledger().charge(owner, charge) {
            return Err(IpcError::Quota);
        }
        match sys::new_entry(charge) {
            Ok(name) => RegionWriter::from_handle(new_handle(
                &self.inner,
                name,
                len,
                HandleKind::RegionWriter,
                None,
            ))
            .map_err(|_h| IpcError::Refused),
            Err(_) => {
                self.inner.ledger().refund(owner, charge);
                Err(IpcError::Refused)
            }
        }
    }

    pub(super) fn release(&self, owner: OsPeer) {
        if self.inner.is_broker() {
            self.inner.ledger().release(owner);
        }
    }

    pub(super) fn map(&self, h: &Handle) -> Result<Mapping, IpcError> {
        let r = self.own_region(h).ok_or(IpcError::Refused)?;
        if let Some(s) = &r.snapshot {
            return Ok(Mapping::new(Box::new(SnapshotView(Arc::clone(s)))));
        }
        let writable = match r.kind {
            HandleKind::RegionWriter => true,
            HandleKind::ReadOnlyRegion | HandleKind::ConsumerRegion => false,
            // A producer holds its one mapping.
            _ => return Err(IpcError::Refused),
        };
        sys::map_entry(r.name, r.len, writable, false, Some(Arc::clone(&r.live)))
            .map(|m| Mapping::new(Box::new(m)))
            .map_err(|_e| IpcError::Refused)
    }

    pub(super) fn seal(&self, writer: RegionWriter) -> Result<ReadOnlyRegion, RegionRefused> {
        let Some(r) = self.own_region(writer.handle()) else {
            return Err(RegionRefused::new(writer.into_handle(), IpcError::Refused));
        };
        if r.live.load(Ordering::Acquire) != 0 {
            return Err(RegionRefused::new(writer.into_handle(), IpcError::Busy));
        }
        let Ok(name) = sys::sub_entry(r.name, len64(r.len), false) else {
            return Err(RegionRefused::new(writer.into_handle(), IpcError::Refused));
        };
        let h = new_handle(&self.inner, name, r.len, HandleKind::ReadOnlyRegion, None);
        drop(writer);
        ReadOnlyRegion::from_handle(h).map_err(|h| RegionRefused::new(h, IpcError::Refused))
    }

    pub(super) fn share(&self, writer: RegionWriter) -> Result<ProducerRegion, RegionRefused> {
        let Some(r) = self.own_region(writer.handle()) else {
            return Err(RegionRefused::new(writer.into_handle(), IpcError::Refused));
        };
        let Ok(m) = sys::map_entry(r.name, r.len, true, false, None) else {
            return Err(RegionRefused::new(writer.into_handle(), IpcError::Refused));
        };
        // The producer's own reference; the writer's goes with it.
        let (name, len) = (r.name, r.len);
        if sys::add_send_ref(name).is_err() {
            return Err(RegionRefused::new(writer.into_handle(), IpcError::Refused));
        }
        drop(writer);
        let h = new_handle(&self.inner, name, len, HandleKind::ProducerRegion, None);
        ProducerRegion::new(h, Mapping::new(Box::new(m)))
            .map_err(|(h, _m)| RegionRefused::new(h, IpcError::Refused))
    }

    pub(super) fn consumer(&self, producer: &ProducerRegion) -> Result<ConsumerRegion, IpcError> {
        let r = self
            .own_region(producer.handle())
            .ok_or(IpcError::Refused)?;
        let name = sys::sub_entry(r.name, len64(r.len), false).map_err(|_e| IpcError::Refused)?;
        ConsumerRegion::from_handle(new_handle(
            &self.inner,
            name,
            r.len,
            HandleKind::ConsumerRegion,
            None,
        ))
        .map_err(|_h| IpcError::Refused)
    }
}
