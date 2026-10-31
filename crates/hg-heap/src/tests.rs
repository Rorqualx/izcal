//! The heap's tests. They run on a small cage backed by an ordinary
//! allocation, so Miri can run them (it cannot map a real reservation).

use std::sync::{Arc, Mutex};

use hg_substrate::{DecisionKind, DecisionLog, Position};

use crate::cage::test_memory::TestMemory;

use crate::{Any, Heap, HeapConfig, Object, Stress, Value};

/// A decision log that keeps every entry.
#[derive(Default)]
struct Log(Mutex<Vec<(DecisionKind, Position, u64)>>);

impl DecisionLog for Log {
    fn record(&self, kind: DecisionKind, at: Position, detail: u64) {
        self.0.lock().unwrap().push((kind, at, detail));
    }
}

const CAGE: u64 = 4 << 20;

fn heap(stress: Stress, trigger: u64) -> (Heap, Arc<Log>) {
    let log = Arc::new(Log::default());
    let h = Heap::new(
        Arc::new(TestMemory::default()),
        Arc::clone(&log) as Arc<dyn DecisionLog>,
        HeapConfig {
            cage_bytes: CAGE,
            trigger,
            stress,
            agent: 7,
        },
    )
    .unwrap();
    (h, log)
}

crate::object! {
    /// A list cell.
    pub(crate) Cell {
        traced NEXT: Cell,
        traced VALUE: Any,
        raw N: u32,
        raw W: f64,
        tail none,
    }
}

crate::object! {
    /// A weak reference cell.
    pub(crate) WeakCell {
        weak TARGET: Any,
        tail none,
    }
}

crate::object! {
    /// An array of values.
    pub(crate) Array {
        tail slots,
    }
}

crate::object! {
    /// A byte string.
    pub(crate) Bytes {
        tail bytes,
    }
}

/// Builds a list of `n` cells holding 0..n, rooted, and returns the
/// digest and the sum of the values read back.
fn list(stress: Stress, n: u32) -> (u64, i64, u64) {
    let (mut h, _) = heap(stress, 1 << 40);
    let r = h.enter(|m| {
        m.scope(|m, s| {
            let head = m.alloc::<Cell>(s, 0).unwrap();
            for i in 0..n {
                let c = m.alloc::<Cell>(s, 0).unwrap();
                let (hd, cg) = (head.get(m), c.get(m));
                cg.set(m, Cell::NEXT, hd.get(m, Cell::NEXT));
                cg.set_value(m, Cell::VALUE, Value::Small(i32::try_from(i).unwrap()));
                cg.set_raw(m, Cell::N, i);
                cg.set_raw(m, Cell::W, f64::from(i) / 2.0);
                hd.set(m, Cell::NEXT, Some(cg));
                m.safepoint();
            }
            let mut sum = 0i64;
            let mut cur = head.get(m).get(m, Cell::NEXT);
            while let Some(c) = cur {
                if let Value::Small(v) = c.get_value(m, Cell::VALUE) {
                    sum += i64::from(v);
                    assert_eq!(c.raw(m, Cell::N), u32::try_from(v).unwrap());
                    assert_eq!(c.raw(m, Cell::W), f64::from(v) / 2.0);
                }
                cur = c.get(m, Cell::NEXT);
            }
            (m.digest(), sum)
        })
    });
    (r.0, r.1, h.stats().moved)
}

#[test]
fn fields_survive_and_every_collector_mode_gives_the_same_graph() {
    let n = 60;
    let want = i64::from(n) * i64::from(n - 1) / 2;
    let (d0, sum0, moved0) = list(Stress::None, n);
    let (d1, sum1, moved1) = list(Stress::MoveOnAlloc, n);
    let (d2, sum2, moved2) = list(Stress::MoveAtSafepoint, n);
    assert_eq!((sum0, sum1, sum2), (want, want, want));
    assert_eq!(moved0, 0);
    assert!(moved1 > 0 && moved2 > 0, "{moved1} {moved2}");
    // The digest is independent of where the objects live.
    assert_eq!(d0, d1);
    assert_eq!(d0, d2);
}

#[test]
fn unrooted_objects_are_collected_and_their_blocks_freed() {
    let (mut h, log) = heap(Stress::None, 1 << 40);
    h.enter(|m| {
        m.scope(|m, s| {
            // About 3 MiB of garbage in 1 KiB arrays: many blocks.
            for _ in 0..3000 {
                m.scope(|m, s2| {
                    let _ = m.alloc::<Array>(s2, 250).unwrap();
                });
            }
            let keep = m.alloc::<Array>(s, 4).unwrap();
            keep.get(m).set_tail(m, 3, Value::Small(9));
            let before = m.stats().blocks_used;
            m.advance(41);
            m.collect();
            let after = m.stats().blocks_used;
            assert!(before > 50 && after <= 2, "{before} {after}");
            assert_eq!(keep.get(m).tail(m, 3), Some(Value::Small(9)));
            assert_eq!(keep.get(m).tail(m, 4), None);
        });
    });
    let log = log.0.lock().unwrap();
    assert_eq!(
        *log,
        vec![(
            DecisionKind::GC_COMPLETE,
            Position {
                agent: 7,
                progress: 41
            },
            0
        )]
    );
}

#[test]
fn canonical_cycles_clear_dead_weak_targets_and_others_never_do() {
    for stress in [Stress::None, Stress::MoveOnAlloc, Stress::MoveAtSafepoint] {
        let (mut h, log) = heap(stress, 1 << 40);
        h.enter(|m| {
            m.scope(|m, s| {
                let weak = m.alloc::<WeakCell>(s, 0).unwrap();
                let held = m.alloc::<Cell>(s, 0).unwrap();
                let kept = m.alloc::<WeakCell>(s, 0).unwrap();
                m.scope(|m, s2| {
                    let dies = m.alloc::<Cell>(s2, 0).unwrap();
                    let w = weak.get(m);
                    w.set_weak(m, WeakCell::TARGET, Some(dies.get(m).erase()));
                    let k = kept.get(m);
                    k.set_weak(m, WeakCell::TARGET, Some(held.get(m).erase()));
                    // A deref keeps a target alive until ClearKeptObjects.
                    let only_weak = m.alloc::<Cell>(s2, 0).unwrap();
                    let kept_target = only_weak.get(m);
                    m.keep_alive(kept_target);
                    let h2 = held.get(m);
                    h2.set_value(m, Cell::VALUE, Value::Ref(kept_target.erase()));
                    h2.set_value(m, Cell::VALUE, Value::Small(0));
                });
                // Non-canonical cycles (stress) never clear.
                m.safepoint();
                assert!(weak.get(m).get_weak(m, WeakCell::TARGET).is_some());
                m.collect();
                assert!(weak.get(m).get_weak(m, WeakCell::TARGET).is_none());
                assert!(kept.get(m).get_weak(m, WeakCell::TARGET).is_some());
                m.clear_kept_objects();
                m.collect();
            });
        });
        let log = log.0.lock().unwrap();
        let details: Vec<u64> = log.iter().map(|e| e.2).collect();
        assert_eq!(details, vec![1, 0], "{stress:?}");
    }
}

#[test]
fn byte_tails_and_large_objects_round_trip() {
    let (mut h, _) = heap(Stress::MoveOnAlloc, 1 << 40);
    h.enter(|m| {
        m.scope(|m, s| {
            let b = m.alloc::<Bytes>(s, 13).unwrap();
            for i in 0..13u8 {
                b.get(m).set_byte(m, u32::from(i), i * 3);
            }
            // 40 KiB: a large object, never moved.
            let big = m.alloc::<Array>(s, 10_000).unwrap();
            big.get(m).set_tail(m, 9_999, Value::Small(-4));
            let before = big.get(m).slot();
            let _ = m.alloc::<Cell>(s, 0).unwrap(); // moves the small ones
            assert_eq!(big.get(m).slot(), before);
            assert_eq!(big.get(m).tail(m, 9_999), Some(Value::Small(-4)));
            for i in 0..13u8 {
                assert_eq!(b.get(m).byte(m, u32::from(i)), Some(i * 3));
            }
            assert_eq!(b.get(m).byte(m, 13), None);
        });
    });
}

#[test]
fn a_checked_cast_follows_the_header() {
    let (mut h, _) = heap(Stress::None, 1 << 40);
    h.enter(|m| {
        m.scope(|m, s| {
            let c = m.alloc::<Cell>(s, 0).unwrap();
            let _ = m.alloc::<Array>(s, 1).unwrap(); // registers Array
            let any = c.get(m).erase();
            assert!(any.cast::<Cell>(m).is_some());
            assert!(any.cast::<Array>(m).is_none());
            assert!(any.cast::<WeakCell>(m).is_none()); // never allocated
        });
    });
}

#[test]
fn the_canonical_trigger_counts_object_bytes() {
    // Each Cell is 8 + 8 + 16 = 32 bytes; a trigger of 320 gives a cycle
    // every 10 allocations.
    assert_eq!(<Cell as Object>::LAYOUT.size(0), Some(32));
    let (mut h, log) = heap(Stress::None, 320);
    h.enter(|m| {
        m.scope(|m, s| {
            for _ in 0..35 {
                let _ = m.alloc::<Cell>(s, 0).unwrap();
            }
        });
    });
    assert_eq!(log.0.lock().unwrap().len(), 3);
    assert_eq!(h.stats().canonical_cycles, 3);
}

#[test]
fn corrupted_slots_and_headers_stay_inside_the_cage() {
    // H-2: arbitrary words in object slots and headers give wrong answers,
    // never an access outside the cage (or a panic).
    let mut x = 0x9E37_79B9_7F4A_7C15u64;
    let mut next = || {
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        x
    };
    for stress in [Stress::None, Stress::MoveOnAlloc] {
        let (mut h, _) = heap(stress, 4096);
        h.enter(|m| {
            m.scope(|m, s| {
                let arr = m.alloc::<Array>(s, 64).unwrap();
                for round in 0..40 {
                    let c = m.alloc::<Cell>(s, 0).unwrap();
                    let a = arr.get(m);
                    for i in 0..64 {
                        let w = u32::try_from(next() & 0xFFFF_FFFF).unwrap();
                        let v = crate::Slot::from_raw(w);
                        a.set_tail(
                            m,
                            i,
                            match v.as_small() {
                                Some(k) => Value::Small(k),
                                None => Value::Ref(c.get(m).erase()),
                            },
                        );
                    }
                    // Raw corruption of a header and a slot, through the
                    // only path there is: the cage module, as a fuzzer's
                    // in-cage write API would.
                    let off = c.get(m).slot().as_reference().unwrap();
                    m.state_for_tests()
                        .cage
                        .store32(off, u32::try_from(next() & 0xFFFF_FFFF).unwrap());
                    m.state_for_tests()
                        .cage
                        .store32(off + 8, u32::try_from(next() & 0xFFFF_FFFF).unwrap());
                    if round % 7 == 0 {
                        m.collect();
                    }
                    let _ = m.digest();
                }
            });
        });
    }
}
