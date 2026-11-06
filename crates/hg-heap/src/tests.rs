//! The heap's tests. They run on a small cage backed by an ordinary
//! allocation, so Miri can run them (it cannot map a real reservation).

use std::sync::{Arc, Mutex};

use hg_substrate::{DecisionKind, DecisionLog, Position};

use crate::cage::test_memory::TestMemory;

use crate::{Any, Heap, HeapConfig, Object, ProcessCage, Stress, Value};

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
fn only_charged_units_trigger_a_canonical_cycle() {
    // Allocation charges nothing; charge() does. A trigger of 30 units
    // with 3 units per allocating operation gives a cycle every 10.
    assert_eq!(<Cell as Object>::LAYOUT.size(0), Some(32));
    let (mut h, log) = heap(Stress::None, 30);
    h.enter(|m| {
        m.scope(|m, s| {
            for _ in 0..100 {
                let _ = m.alloc::<Cell>(s, 0).unwrap();
            }
            assert_eq!(m.stats().canonical_cycles, 0);
            for _ in 0..35 {
                m.charge(3);
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
        let (mut h, _) = heap(stress, 64);
        h.enter(|m| {
            m.scope(|m, s| {
                let arr = m.alloc::<Array>(s, 64).unwrap();
                for round in 0..40 {
                    m.charge(4);
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

#[test]
fn value_roots_hold_small_integers_and_follow_moved_references() {
    for stress in [Stress::None, Stress::MoveOnAlloc] {
        let (mut h, _) = heap(stress, 1 << 40);
        h.enter(|m| {
            m.scope(|m, s| {
                let small = s.root_value(m, Value::Small(-7));
                let cell = m.alloc::<Cell>(s, 0).unwrap();
                cell.get(m).set_raw(m, Cell::N, 99);
                let v = Value::Ref(cell.get(m).erase());
                let r = s.root_value(m, v);
                let _ = m.alloc::<Cell>(s, 0).unwrap(); // moves everything under stress
                assert_eq!(small.get_value(m), Value::Small(-7));
                let Value::Ref(g) = r.get_value(m) else {
                    panic!("a reference stays a reference")
                };
                assert_eq!(g.cast::<Cell>(m).unwrap().raw(m, Cell::N), 99);
                assert_eq!(g, cell.get(m).erase());
                r.set_value(m, Value::Small(3));
                assert_eq!(r.get_value(m), Value::Small(3));
            });
        });
    }
}

#[test]
fn canonical_cycles_report_the_holders_whose_weak_slots_they_cleared() {
    for stress in [Stress::None, Stress::MoveOnAlloc, Stress::MoveAtSafepoint] {
        let (mut h, _) = heap(stress, 1 << 40);
        h.enter(|m| {
            m.scope(|m, s| {
                let a = m.alloc::<WeakCell>(s, 0).unwrap();
                let b = m.alloc::<WeakCell>(s, 0).unwrap();
                let c = m.alloc::<WeakCell>(s, 0).unwrap();
                let alive = m.alloc::<Cell>(s, 0).unwrap();
                m.scope(|m, s2| {
                    let d1 = m.alloc::<Cell>(s2, 0).unwrap();
                    let d2 = m.alloc::<Cell>(s2, 0).unwrap();
                    a.get(m)
                        .set_weak(m, WeakCell::TARGET, Some(d1.get(m).erase()));
                    b.get(m)
                        .set_weak(m, WeakCell::TARGET, Some(alive.get(m).erase()));
                    c.get(m)
                        .set_weak(m, WeakCell::TARGET, Some(d2.get(m).erase()));
                });
                // Nothing is reported before a canonical cycle, nor by a
                // stress cycle.
                m.safepoint();
                assert!(m.take_cleared(s).is_empty());
                m.collect();
                // The report survives a moving cycle before it is taken.
                let _ = m.alloc::<Cell>(s, 0).unwrap();
                m.safepoint();
                let got = m.take_cleared(s);
                assert_eq!(got.len(), 2, "{stress:?}");
                assert_eq!(got[0].get(m), a.get(m).erase());
                assert_eq!(got[1].get(m), c.get(m).erase());
                assert!(m.take_cleared(s).is_empty());
                m.collect();
                assert!(m.take_cleared(s).is_empty());
            });
        });
    }
}

/// JS-D17: several agent heaps share the process's cage, each owning its
/// own chunks. Interleaved work in two heaps gives each the digest it has
/// alone; a slot pointing into the other heap's chunks is never followed;
/// a dropped heap's chunks are reused.
#[test]
fn agent_heaps_share_one_cage_and_own_their_chunks() {
    let build = |m: &mut crate::Mutator<'_>, n: u32, seed: i32| -> u64 {
        m.scope(|m, s| {
            let head = m.alloc::<Cell>(s, 0).unwrap();
            for i in 0..n {
                let c = m.alloc::<Cell>(s, 0).unwrap();
                let (hd, cg) = (head.get(m), c.get(m));
                cg.set(m, Cell::NEXT, hd.get(m, Cell::NEXT));
                cg.set_value(
                    m,
                    Cell::VALUE,
                    Value::Small(seed + i32::try_from(i).unwrap()),
                );
                hd.set(m, Cell::NEXT, Some(cg));
            }
            m.collect();
            m.digest()
        })
    };
    let alone = |seed| {
        let (mut h, _) = heap(Stress::MoveOnAlloc, 1 << 40);
        h.enter(|m| build(m, 40, seed))
    };
    let (da, db) = (alone(0), alone(1000));

    let mem = Arc::new(TestMemory::default());
    let cage = ProcessCage::new(mem, CAGE).unwrap();
    let config = HeapConfig {
        cage_bytes: 0,
        trigger: 1 << 40,
        stress: Stress::MoveOnAlloc,
        agent: 1,
    };
    let mut a = Heap::in_cage(&cage, Arc::new(Log::default()), config);
    let mut b = Heap::in_cage(
        &cage,
        Arc::new(Log::default()),
        HeapConfig { agent: 2, ..config },
    );
    // Interleave: a, then b, then a again (each rebuild moves everything).
    let a1 = a.enter(|m| build(m, 40, 0));
    let b1 = b.enter(|m| build(m, 40, 1000));
    let a2 = a.enter(|m| build(m, 40, 0));
    assert_eq!((a1, a2, b1), (da, da, db));

    // A slot of heap a planted to point at an object of heap b is not
    // followed: a's collection neither marks nor moves b's object, and a's
    // digest counts it as a reference to nothing.
    b.enter(|mb| {
        mb.scope(|mb, sb| {
            let theirs = mb.alloc::<Cell>(sb, 0).unwrap();
            theirs.get(mb).set_value(mb, Cell::VALUE, Value::Small(77));
            let slot = theirs.get(mb).slot();
            a.enter(|ma| {
                ma.scope(|ma, sa| {
                    let mine = ma.alloc::<Cell>(sa, 0).unwrap();
                    let off = mine.get(ma).slot().as_reference().unwrap();
                    ma.state_for_tests().cage.store32(off + 8, slot.raw());
                    ma.collect();
                    let off = mine.get(ma).slot().as_reference().unwrap();
                    assert_eq!(ma.state_for_tests().cage.load32(off + 8), slot.raw());
                });
            });
            assert_eq!(theirs.get(mb).slot(), slot);
            assert_eq!(theirs.get(mb).get_value(mb, Cell::VALUE), Value::Small(77));
        });
    });

    let before = b.stats().blocks;
    drop(b);
    let c = Heap::in_cage(
        &cage,
        Arc::new(Log::default()),
        HeapConfig { agent: 3, ..config },
    );
    let mut c = c;
    let dc = c.enter(|m| build(m, 40, 1000));
    assert_eq!(dc, db);
    assert!(before > 0);
}
