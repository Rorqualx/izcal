//! Simulated shared-memory regions (M1 phase D6; security scope IPC-R1,
//! IPC-R7).
//!
//! A region is bytes in the world's state plus a kind state: a writer,
//! sealed (by the broker or by a sandboxed process) or a producer. Handles
//! are owned like channel ends: a process sends only region handles it
//! holds, never a producer, and a writer only if it is the broker (a
//! process nobody launched; D6 review M1). Only the broker allocates. Each
//! owner is charged the region's full size against the quotas in
//! `hg_substrate::ipc` (review M4).
//!
//! The model follows the OS the world is configured as
//! ([`crate::SimOs`], review M7), so a scenario that would fail on an OS
//! fails here too:
//! - Linux refuses a seal while any mapping of the writer is live, in any
//!   process ([`IpcError::Busy`]; fact L1), and refuses to map a producer
//!   again (L3); a read-only region is immutable.
//! - macOS and Windows refuse a seal while the sealing process itself has
//!   a live mapping of the region (`Busy`, as the backends do); a mapping
//!   another process made earlier, or one a compromised process made
//!   with the OS directly ([`crate::World::hide_mappings`]), is not seen.
//! - On macOS a read-only region that crossed as `immutable` is mapped as a
//!   snapshot (fact M4); any other is shared, and a writer mapping another
//!   process kept across the seal still writes into it.
//! - On Windows a sandboxed process's seal does not stop a mapping another
//!   process kept, or a compromised filler hid (fact S8); a region that crosses as `immutable` is
//!   copied into a broker-filled one (review Q1).
//!
//! Unordered writes are torn at byte granularity: `write_from` lands a
//! seeded prefix at once and the rest later, at the writer's next release
//! store, seal or unmap, or (seeded) before some later read. A reader that
//! does not use a seqlock or a release/acquire pair can see the halves.

use core::any::Any;
use core::ptr::NonNull;
use core::sync::atomic::{AtomicBool, AtomicU8, Ordering};
use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

use hg_substrate::ipc::{
    AccessError, ConsumerRegion, Handle, HandleKind, HandleObject, IpcError,
    MAX_REGION_BYTES_PER_OWNER, MAX_REGION_BYTES_TOTAL, MAX_REGIONS_PER_OWNER, Mapping,
    MappingObject, OsPeer, ProducerRegion, ReadOnlyRegion, RegionObject, RegionRefused,
    RegionWriter, check_range, check_word,
};
use hg_substrate::prng::Xoshiro256StarStar;

use crate::ipc::{Owner, SimIpc};
use crate::world::{ProcessId, Shared, State, below};

/// Which OS's region behaviour the world models (D6 review M7).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SimOs {
    /// memfd seals.
    #[default]
    Linux,
    /// Mach memory entries.
    MacOs,
    /// Broker-relayed sections.
    Windows,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CellState {
    Writer,
    Sealed { by_broker: bool },
    Producer,
}

struct Cell {
    bytes: Vec<u8>,
    state: CellState,
    owner: ProcessId,
    /// Charged to `owner` until it is released ([`Ipc::release_owner`]);
    /// the bytes live on while anyone holds them.
    charged: bool,
    /// Live handles and mappings; at 0 the region is freed.
    refs: u32,
    /// Live mappings, and how many of them are writable.
    maps: u32,
    maps_rw: u32,
    /// Who made each live mapping (one entry per mapping).
    mapped_by: Vec<Owner>,
    /// The not yet visible tail of the last torn write: (offset, bytes).
    pending: Option<(usize, Vec<u8>)>,
}

struct HandleState {
    cell: u64,
    kind: HandleKind,
    owner: Owner,
    /// Attested by the receiving transport when the handle arrived.
    immutable: bool,
}

/// Every region in the world.
pub(crate) struct Regions {
    cells: BTreeMap<u64, Cell>,
    handles: BTreeMap<u64, HandleState>,
    next: u64,
    /// Processes that map behind their backend's back
    /// ([`crate::World::hide_mappings`]).
    hidden: BTreeSet<ProcessId>,
    /// The stream torn writes draw from, apart from the scheduler's, so
    /// a run without regions draws exactly what it did before.
    rng: Xoshiro256StarStar,
}

/// Marks `pid` as mapping behind its backend's back (see
/// [`crate::World::hide_mappings`]).
pub(crate) fn hide_mappings(st: &mut State, pid: ProcessId) {
    st.trace
        .record(format_args!("region hide-mappings {}", pid.0));
    st.regions.hidden.insert(pid);
}

impl Regions {
    pub(crate) fn new(rng: Xoshiro256StarStar) -> Self {
        Self {
            cells: BTreeMap::new(),
            handles: BTreeMap::new(),
            next: 0,
            hidden: BTreeSet::new(),
            rng,
        }
    }

    fn id(&mut self) -> u64 {
        let id = self.next;
        self.next = self.next.saturating_add(1);
        id
    }

    fn charged(&self, owner: ProcessId) -> (usize, u64) {
        self.cells
            .values()
            .filter(|c| c.charged && c.owner == owner)
            .fold((0, 0), |(n, b), c| {
                (n.saturating_add(1), b.saturating_add(len64(c.bytes.len())))
            })
    }

    fn total(&self) -> u64 {
        self.cells
            .values()
            .filter(|c| c.charged)
            .fold(0u64, |b, c| b.saturating_add(len64(c.bytes.len())))
    }

    /// Makes the last torn write's tail visible.
    fn flush(cell: &mut Cell) {
        if let Some((off, tail)) = cell.pending.take()
            && let Some(dst) = off
                .checked_add(tail.len())
                .and_then(|end| cell.bytes.get_mut(off..end))
        {
            dst.copy_from_slice(&tail);
        }
    }

    fn release(st: &mut State, cell: u64) {
        let Some(c) = st.regions.cells.get_mut(&cell) else {
            return;
        };
        c.refs = c.refs.saturating_sub(1);
        if c.refs == 0 {
            st.regions.cells.remove(&cell);
            st.trace.record(format_args!("region free #{cell}"));
        }
    }

    /// A handle that arrived in a message now belongs to `to`; the
    /// receiving transport's attestation is recorded with it.
    pub(crate) fn arrive(st: &mut State, h: &Handle, to: Owner, os: SimOs) {
        let Some(r) = h.object().downcast_ref::<SimRegion>() else {
            return;
        };
        let Some(hs) = st.regions.handles.get_mut(&r.id) else {
            return;
        };
        hs.owner = to;
        let cell = hs.cell;
        let kind = hs.kind;
        let want = r.want.load(Ordering::Relaxed);
        let state = st.regions.cells.get(&cell).map(|c| c.state);
        let immutable = kind == HandleKind::ReadOnlyRegion
            && match os {
                SimOs::Linux => true,
                SimOs::MacOs => want,
                SimOs::Windows => matches!(state, Some(CellState::Sealed { by_broker: true })),
            };
        if let Some(hs) = st.regions.handles.get_mut(&r.id) {
            hs.immutable = immutable;
        }
    }

    /// Whether `h` is a region handle `me` holds and may send: a writer
    /// only from the broker, a producer never. Changes nothing.
    pub(crate) fn may_send(st: &State, h: &Handle, me: Owner, broker: bool) -> bool {
        let Some(r) = h.object().downcast_ref::<SimRegion>() else {
            return false;
        };
        let Some(hs) = st.regions.handles.get(&r.id) else {
            return false;
        };
        hs.owner == me
            && match hs.kind {
                HandleKind::RegionWriter => broker,
                HandleKind::ReadOnlyRegion | HandleKind::ConsumerRegion => true,
                HandleKind::ProducerRegion
                | HandleKind::ChannelSender
                | HandleKind::BoundChannelSender
                | HandleKind::ChannelReceiver
                | HandleKind::GpuIoSurface
                | HandleKind::GpuDmaBuf
                | HandleKind::GpuDxgiShared => false,
            }
    }

    /// `h` leaves its holder (after [`Regions::may_send`]). On Windows an
    /// `immutable` read-only region a sandboxed process sealed is first
    /// copied into a region the broker filled (review Q1), and the handle
    /// names the copy.
    pub(crate) fn send(st: &mut State, h: &Handle) {
        let Some(r) = h.object().downcast_ref::<SimRegion>() else {
            return;
        };
        let Some(hs) = st.regions.handles.get(&r.id) else {
            return;
        };
        let (cell, kind, want) = (hs.cell, hs.kind, r.want.load(Ordering::Relaxed));
        if kind == HandleKind::ReadOnlyRegion
            && want
            && st.config.region_os == SimOs::Windows
            && st
                .regions
                .cells
                .get(&cell)
                .is_some_and(|c| c.state == (CellState::Sealed { by_broker: false }))
            && let Some(c) = st.regions.cells.get_mut(&cell)
        {
            Regions::flush(c);
            let copy = Cell {
                bytes: c.bytes.clone(),
                state: CellState::Sealed { by_broker: true },
                owner: c.owner,
                charged: c.charged,
                refs: 1,
                maps: 0,
                maps_rw: 0,
                mapped_by: Vec::new(),
                pending: None,
            };
            let new = st.regions.id();
            st.regions.cells.insert(new, copy);
            if let Some(hs) = st.regions.handles.get_mut(&r.id) {
                hs.cell = new;
            }
            Regions::release(st, cell);
            st.trace
                .record(format_args!("region copy #{cell} -> #{new} (immutable)"));
        }
        if let Some(hs) = st.regions.handles.get_mut(&r.id) {
            hs.owner = Owner::Transit;
        }
    }

    /// Releases `pid`'s charges: it crashed or exited (what the launcher
    /// reports to an OS broker through `release_owner`).
    pub(crate) fn owner_gone(st: &mut State, pid: ProcessId) {
        let mut n = 0u32;
        for c in st
            .regions
            .cells
            .values_mut()
            .filter(|c| c.owner == pid && c.charged)
        {
            c.charged = false;
            n = n.saturating_add(1);
        }
        if n > 0 {
            st.trace
                .record(format_args!("region release owner {} ({n})", pid.0));
        }
    }

    /// Whether `h` is a writer, for the receiving side's broker check.
    pub(crate) fn is_writer(h: &Handle) -> bool {
        h.kind() == HandleKind::RegionWriter
    }
}

fn len64(n: usize) -> u64 {
    u64::try_from(n).unwrap_or(u64::MAX)
}

const K_WRITER: u8 = 0;
const K_READ_ONLY: u8 = 1;
const K_CONSUMER: u8 = 2;
const K_PRODUCER: u8 = 3;

fn kind_code(k: HandleKind) -> u8 {
    match k {
        HandleKind::ReadOnlyRegion => K_READ_ONLY,
        HandleKind::ConsumerRegion => K_CONSUMER,
        HandleKind::ProducerRegion => K_PRODUCER,
        // Only the four region kinds are ever stored.
        HandleKind::RegionWriter
        | HandleKind::ChannelSender
        | HandleKind::BoundChannelSender
        | HandleKind::ChannelReceiver
        | HandleKind::GpuIoSurface
        | HandleKind::GpuDmaBuf
        | HandleKind::GpuDxgiShared => K_WRITER,
    }
}

fn code_kind(c: u8) -> HandleKind {
    match c {
        K_READ_ONLY => HandleKind::ReadOnlyRegion,
        K_CONSUMER => HandleKind::ConsumerRegion,
        K_PRODUCER => HandleKind::ProducerRegion,
        _ => HandleKind::RegionWriter,
    }
}

/// A region handle this transport made. Sealing or sharing a writer turns
/// this same object into the new kind.
pub(crate) struct SimRegion {
    shared: Arc<Shared>,
    id: u64,
    kind: AtomicU8,
    len: usize,
    want: AtomicBool,
}

impl core::fmt::Debug for SimRegion {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("SimRegion")
            .field("id", &self.id)
            .field("kind", &code_kind(self.kind.load(Ordering::Relaxed)))
            .finish()
    }
}

impl HandleObject for SimRegion {
    fn kind(&self) -> HandleKind {
        code_kind(self.kind.load(Ordering::Relaxed))
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

impl RegionObject for SimRegion {
    fn len(&self) -> usize {
        self.len
    }
    fn immutable(&self) -> bool {
        let st = self.shared.lock();
        st.regions
            .handles
            .get(&self.id)
            .is_some_and(|h| h.immutable)
    }
    fn want_immutable(&self) {
        self.want.store(true, Ordering::Relaxed);
    }
}

impl Drop for SimRegion {
    fn drop(&mut self) {
        let mut st = self.shared.lock();
        if let Some(hs) = st.regions.handles.remove(&self.id) {
            Regions::release(&mut st, hs.cell);
        }
    }
}

/// A mapping of a simulated region.
struct SimMapping {
    shared: Arc<Shared>,
    cell: u64,
    /// The process that made it.
    by: Owner,
    len: usize,
    writable: bool,
    /// A macOS snapshot: the bytes at map time.
    snapshot: Option<Vec<u8>>,
    /// Stable bytes for `base`; the simulator models only the checked
    /// accessors (the raw base is outside DST scope).
    raw: Box<[u8]>,
}

impl core::fmt::Debug for SimMapping {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("SimMapping")
            .field("cell", &self.cell)
            .field("len", &self.len)
            .field("writable", &self.writable)
            .finish()
    }
}

impl SimMapping {
    fn with_cell<T>(&self, f: impl FnOnce(&mut Cell, &mut Xoshiro256StarStar) -> T) -> Option<T> {
        let mut st = self.shared.lock();
        let st = &mut *st;
        let c = st.regions.cells.get_mut(&self.cell)?;
        Some(f(c, &mut st.regions.rng))
    }

    fn word(&self, off: usize, size: usize) -> Result<Vec<u8>, AccessError> {
        check_word(off, size, self.len)?;
        let mut b = vec![0u8; size];
        if let Some(s) = &self.snapshot {
            let end = off.checked_add(size).ok_or(AccessError::OutOfBounds)?;
            b.copy_from_slice(s.get(off..end).ok_or(AccessError::OutOfBounds)?);
            return Ok(b);
        }
        self.with_cell(|c, _| {
            let end = off.checked_add(size)?;
            b.copy_from_slice(c.bytes.get(off..end)?);
            Some(())
        })
        .flatten()
        .ok_or(AccessError::OutOfBounds)?;
        Ok(b)
    }

    fn store(&self, off: usize, bytes: &[u8]) -> Result<(), AccessError> {
        check_word(off, bytes.len(), self.len)?;
        if !self.writable {
            return Err(AccessError::ReadOnly);
        }
        self.with_cell(|c, _| {
            // A release store publishes every earlier write.
            Regions::flush(c);
            let end = off.checked_add(bytes.len())?;
            c.bytes.get_mut(off..end)?.copy_from_slice(bytes);
            Some(())
        })
        .flatten()
        .ok_or(AccessError::OutOfBounds)
    }
}

impl MappingObject for SimMapping {
    fn len(&self) -> usize {
        self.len
    }
    fn writable(&self) -> bool {
        self.writable
    }
    fn read(&self, off: usize, out: &mut [u8]) -> Result<(), AccessError> {
        check_range(off, out.len(), self.len)?;
        let end = off.checked_add(out.len()).ok_or(AccessError::OutOfBounds)?;
        if let Some(s) = &self.snapshot {
            out.copy_from_slice(s.get(off..end).ok_or(AccessError::OutOfBounds)?);
            return Ok(());
        }
        self.with_cell(|c, rng| {
            // Sometimes the torn tail lands before this read.
            if c.pending.is_some() && below(rng, 2) == 0 {
                Regions::flush(c);
            }
            out.copy_from_slice(c.bytes.get(off..end)?);
            Some(())
        })
        .flatten()
        .ok_or(AccessError::OutOfBounds)
    }
    fn write(&self, off: usize, data: &[u8]) -> Result<(), AccessError> {
        check_range(off, data.len(), self.len)?;
        if !self.writable {
            return Err(AccessError::ReadOnly);
        }
        self.with_cell(|c, rng| {
            Regions::flush(c);
            // Land a seeded prefix now and the rest later (torn at byte
            // granularity).
            let n = data.len();
            let k = usize::try_from(below(rng, len64(n).saturating_add(1))).unwrap_or(n);
            let (now, later) = data.split_at(k.min(n));
            let mid = off.checked_add(k.min(n))?;
            c.bytes.get_mut(off..mid)?.copy_from_slice(now);
            if !later.is_empty() {
                c.pending = Some((mid, later.to_vec()));
            }
            Some(())
        })
        .flatten()
        .ok_or(AccessError::OutOfBounds)
    }
    fn load_u32(&self, off: usize, _order: Ordering) -> Result<u32, AccessError> {
        let b = self.word(off, 4)?;
        Ok(u32::from_le_bytes([
            b.first().copied().unwrap_or(0),
            b.get(1).copied().unwrap_or(0),
            b.get(2).copied().unwrap_or(0),
            b.get(3).copied().unwrap_or(0),
        ]))
    }
    fn store_u32(&self, off: usize, v: u32, _order: Ordering) -> Result<(), AccessError> {
        self.store(off, &v.to_le_bytes())
    }
    fn load_u64(&self, off: usize, _order: Ordering) -> Result<u64, AccessError> {
        let b = self.word(off, 8)?;
        let mut a = [0u8; 8];
        a.copy_from_slice(&b);
        Ok(u64::from_le_bytes(a))
    }
    fn store_u64(&self, off: usize, v: u64, _order: Ordering) -> Result<(), AccessError> {
        self.store(off, &v.to_le_bytes())
    }
    fn base(&self) -> NonNull<u8> {
        NonNull::new(self.raw.as_ptr().cast_mut()).unwrap_or(NonNull::dangling())
    }
}

impl Drop for SimMapping {
    fn drop(&mut self) {
        let mut st = self.shared.lock();
        if let Some(c) = st.regions.cells.get_mut(&self.cell) {
            Regions::flush(c);
            c.maps = c.maps.saturating_sub(1);
            if let Some(i) = c.mapped_by.iter().position(|o| *o == self.by) {
                c.mapped_by.swap_remove(i);
            }
            if self.writable {
                c.maps_rw = c.maps_rw.saturating_sub(1);
            }
        }
        Regions::release(&mut st, self.cell);
    }
}

impl SimIpc {
    fn new_handle(&self, st: &mut State, cell: u64, kind: HandleKind, len: usize) -> SimRegion {
        let id = st.regions.id();
        st.regions.handles.insert(
            id,
            HandleState {
                cell,
                kind,
                owner: self.me(),
                immutable: kind == HandleKind::ReadOnlyRegion
                    && match st.config.region_os {
                        SimOs::Linux => true,
                        SimOs::MacOs => false,
                        SimOs::Windows => matches!(
                            st.regions.cells.get(&cell).map(|c| c.state),
                            Some(CellState::Sealed { by_broker: true })
                        ),
                    },
            },
        );
        if let Some(c) = st.regions.cells.get_mut(&cell) {
            c.refs = c.refs.saturating_add(1);
        }
        SimRegion {
            shared: Arc::clone(&self.shared),
            id,
            kind: AtomicU8::new(kind_code(kind)),
            len,
            want: AtomicBool::new(false),
        }
    }

    /// The handle `h` names if it is this transport's region and `me`
    /// holds it: (handle id, cell, kind).
    fn held_region(&self, st: &State, h: &Handle) -> Option<(u64, u64, HandleKind)> {
        let r = h.object().downcast_ref::<SimRegion>()?;
        if !Arc::ptr_eq(&r.shared, &self.shared) {
            return None;
        }
        let hs = st.regions.handles.get(&r.id)?;
        (hs.owner == self.me()).then_some((r.id, hs.cell, hs.kind))
    }

    pub(crate) fn region_allocate(
        &self,
        len: usize,
        owner: OsPeer,
    ) -> Result<RegionWriter, IpcError> {
        let mut st = self.shared.lock();
        if !self.is_broker(&st) {
            return Err(IpcError::Unsupported);
        }
        if len == 0 || u32::try_from(len).is_err() {
            return Err(IpcError::Refused);
        }
        let owner = ProcessId(owner.pid());
        let (n, bytes) = st.regions.charged(owner);
        let total = st.regions.total();
        let l = len64(len);
        if n >= MAX_REGIONS_PER_OWNER
            || bytes.saturating_add(l) > MAX_REGION_BYTES_PER_OWNER
            || total.saturating_add(l) > MAX_REGION_BYTES_TOTAL
        {
            st.trace
                .record(format_args!("region alloc {len} refused: quota"));
            return Err(IpcError::Quota);
        }
        let cell = st.regions.id();
        st.regions.cells.insert(
            cell,
            Cell {
                bytes: vec![0; len],
                state: CellState::Writer,
                owner,
                charged: true,
                refs: 0,
                maps: 0,
                maps_rw: 0,
                mapped_by: Vec::new(),
                pending: None,
            },
        );
        st.trace
            .record(format_args!("region alloc #{cell} {len} for {}", owner.0));
        let h = self.new_handle(&mut st, cell, HandleKind::RegionWriter, len);
        drop(st);
        RegionWriter::from_handle(Handle::new(Box::new(h))).map_err(|_| IpcError::Refused)
    }

    pub(crate) fn region_map(&self, h: &Handle) -> Result<Mapping, IpcError> {
        let mut st = self.shared.lock();
        let Some((hid, cell, kind)) = self.held_region(&st, h) else {
            return Err(IpcError::Refused);
        };
        let os = st.config.region_os;
        let snap_wanted = os == SimOs::MacOs
            && kind == HandleKind::ReadOnlyRegion
            && st.regions.handles.get(&hid).is_some_and(|x| x.immutable);
        let Some(c) = st.regions.cells.get_mut(&cell) else {
            return Err(IpcError::Closed);
        };
        let writable = match (kind, c.state) {
            (HandleKind::RegionWriter, CellState::Writer) => true,
            // A Linux producer can never be mapped again (FUTURE_WRITE).
            (HandleKind::ProducerRegion, _) if os == SimOs::Linux => return Err(IpcError::Refused),
            (HandleKind::ProducerRegion, CellState::Producer) => true,
            (HandleKind::ReadOnlyRegion, CellState::Sealed { .. })
            | (HandleKind::ConsumerRegion, CellState::Producer) => false,
            _ => return Err(IpcError::Refused),
        };
        // A snapshot holds what is visible now; a torn write's tail stays
        // pending for shared mappings.
        let snapshot = snap_wanted.then(|| c.bytes.clone());
        let len = c.bytes.len();
        c.maps = c.maps.saturating_add(1);
        c.mapped_by.push(self.me());
        c.refs = c.refs.saturating_add(1);
        if writable {
            c.maps_rw = c.maps_rw.saturating_add(1);
        }
        st.trace.record(format_args!(
            "region map #{cell} {}{}",
            if writable { "rw" } else { "r" },
            if snapshot.is_some() { " snapshot" } else { "" }
        ));
        drop(st);
        Ok(Mapping::new(Box::new(SimMapping {
            shared: Arc::clone(&self.shared),
            cell,
            by: self.me(),
            len,
            writable,
            snapshot,
            raw: vec![0u8; len].into_boxed_slice(),
        })))
    }

    /// Turns the writer `w` names into `to`: the same object, relabelled.
    fn relabel(st: &mut State, w: &Handle, to: HandleKind) {
        if let Some(r) = w.object().downcast_ref::<SimRegion>() {
            r.kind.store(kind_code(to), Ordering::Relaxed);
            let immutable = to == HandleKind::ReadOnlyRegion && st.config.region_os == SimOs::Linux;
            if let Some(hs) = st.regions.handles.get_mut(&r.id) {
                hs.kind = to;
                hs.immutable = immutable;
            }
        }
    }

    pub(crate) fn region_seal(&self, w: RegionWriter) -> Result<ReadOnlyRegion, RegionRefused> {
        let mut st = self.shared.lock();
        let held = self.held_region(&st, w.handle());
        let error = match held {
            Some((_, cell, HandleKind::RegionWriter)) => {
                let me = self.me();
                let hidden = matches!(me, Owner::Proc(p, _) if st.regions.hidden.contains(&p));
                let live = st.regions.cells.get(&cell).is_some_and(|c| {
                    if st.config.region_os == SimOs::Linux {
                        // F_SEAL_WRITE fails while any shared mapping
                        // exists (fact L1).
                        c.maps > 0
                    } else {
                        // The macOS and Windows backends see only this
                        // process's own mappings, made through them.
                        !hidden && c.mapped_by.contains(&me)
                    }
                });
                // The caller gets an error, not a violation.
                live.then_some(IpcError::Busy)
            }
            _ => Some(IpcError::Refused),
        };
        let Some((_, cell, _)) = held.filter(|_| error.is_none()) else {
            drop(st);
            return Err(RegionRefused::new(
                w.into_handle(),
                error.unwrap_or(IpcError::Refused),
            ));
        };
        let by_broker = self.is_broker(&st);
        if let Some(c) = st.regions.cells.get_mut(&cell) {
            Regions::flush(c);
            c.state = CellState::Sealed { by_broker };
        }
        let h = w.into_handle();
        Self::relabel(&mut st, &h, HandleKind::ReadOnlyRegion);
        // The broker filled it itself: immutable on Windows too.
        if st.config.region_os == SimOs::Windows
            && by_broker
            && let Some(r) = h.object().downcast_ref::<SimRegion>()
            && let Some(hs) = st.regions.handles.get_mut(&r.id)
        {
            hs.immutable = true;
        }
        st.trace.record(format_args!(
            "region seal #{cell} by {}",
            if by_broker { "broker" } else { "child" }
        ));
        drop(st);
        ReadOnlyRegion::from_handle(h).map_err(|h| RegionRefused::new(h, IpcError::Refused))
    }

    pub(crate) fn region_share(&self, w: RegionWriter) -> Result<ProducerRegion, RegionRefused> {
        let mut st = self.shared.lock();
        let Some((_, cell, HandleKind::RegionWriter)) = self.held_region(&st, w.handle()) else {
            drop(st);
            return Err(RegionRefused::new(w.into_handle(), IpcError::Refused));
        };
        let len = w.len();
        if let Some(c) = st.regions.cells.get_mut(&cell) {
            c.state = CellState::Producer;
            // The producer's one writable mapping, made before the region
            // closes to new ones.
            c.maps = c.maps.saturating_add(1);
            c.maps_rw = c.maps_rw.saturating_add(1);
            c.mapped_by.push(self.me());
            c.refs = c.refs.saturating_add(1);
        }
        let h = w.into_handle();
        Self::relabel(&mut st, &h, HandleKind::ProducerRegion);
        st.trace.record(format_args!("region share #{cell}"));
        drop(st);
        let mapping = Mapping::new(Box::new(SimMapping {
            shared: Arc::clone(&self.shared),
            cell,
            by: self.me(),
            len,
            writable: true,
            snapshot: None,
            raw: vec![0u8; len].into_boxed_slice(),
        }));
        ProducerRegion::new(h, mapping).map_err(|(h, m)| {
            drop(m);
            RegionRefused::new(h, IpcError::Refused)
        })
    }

    pub(crate) fn region_consumer(&self, p: &ProducerRegion) -> Result<ConsumerRegion, IpcError> {
        let mut st = self.shared.lock();
        let Some((_, cell, HandleKind::ProducerRegion)) = self.held_region(&st, p.handle()) else {
            return Err(IpcError::Refused);
        };
        let h = self.new_handle(&mut st, cell, HandleKind::ConsumerRegion, p.len());
        drop(st);
        ConsumerRegion::from_handle(Handle::new(Box::new(h))).map_err(|_| IpcError::Refused)
    }

    pub(crate) fn region_release_owner(&self, owner: OsPeer) {
        let mut st = self.shared.lock();
        if !self.is_broker(&st) {
            return;
        }
        let pid = ProcessId(owner.pid());
        let mut n = 0u32;
        for c in st
            .regions
            .cells
            .values_mut()
            .filter(|c| c.owner == pid && c.charged)
        {
            c.charged = false;
            n = n.saturating_add(1);
        }
        st.trace
            .record(format_args!("region release owner {} ({n})", pid.0));
    }

    /// Whether this process is the broker: a process nobody launched.
    pub(crate) fn is_broker(&self, st: &State) -> bool {
        st.proc(self.pid, self.generation)
            .is_some_and(|p| !p.launched)
    }
}
