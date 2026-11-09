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
    heap_in(CAGE, stress, trigger)
}

/// A heap in a cage of `cage_bytes`.
fn heap_in(cage_bytes: u64, stress: Stress, trigger: u64) -> (Heap, Arc<Log>) {
    let log = Arc::new(Log::default());
    let h = Heap::new(
        Arc::new(TestMemory::default()),
        Arc::clone(&log) as Arc<dyn DecisionLog>,
        HeapConfig {
            cage_bytes,
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
        // The heap verifier is off for this test, and only for this test:
        // it assumes the cage holds what the collector and the mutator
        // put there, and this test's whole point is that it does not.
        // Under arbitrary cage contents a slot may name a dead object of
        // this very heap, which H-2 allows (wrong answers inside the
        // cage) and the verifier reports as a collector bug. The two
        // check different properties; see the `verify` module. Every
        // assertion of this test is unchanged.
        h.set_verify(false);
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

/// A root made in an outer scope from inside an inner one lives as long
/// as the outer scope (JS spec 6.5, rule 3), not the inner one: the inner
/// exit used to drop it, and the next root took its place.
#[test]
fn an_outer_root_made_in_an_inner_scope_outlives_the_inner_scope() {
    for stress in [Stress::None, Stress::MoveOnAlloc] {
        let (mut h, _) = heap(stress, 1 << 40);
        h.enter(|m| {
            m.scope(|m, outer| {
                let (r, v) = m.scope(|m, inner| {
                    let _short = m.alloc::<Cell>(inner, 0).unwrap();
                    let r = m.alloc::<Cell>(outer, 0).unwrap();
                    r.get(m).set_raw(m, Cell::N, 42);
                    let v = outer.root_value(m, Value::Small(5));
                    (r, v)
                });
                assert_eq!(r.get(m).raw(m, Cell::N), 42);
                let other = m.alloc::<Cell>(outer, 0).unwrap();
                other.get(m).set_raw(m, Cell::N, 7);
                m.safepoint();
                assert_eq!(r.get(m).raw(m, Cell::N), 42);
                assert_eq!(v.get_value(m), Value::Small(5));
                assert_eq!(other.get(m).raw(m, Cell::N), 7);
            });
        });
    }
}

/// A moving cycle with no room left for the copies keeps the blocks it
/// could not empty: every live object reads back what it held. They used
/// to be freed and poisoned with the evacuated blocks. The cage is the
/// smallest that holds a heap (two chunks), so Miri fills it quickly.
#[test]
fn an_evacuation_that_runs_out_of_room_keeps_its_live_objects() {
    let (mut h, _) = heap_in(2 * crate::cage::CHUNK, Stress::MoveAtSafepoint, 1 << 40);
    h.enter(|m| {
        m.scope(|m, s| {
            let mut kept = Vec::new();
            for i in 0.. {
                let Ok(a) = m.alloc::<Array>(s, 1000) else {
                    break;
                };
                a.get(m).set_tail(m, 0, Value::Small(i));
                kept.push((i, a));
            }
            assert!(kept.len() > 20, "the cage filled: {}", kept.len());
            m.safepoint();
            for (i, a) in &kept {
                assert_eq!(a.get(m).tail(m, 0), Some(Value::Small(*i)), "array {i}");
            }
            // The heap still works: what was freed can be allocated again.
            drop(kept);
        });
        m.scope(|m, s| {
            m.safepoint();
            assert!(m.alloc::<Array>(s, 1000).is_ok());
        });
    });
}

// --- the heap verifier (JS-M0 plan Phase 2 item 4) --------------------------
//
// Every test above now runs with the verifier on after every cycle, which
// is the real coverage. These tests check the verifier itself: that it
// passes on a well-formed heap in all three collector modes, and that it
// catches each invariant it claims to, planted one at a time. A verifier
// that cannot fail is worth nothing, so each violation gets a test that
// produces it.

/// The offset a rooted reference currently lives at.
fn offset_of<'h, T: crate::Referent>(m: &crate::Mutator<'h>, r: &crate::Root<'h, '_, T>) -> u64 {
    r.get(m).slot().as_reference().expect("a reference")
}

#[test]
fn the_verifier_passes_on_a_well_formed_heap_in_every_collector_mode() {
    for stress in [Stress::None, Stress::MoveOnAlloc, Stress::MoveAtSafepoint] {
        let (mut h, _) = heap(stress, 1 << 40);
        let audit = h.enter(|m| {
            m.scope(|m, s| {
                // A graph with every slot kind: strong fields, a weak
                // field, a slot tail and a byte tail.
                let head = m.alloc::<Cell>(s, 0).unwrap();
                let arr = m.alloc::<Array>(s, 8).unwrap();
                let bytes = m.alloc::<Bytes>(s, 24).unwrap();
                let watcher = m.alloc::<WeakCell>(s, 0).unwrap();
                for i in 0..24u32 {
                    let c = m.alloc::<Cell>(s, 0).unwrap();
                    let (hd, cg) = (head.get(m), c.get(m));
                    cg.set(m, Cell::NEXT, hd.get(m, Cell::NEXT));
                    hd.set(m, Cell::NEXT, Some(cg));
                    if i < 8 {
                        arr.get(m).set_tail(m, i, Value::Ref(cg.erase()));
                    }
                    if i == 0 {
                        watcher
                            .get(m)
                            .set_weak(m, WeakCell::TARGET, Some(cg.erase()));
                    }
                    bytes.get(m).set_byte(m, i, u8::try_from(i).unwrap());
                    m.safepoint();
                }
                m.collect();
                m.verify().expect("a well-formed heap verifies")
            })
        });
        // The graph is 1 head + 24 cells + the array + the bytes + the
        // watcher. Every mode must see the same live set, whatever it
        // moved.
        assert_eq!(audit.objects, 28, "{stress:?}");
        assert_eq!(audit.weak_slots, 1, "{stress:?}");
        assert_eq!(audit.foreign, 0, "{stress:?}");
        assert!(audit.bytes > 0 && audit.slots > 0, "{stress:?}");
    }
}

#[test]
fn the_verifier_catches_an_evacuation_that_left_a_stale_reference() {
    // The bug class the verifier exists for (heap design H-7: "every slot
    // that reached it is updated"). It is planted the way a collector bug
    // would leave it: a live object's slot keeps the address the target
    // had *before* the cycle moved it. H-2 means that address still
    // decodes to an 8-aligned offset inside the cage and reads back
    // whatever lives there now, so nothing else notices.
    let (mut h, _) = heap(Stress::MoveAtSafepoint, 1 << 40);
    h.enter(|m| {
        m.scope(|m, s| {
            let a = m.alloc::<Cell>(s, 0).unwrap();
            let b = m.alloc::<Cell>(s, 0).unwrap();
            a.get(m).set(m, Cell::NEXT, Some(b.get(m)));
            let before = offset_of(m, &b);

            // A move-everything cycle; `safepoint` runs one and the
            // verifier passes after it.
            m.safepoint();
            let after = offset_of(m, &b);
            assert_ne!(before, after, "the cycle was supposed to move b");
            assert!(m.verify().is_ok(), "the heap is sound before the plant");

            // Plant the bug: a's NEXT keeps b's old address.
            let a_off = offset_of(m, &a);
            let slot = crate::Slot::reference(before).unwrap();
            let at = a_off + crate::object::Layout::slot_offset(Cell::NEXT.index());
            m.state_for_tests().cage.store32(at, slot.raw());

            match m.verify() {
                Err(crate::Violation::Dangling {
                    holder,
                    slot,
                    target,
                }) => {
                    assert_eq!(holder, a_off);
                    assert_eq!(slot, at);
                    assert_eq!(target, before, "it names b's pre-cycle address");
                }
                other => panic!("the verifier missed a stale reference: {other:?}"),
            }
        });
    });
}

#[test]
fn the_verifier_catches_a_reference_to_an_object_the_cycle_did_not_mark() {
    // A reference into a block that survived (its other lines are live)
    // but to an object the cycle swept. The block is still a small-object
    // block, so this is not `Dangling`; it is an unmarked object, which
    // is what a collector that lost a mark bit would leave.
    let (mut h, _) = heap(Stress::None, 1 << 40);
    h.enter(|m| {
        m.scope(|m, s| {
            let keep = m.alloc::<Cell>(s, 0).unwrap();
            let doomed_off = m.scope(|m, inner| {
                let d = m.alloc::<Cell>(inner, 0).unwrap();
                offset_of(m, &d)
            });
            m.collect();
            // `keep` is still live, so its block was not freed whole.
            let keep_off = offset_of(m, &keep);
            assert!(m.verify().is_ok());

            let at = keep_off + crate::object::Layout::slot_offset(Cell::NEXT.index());
            let slot = crate::Slot::reference(doomed_off).unwrap();
            m.state_for_tests().cage.store32(at, slot.raw());
            match m.verify() {
                Err(crate::Violation::Unmarked { at }) => assert_eq!(at, doomed_off),
                Err(crate::Violation::Dangling { target, .. }) => {
                    // Acceptable too: if the sweep freed the whole block,
                    // the same plant is a dangling reference instead.
                    assert_eq!(target, doomed_off);
                }
                other => panic!("the verifier missed a swept object: {other:?}"),
            }
        });
    });
}

#[test]
fn the_verifier_catches_a_weak_slot_left_naming_a_dead_object() {
    // H-10: a canonical cycle clears a weak slot whose target died. A
    // collector that skipped the clearing would leave this.
    let (mut h, _) = heap(Stress::None, 1 << 40);
    h.enter(|m| {
        m.scope(|m, s| {
            let w = m.alloc::<WeakCell>(s, 0).unwrap();
            let dead_off = m.scope(|m, inner| {
                let t = m.alloc::<Cell>(inner, 0).unwrap();
                w.get(m)
                    .set_weak(m, WeakCell::TARGET, Some(t.get(m).erase()));
                offset_of(m, &t)
            });
            m.collect();
            assert!(
                w.get(m).get_weak(m, WeakCell::TARGET).is_none(),
                "the cycle should have cleared it"
            );
            assert!(m.verify().is_ok());

            let w_off = offset_of(m, &w);
            let at = w_off + crate::object::Layout::slot_offset(WeakCell::TARGET.index());
            m.state_for_tests()
                .cage
                .store32(at, crate::Slot::reference(dead_off).unwrap().raw());
            match m.verify() {
                Err(crate::Violation::DeadWeak {
                    holder,
                    slot,
                    target,
                }) => {
                    assert_eq!((holder, slot, target), (w_off, at, dead_off));
                }
                other => panic!("the verifier missed an uncleared weak slot: {other:?}"),
            }
        });
    });
}

#[test]
fn the_verifier_counts_a_cross_heap_slot_as_foreign_rather_than_a_violation() {
    // H-12: a slot naming another agent heap's chunk is never followed,
    // so it is not a collector bug. It is counted, because safe code
    // cannot create one.
    let mem = Arc::new(TestMemory::default());
    let cage = ProcessCage::new(mem, CAGE).unwrap();
    let config = HeapConfig {
        cage_bytes: 0,
        trigger: 1 << 40,
        stress: Stress::None,
        agent: 1,
    };
    let mut a = Heap::in_cage(&cage, Arc::new(Log::default()), config);
    let mut b = Heap::in_cage(
        &cage,
        Arc::new(Log::default()),
        HeapConfig { agent: 2, ..config },
    );
    b.enter(|mb| {
        mb.scope(|mb, sb| {
            let theirs = mb.alloc::<Cell>(sb, 0).unwrap();
            let slot = theirs.get(mb).slot();
            a.enter(|ma| {
                ma.scope(|ma, sa| {
                    let mine = ma.alloc::<Cell>(sa, 0).unwrap();
                    let off = offset_of(ma, &mine);
                    let at = off + crate::object::Layout::slot_offset(Cell::NEXT.index());
                    ma.state_for_tests().cage.store32(at, slot.raw());
                    ma.collect();
                    let audit = ma.verify().expect("a foreign slot is not a violation");
                    assert_eq!(audit.foreign, 1);
                });
            });
        });
    });
}

#[test]
fn verifying_after_an_allocation_does_not_report_a_false_unmarked() {
    // Allocation marks an object's lines but not its mark bit, so between
    // one cycle and the next a live object is legitimately unmarked. The
    // first version of the verifier reported that as a violation, which
    // made the public entry points unusable anywhere but immediately
    // after a cycle. The mark checks are now skipped in that state, and
    // the audit says so; every other check still runs.
    let (mut h, _) = heap(Stress::None, 1 << 40);
    h.enter(|m| {
        m.scope(|m, s| {
            let head = m.alloc::<Cell>(s, 0).unwrap();
            m.collect();
            let after_cycle = m.verify().expect("sound right after a cycle");
            assert!(after_cycle.marks_checked);

            // Allocate without collecting: the new object has no mark bit.
            let fresh = m.alloc::<Cell>(s, 0).unwrap();
            head.get(m).set(m, Cell::NEXT, Some(fresh.get(m)));
            let after_alloc = m.verify().expect("an unmarked fresh object is not a bug");
            assert!(!after_alloc.marks_checked);
            assert_eq!(after_alloc.objects, 2);

            // The checks that do not depend on marks still bite: a slot
            // naming freed memory is caught whether or not marks are
            // being checked.
            let gone = m.scope(|m, inner| {
                let d = m.alloc::<Array>(inner, 4000).unwrap();
                offset_of(m, &d)
            });
            m.collect();
            let fresh2 = m.alloc::<Cell>(s, 0).unwrap();
            let at = offset_of(m, &fresh2)
                + crate::object::Layout::slot_offset(Cell::NEXT.index());
            m.state_for_tests()
                .cage
                .store32(at, crate::Slot::reference(gone).unwrap().raw());
            assert!(
                matches!(m.verify(), Err(crate::Violation::Dangling { target, .. }) if target == gone),
                "a dangling slot must be caught with the mark checks off too"
            );
        });
    });
}
