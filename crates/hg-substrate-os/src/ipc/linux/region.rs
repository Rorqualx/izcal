//! Shared-memory regions on Linux (M1 phase D6): sealed memfds. Written
//! from memfd_create(2), fcntl(2) "File sealing", mmap(2) and the D6
//! probes (research notes "M1 D6 shared memory", facts L1 to L13); no
//! other project's code is a source.
//!
//! **Kinds and seals.** The broker makes every region: a memfd made with
//! `MFD_NOEXEC_SEAL`, sized, and sealed against growing and shrinking, so
//! a writer's seals are exactly `GROW | SHRINK | EXEC` (review M5).
//! - sealing a writer adds `WRITE | SEAL`; the kernel refuses `WRITE`
//!   with `EBUSY` while any shared mapping made through a writable
//!   descriptor exists, even a read-only one (fact L1b), so a sealed
//!   region has no writer anywhere and its contents are unchanging: on
//!   Linux every read-only region is attested immutable;
//! - sharing a writer maps it read-write once, then adds
//!   `FUTURE_WRITE | SEAL`: that one mapping stays writable (fact L2) and
//!   no new writable mapping, `mprotect` or `write` can be made by
//!   anyone (review B3). The producer never crosses; its consumers do.
//!
//! **What a receiver checks** is `ipc::unix::accept::accept_region`:
//! seals by kind, the size equal to the declared length, and a writer
//! only from the broker.
//!
//! **The startup self-test.** Before the first allocation the broker
//! checks, on this kernel, every behaviour above (review Q4): the exec
//! seal, `EBUSY`, and that a consumer cannot write. If any fails, regions
//! are unsupported in this browser.

use core::any::Any;
use std::os::fd::{AsFd, OwnedFd};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use hg_substrate::ipc::{
    ConsumerRegion, Handle, HandleKind, HandleObject, IpcError, Mapping, OsPeer, ProducerRegion,
    ReadOnlyRegion, RegionObject, RegionRefused, RegionWriter,
};

use super::{Inner, LinuxIpc, sys};
use crate::ipc::common::codec::{HandleDesc, REGION_IMMUTABLE, WireKind};
use crate::ipc::unix::accept::{
    CONSUMER_SEALS, READ_ONLY_SEALS, SEAL_FUTURE_WRITE, SEAL_GROW, SEAL_SEAL, SEAL_SHRINK,
    SEAL_WRITE, WRITER_SEALS,
};

/// A region handle: the memfd, shared by every handle of the same region
/// in this process, and what this process knows of it.
pub(super) struct LinuxRegion {
    inner: Arc<Inner>,
    pub(super) fd: Arc<OwnedFd>,
    len: usize,
    kind: HandleKind,
    /// Attested unchanging: a read-only region (sealed against writes).
    immutable: bool,
    /// Carry it with the `immutable` flag (the IDL attribute).
    want: AtomicBool,
}

impl core::fmt::Debug for LinuxRegion {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("LinuxRegion")
            .field("kind", &self.kind)
            .field("len", &self.len)
            .finish_non_exhaustive()
    }
}

impl HandleObject for LinuxRegion {
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

impl RegionObject for LinuxRegion {
    fn len(&self) -> usize {
        self.len
    }
    fn immutable(&self) -> bool {
        self.immutable
    }
    fn want_immutable(&self) {
        self.want.store(true, Ordering::Relaxed);
    }
}

impl LinuxRegion {
    /// The descriptor to send and its record entry, for the region kinds
    /// that may cross from this process; `None` for any other.
    pub(super) fn to_send(&self, from_broker: bool) -> Option<(Arc<OwnedFd>, HandleDesc)> {
        let len = u32::try_from(self.len).ok()?;
        let desc = match self.kind {
            // A writer crosses only from the broker (review M1).
            HandleKind::RegionWriter if from_broker => {
                HandleDesc::region(WireKind::RegionWriter, len, 0)
            }
            HandleKind::ReadOnlyRegion => {
                let flags = if self.want.load(Ordering::Relaxed) {
                    REGION_IMMUTABLE
                } else {
                    0
                };
                HandleDesc::region(WireKind::ReadOnlyRegion, len, flags)
            }
            HandleKind::ConsumerRegion => HandleDesc::region(WireKind::ConsumerRegion, len, 0),
            // A producer never crosses (review B3), nor a writer from a
            // sandboxed process.
            _ => return None,
        };
        Some((Arc::clone(&self.fd), desc))
    }
}

/// A received region, accepted as `kind` with the declared `len`.
pub(super) fn received(inner: &Arc<Inner>, fd: OwnedFd, kind: HandleKind, len: u32) -> Handle {
    new_handle(inner, Arc::new(fd), usize::try_from(len).unwrap_or(0), kind)
}

fn new_handle(inner: &Arc<Inner>, fd: Arc<OwnedFd>, len: usize, kind: HandleKind) -> Handle {
    Handle::new(Box::new(LinuxRegion {
        inner: Arc::clone(inner),
        fd,
        len,
        kind,
        immutable: kind == HandleKind::ReadOnlyRegion,
        want: AtomicBool::new(false),
    }))
}

/// The kernel behaviours the region rules rely on, checked once on this
/// kernel (review Q4). Each step is one of the module's claims.
fn self_test() -> bool {
    let grow_shrink = SEAL_GROW | SEAL_SHRINK;
    let ok = |fd: &OwnedFd, seals: i32| sys::get_seals(fd.as_fd()).is_ok_and(|s| s == seals);
    // A writer: the exec seal comes with MFD_NOEXEC_SEAL.
    let Ok(w) = sys::memfd_region(4096, grow_shrink) else {
        return false;
    };
    if !ok(&w, WRITER_SEALS) {
        return false;
    }
    // EBUSY: a read-only shared mapping through the writable descriptor
    // blocks the write seal (fact L1b).
    if sys::seal_write_with_reader(w.as_fd(), 4096) != Some(libc::EBUSY) {
        return false;
    }
    // Sealed: no write by descriptor or by a new mapping.
    if sys::add_seals(w.as_fd(), SEAL_WRITE | SEAL_SEAL).is_err()
        || !ok(&w, READ_ONLY_SEALS)
        || sys::try_pwrite(w.as_fd()).is_ok()
        || sys::map_region(w.as_fd(), 4096, true).is_ok()
    {
        return false;
    }
    // A producer: its own mapping stays writable; nobody else can write.
    let Ok(p) = sys::memfd_region(4096, grow_shrink) else {
        return false;
    };
    let Ok(pm) = sys::map_region(p.as_fd(), 4096, true) else {
        return false;
    };
    let sealed = sys::add_seals(p.as_fd(), SEAL_FUTURE_WRITE | SEAL_SEAL).is_ok();
    let still_writes = pm.write_from(0, &[7]).is_ok();
    let no_new_writer = sys::map_region(p.as_fd(), 4096, true).is_err();
    let no_write = sys::try_pwrite(p.as_fd()).is_err();
    let no_widen = matches!(sys::can_widen_reader(p.as_fd(), 4096), Ok(false));
    let mut seen = [0u8; 1];
    let reads = sys::map_region(p.as_fd(), 4096, false)
        .is_ok_and(|m| m.read_into(0, &mut seen).is_ok() && seen == [7]);
    sealed
        && ok(&p, CONSUMER_SEALS)
        && still_writes
        && no_new_writer
        && no_write
        && no_widen
        && reads
}

impl LinuxIpc {
    /// The region `h` names, if it is one of this transport's.
    fn own_region<'h>(&self, h: &'h Handle) -> Option<&'h LinuxRegion> {
        let r = h.object().downcast_ref::<LinuxRegion>()?;
        Arc::ptr_eq(&r.inner, &self.inner).then_some(r)
    }

    /// For `send`: a region handle's descriptor and record entry, or
    /// `Err` for one that may not cross; `Ok(None)` for a non-region.
    pub(super) fn region_to_send(
        &self,
        h: &Handle,
    ) -> Result<Option<(Arc<OwnedFd>, HandleDesc)>, IpcError> {
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
        if !*self.inner.regions_ok.get_or_init(self_test) {
            return Err(IpcError::Unsupported);
        }
        let charge = u64::try_from(len).map_err(|_e| IpcError::Refused)?;
        if !self.inner.ledger().charge(owner, charge) {
            return Err(IpcError::Quota);
        }
        match sys::memfd_region(len, SEAL_GROW | SEAL_SHRINK) {
            Ok(fd) => RegionWriter::from_handle(new_handle(
                &self.inner,
                Arc::new(fd),
                len,
                HandleKind::RegionWriter,
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
        let writable = match r.kind {
            HandleKind::RegionWriter => true,
            HandleKind::ReadOnlyRegion | HandleKind::ConsumerRegion => false,
            // A producer holds its one mapping (review B3).
            _ => return Err(IpcError::Refused),
        };
        sys::map_region(r.fd.as_fd(), r.len, writable).map_err(|_e| IpcError::Refused)
    }

    pub(super) fn seal(&self, writer: RegionWriter) -> Result<ReadOnlyRegion, RegionRefused> {
        let Some(r) = self.own_region(writer.handle()) else {
            return Err(RegionRefused::new(writer.into_handle(), IpcError::Refused));
        };
        match sys::add_seals(r.fd.as_fd(), SEAL_WRITE | SEAL_SEAL) {
            Ok(()) => {}
            // A live mapping (review minor 6): an error, the writer back.
            Err(e) if e.raw_os_error() == Some(libc::EBUSY) => {
                return Err(RegionRefused::new(writer.into_handle(), IpcError::Busy));
            }
            Err(_) => return Err(RegionRefused::new(writer.into_handle(), IpcError::Refused)),
        }
        let h = new_handle(
            &self.inner,
            Arc::clone(&r.fd),
            r.len,
            HandleKind::ReadOnlyRegion,
        );
        drop(writer);
        ReadOnlyRegion::from_handle(h).map_err(|h| RegionRefused::new(h, IpcError::Refused))
    }

    pub(super) fn share(&self, writer: RegionWriter) -> Result<ProducerRegion, RegionRefused> {
        let Some(r) = self.own_region(writer.handle()) else {
            return Err(RegionRefused::new(writer.into_handle(), IpcError::Refused));
        };
        let mapping = match sys::map_region(r.fd.as_fd(), r.len, true) {
            Ok(m) => m,
            Err(_) => return Err(RegionRefused::new(writer.into_handle(), IpcError::Refused)),
        };
        if sys::add_seals(r.fd.as_fd(), SEAL_FUTURE_WRITE | SEAL_SEAL).is_err() {
            drop(mapping);
            return Err(RegionRefused::new(writer.into_handle(), IpcError::Refused));
        }
        let h = new_handle(
            &self.inner,
            Arc::clone(&r.fd),
            r.len,
            HandleKind::ProducerRegion,
        );
        drop(writer);
        ProducerRegion::new(h, mapping).map_err(|(h, _m)| RegionRefused::new(h, IpcError::Refused))
    }

    pub(super) fn consumer(&self, producer: &ProducerRegion) -> Result<ConsumerRegion, IpcError> {
        let r = self
            .own_region(producer.handle())
            .ok_or(IpcError::Refused)?;
        ConsumerRegion::from_handle(new_handle(
            &self.inner,
            Arc::clone(&r.fd),
            r.len,
            HandleKind::ConsumerRegion,
        ))
        .map_err(|_h| IpcError::Refused)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn this_kernel_passes_the_self_test() {
        assert!(self_test());
    }
}
