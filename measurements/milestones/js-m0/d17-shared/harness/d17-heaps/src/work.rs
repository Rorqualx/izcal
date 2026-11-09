//! The agent workload: the objects the agents allocate, and the three
//! shapes of allocation a process with several agents actually performs.

use hg_heap::{AllocError, Any, Heap, HeapConfig, Mutator, ProcessCage, Stress};
use hg_substrate::{DecisionLog, NotRecording};
use std::sync::Arc;

hg_heap::object! {
    /// A list cell: two traced fields and a counter, the smallest object
    /// that holds a reference.
    pub Cell {
        traced NEXT: Cell,
        traced OTHER: Any,
        raw N: u32,
        tail none,
    }
}

hg_heap::object! {
    /// An array of slots, for allocations larger than a block.
    pub Arr {
        tail slots,
    }
}

/// Cage bytes for every heap in this harness: the production figure
/// (JS-D17: 16 GiB).
pub const CAGE_BYTES: u64 = 16 << 30;

/// A heap's config with no canonical cycles (the grow and churn modes
/// want the heap to grow, so the collector must not reclaim).
pub fn no_gc(agent: u32) -> HeapConfig {
    HeapConfig {
        cage_bytes: CAGE_BYTES,
        trigger: u64::MAX,
        stress: Stress::None,
        agent,
    }
}

/// A heap's config that collects every `trigger` charged units.
pub fn with_gc(agent: u32, trigger: u64) -> HeapConfig {
    HeapConfig {
        cage_bytes: CAGE_BYTES,
        trigger,
        stress: Stress::None,
        agent,
    }
}

/// A decision log that records nothing, so logging is not in the way of
/// what is being measured.
pub fn log() -> Arc<dyn DecisionLog> {
    Arc::new(NotRecording)
}

/// An agent heap in `cage`.
pub fn heap_in(cage: &ProcessCage, config: HeapConfig) -> Heap {
    Heap::in_cage(cage, log(), config)
}

/// What one unit of work produced: the checksum the arithmetic must have
/// come to, and how many allocations succeeded.
#[derive(Debug, Clone, Copy, Default)]
pub struct Did {
    /// The sum of the counters read back out of the heap.
    pub checksum: u64,
    /// Allocations that succeeded.
    pub allocs: u64,
    /// Allocations refused because the cage was full.
    pub refused: u64,
}

/// **grow**: a live chain of `n` cells, all reachable, so the heap grows
/// monotonically and takes chunks from the cage as it goes. Every cell's
/// counter is read back, so the allocations cannot be dead code.
pub fn grow(h: &mut Heap, n: u64) -> Did {
    h.enter(|m| {
        m.scope(|m, s| {
            let mut did = Did::default();
            let Ok(head) = m.alloc::<Cell>(s, 0) else {
                did.refused += 1;
                return did;
            };
            did.allocs += 1;
            for i in 0..n {
                match m.alloc::<Cell>(s, 0) {
                    Ok(c) => {
                        did.allocs += 1;
                        let (hd, cg) = (head.get(m), c.get(m));
                        cg.set(m, Cell::NEXT, hd.get(m, Cell::NEXT));
                        cg.set_raw(m, Cell::N, u32::try_from(i & 0xFFFF).unwrap_or(0));
                        hd.set(m, Cell::NEXT, Some(cg));
                    }
                    Err(AllocError::OutOfMemory) => {
                        did.refused += 1;
                        break;
                    }
                }
            }
            // Walk the chain back, so every allocation is read.
            let mut cur = head.get(m).get(m, Cell::NEXT);
            while let Some(c) = cur {
                did.checksum += u64::from(c.raw(m, Cell::N));
                cur = c.get(m, Cell::NEXT);
            }
            did
        })
    })
}

/// **steady**: `n` cells allocated in inner scopes that exit, so they die
/// and the collector reuses the blocks; the heap stops growing and the
/// cage sees almost no traffic. `trigger` units are charged per
/// allocation.
pub fn steady(h: &mut Heap, n: u64) -> Did {
    h.enter(|m| {
        let mut did = Did::default();
        for i in 0..n {
            let r = m.scope(|m, s| -> Result<u64, AllocError> {
                let c = m.alloc::<Cell>(s, 0)?;
                let g = c.get(m);
                g.set_raw(m, Cell::N, u32::try_from(i & 0xFFFF).unwrap_or(0));
                let got = u64::from(g.raw(m, Cell::N));
                Ok(got)
            });
            match r {
                Ok(v) => {
                    did.allocs += 1;
                    did.checksum += v;
                    m.charge(1);
                }
                Err(AllocError::OutOfMemory) => {
                    did.refused += 1;
                    break;
                }
            }
        }
        did
    })
}

/// Allocates `blocks` block-sized arrays into one scope, which is how an
/// agent heap takes whole chunks from the cage quickly.
pub fn take_chunks(m: &mut Mutator<'_>, blocks: u64) -> Did {
    m.scope(|m, s| {
        let mut did = Did::default();
        // 4096 slots is 16 KiB of tail; with the header that is under one
        // 32 KiB block, so each allocation starts a fresh block.
        for i in 0..blocks {
            match m.alloc::<Arr>(s, 4096) {
                Ok(a) => {
                    did.allocs += 1;
                    let g = a.get(m);
                    g.set_tail(m, 0, hg_heap::Value::Small(1));
                    if let Some(hg_heap::Value::Small(v)) = g.tail(m, 0) {
                        did.checksum += u64::try_from(v).unwrap_or(0);
                    }
                    let _ = i;
                }
                Err(AllocError::OutOfMemory) => {
                    did.refused += 1;
                    break;
                }
            }
        }
        did
    })
}

/// **churn**: `rounds` agent heaps, each made in `cage`, each taking
/// `blocks` blocks and then dropped, which returns its chunks to the
/// cage. This is the agent-lifecycle shape: a process whose agents come
/// and go.
pub fn churn(cage: &ProcessCage, agent: u32, rounds: u64, blocks: u64) -> Did {
    let mut did = Did::default();
    for _ in 0..rounds {
        let mut h = heap_in(cage, no_gc(agent));
        let d = h.enter(|m| take_chunks(m, blocks));
        did.allocs += d.allocs;
        did.refused += d.refused;
        did.checksum += d.checksum;
        drop(h);
    }
    did
}
