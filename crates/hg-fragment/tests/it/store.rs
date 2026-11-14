//! The store, the epoch rule, and the id-free canonical tree.

use hg_fragment::{BoxId, BreakTokenId, Error, Fragment, FragmentId, Store, canonical};
use hg_types::geometry::{LayoutUnit, LogicalSize};

fn size(inline: i32, block: i32) -> LogicalSize {
    LogicalSize::new(LayoutUnit::from_px(inline), LayoutUnit::from_px(block))
}

// Clippy counts a `#[test]` body as test code but not a helper beside
// one; the panic here is a test failure like any other.
#[expect(
    clippy::panic,
    reason = "a fixture that will not build is a test failure"
)]
fn leaf(store: &mut Store, id: u32) -> FragmentId {
    match store.push(Fragment::box_fragment(BoxId(id), size(10, 10)), &[]) {
        Ok(f) => f,
        Err(e) => panic!("a leaf pushes: {e:?}"),
    }
}

#[test]
fn f01_a_tree_is_read_back_in_pre_order() {
    let mut s = Store::new();
    let a = leaf(&mut s, 1);
    let b = leaf(&mut s, 2);
    let Ok(root) = s.push(Fragment::box_fragment(BoxId(0), size(100, 100)), &[a, b]) else {
        panic!("the root pushes");
    };
    assert_eq!(s.live_children(root), [a, b]);
    assert_eq!(s.pre_order(root), [root, a, b]);
    assert_eq!(s.live(), 3);
    let Ok(f) = s.get(root) else {
        panic!("the root is there");
    };
    assert_eq!(f.child_count(), 2);
    assert_eq!(f.box_id, BoxId(0));
}

#[test]
fn f02_a_tree_cannot_be_built_over_a_reclaimed_slot() {
    let mut s = Store::new();
    let a = leaf(&mut s, 1);
    assert_eq!(s.reclaim(a), Ok(()));
    // `Reclaimed`, not `NoSuchFragment`: the slot exists and is free,
    // which is a different thing from an index the store never had.
    assert_eq!(
        s.push(Fragment::box_fragment(BoxId(0), size(1, 1)), &[a]),
        Err(Error::Reclaimed)
    );
}

#[test]
fn f03_a_handle_goes_stale_when_its_slot_is_reused() {
    // G45: "any fragment handle held beyond a pass is an (id, epoch)
    // pair checked on use". Without the epoch the next fragment to
    // take the slot would answer to the old handle.
    let mut s = Store::new();
    let a = leaf(&mut s, 1);
    let Ok(held) = s.handle(a) else {
        panic!("a live fragment has a handle");
    };
    assert!(s.resolve(held).is_ok());

    assert_eq!(s.reclaim(a), Ok(()));
    assert_eq!(s.resolve(held), Err(Error::StaleHandle));

    // The slot is reused by the next push, and the handle is still
    // stale — this is the case the epoch exists for.
    let b = leaf(&mut s, 2);
    assert_eq!(b, a, "the free list reuses the slot");
    assert_eq!(s.resolve(held), Err(Error::StaleHandle));
    let Ok(fresh) = s.handle(b) else {
        panic!("the new occupant has a handle");
    };
    assert_ne!(fresh.epoch(), held.epoch());
    let Ok(f) = s.resolve(fresh) else {
        panic!("the fresh handle resolves");
    };
    assert_eq!(f.box_id, BoxId(2));
}

#[test]
fn f04_reclaiming_one_fragment_does_not_cut_a_shared_subtree() {
    // An incremental pass reuses subtrees, so a child can be reachable
    // from two parents. Reclaiming a parent must not take the child.
    let mut s = Store::new();
    let shared = leaf(&mut s, 9);
    let Ok(one) = s.push(Fragment::box_fragment(BoxId(1), size(10, 10)), &[shared]) else {
        panic!("it pushes");
    };
    let Ok(two) = s.push(Fragment::box_fragment(BoxId(2), size(10, 10)), &[shared]) else {
        panic!("it pushes");
    };
    assert_eq!(s.reclaim(one), Ok(()));
    assert!(s.get(shared).is_ok(), "the shared child is still live");
    assert_eq!(s.live_children(two), [shared]);
}

#[test]
fn f05_reclaiming_a_tree_takes_each_slot_once() {
    let mut s = Store::new();
    let a = leaf(&mut s, 1);
    let b = leaf(&mut s, 2);
    let Ok(mid) = s.push(Fragment::box_fragment(BoxId(3), size(10, 10)), &[a, b]) else {
        panic!("it pushes");
    };
    let Ok(root) = s.push(Fragment::box_fragment(BoxId(0), size(10, 10)), &[mid, a]) else {
        panic!("it pushes");
    };
    // `a` is reachable twice; the walk must not reclaim it twice.
    assert_eq!(s.reclaim_tree(root), Ok(4));
    assert_eq!(s.live(), 0);
    assert_eq!(s.reclaim_tree(root), Err(Error::NoSuchFragment));
}

#[test]
fn f06_the_footprint_counts_live_records_and_not_churn() {
    // FP-24: "Footprint counts live records only, so canonical bytes
    // do not depend on history."
    let mut s = Store::new();
    for i in 0..10 {
        let f = leaf(&mut s, i);
        assert_eq!(s.reclaim(f), Ok(()));
    }
    assert_eq!(s.live(), 0);
    assert_eq!(s.capacity(), 1, "one slot, reused ten times");
    let _ = leaf(&mut s, 99);
    assert_eq!(s.live(), 1);
}

#[test]
fn f07_the_canonical_tree_holds_no_id_and_so_two_histories_agree() {
    // LY-D18: ids depend on history, so they are never compared or
    // hashed; the canonical tree is. This is the test that would fail
    // if an id leaked into it.
    //
    // Two stores lay out the same page. The first does it from
    // scratch. The second has churned, so its free list hands out
    // different indices in a different order.
    let mut fresh = Store::new();
    let a = leaf(&mut fresh, 1);
    let b = leaf(&mut fresh, 2);
    let Ok(root) = fresh.push(Fragment::box_fragment(BoxId(0), size(100, 50)), &[a, b]) else {
        panic!("it pushes");
    };

    // The second store has churned: three slots were taken and two
    // freed out of order, so the free list hands them back in the
    // reverse of the order they were made in.
    let mut churned = Store::new();
    let junk: Vec<_> = (0..3).map(|i| leaf(&mut churned, 100 + i)).collect();
    for i in [0_usize, 2] {
        let Some(f) = junk.get(i) else {
            panic!("three were made");
        };
        assert_eq!(churned.reclaim(*f), Ok(()));
    }
    let ca = leaf(&mut churned, 1);
    let cb = leaf(&mut churned, 2);
    let Ok(croot) = churned.push(Fragment::box_fragment(BoxId(0), size(100, 50)), &[ca, cb]) else {
        panic!("it pushes");
    };

    assert_ne!(
        (root.index(), a.index()),
        (croot.index(), ca.index()),
        "the two histories gave different ids"
    );
    assert_eq!(
        canonical(&fresh, root),
        canonical(&churned, croot),
        "the canonical trees are equal although the ids are not"
    );
}

#[test]
fn f08_the_canonical_tree_records_that_a_fragment_broke_not_which_token() {
    // A break-token id is an id like any other, so what the canonical
    // tree carries is that there *was* one; the token's contents are
    // inlined by the caller that owns the table.
    let mut s = Store::new();
    let mut f = Fragment::box_fragment(BoxId(1), size(10, 10));
    f.break_token = Some(BreakTokenId(7));
    f.is_last = false;
    let Ok(id) = s.push(f, &[]) else {
        panic!("it pushes");
    };
    let c = canonical(&s, id);
    let Some(first) = c.first() else {
        panic!("there is one record");
    };
    assert!(first.broke);
    assert!(!first.is_last);
    assert_eq!(first.children, 0);

    // The same fragment with a different token id is canonically the
    // same, because the id is not part of the comparison.
    let mut other = Store::new();
    let mut g = Fragment::box_fragment(BoxId(1), size(10, 10));
    g.break_token = Some(BreakTokenId(4_000));
    g.is_last = false;
    let Ok(gid) = other.push(g, &[]) else {
        panic!("it pushes");
    };
    assert_eq!(canonical(&s, id), canonical(&other, gid));
}

#[test]
fn f09_a_baseline_at_zero_is_not_the_same_as_no_baseline() {
    // Alignment asks which, and conflating them is the classic bug.
    let mut s = Store::new();
    let mut f = Fragment::box_fragment(BoxId(1), size(10, 10));
    f.baselines.first = Some(LayoutUnit::ZERO);
    let Ok(with) = s.push(f, &[]) else {
        panic!("it pushes");
    };
    let without = leaf(&mut s, 2);
    let Ok(a) = s.get(with) else {
        panic!("it is there");
    };
    let Ok(b) = s.get(without) else {
        panic!("it is there");
    };
    assert_eq!(a.baselines.first, Some(LayoutUnit::ZERO));
    assert_eq!(b.baselines.first, None);
    assert_ne!(a.baselines, b.baselines);
}

#[test]
fn f10_reading_a_missing_or_freed_fragment_is_an_error_and_not_a_panic() {
    // G40 and the FreeDom finding: a panic reachable from content is a
    // denial-of-service bug, so the read API answers instead.
    // An id from another store, which this one has no slot for. The
    // id type is opaque on purpose — only a store makes one — so this
    // is how a test reaches an out-of-range index at all.
    let mut other = Store::new();
    for i in 0..5 {
        let _ = leaf(&mut other, i);
    }
    let far = leaf(&mut other, 99);
    let mut s = Store::new();
    assert_eq!(s.get(far), Err(Error::NoSuchFragment));
    let a = leaf(&mut s, 1);
    assert_eq!(s.reclaim(a), Ok(()));
    assert_eq!(s.get(a), Err(Error::Reclaimed));
    assert_eq!(s.reclaim(a), Err(Error::Reclaimed));
    assert_eq!(s.live_children(a), []);
    assert_eq!(s.pre_order(a), []);
}

#[test]
fn f11_a_reused_slot_cannot_become_its_own_ancestors_child() {
    // The cycle: a leaf is reclaimed while its parent still links to
    // it, then the slot is reused by a fragment that has that parent
    // as a child. Walked naively, the tree loops forever — reachable
    // from content, so a denial of service.
    let mut s = Store::new();
    let leaf_id = leaf(&mut s, 1);
    let Ok(parent) = s.push(Fragment::box_fragment(BoxId(2), size(10, 10)), &[leaf_id]) else {
        panic!("it pushes");
    };
    assert_eq!(s.reclaim(leaf_id), Ok(()));
    // The next push takes the freed slot — the one `parent` still
    // links to.
    let Ok(reused) = s.push(Fragment::box_fragment(BoxId(3), size(10, 10)), &[parent]) else {
        panic!("it pushes");
    };
    assert_eq!(reused, leaf_id, "the free list handed back that slot");

    // The walk terminates and never visits the stale link.
    let order = s.pre_order(reused);
    assert_eq!(order, [reused, parent], "the stale child is not followed");
    assert_eq!(canonical(&s, reused).len(), 2);
    // The parent's child link is stale, not silently retargeted.
    let Some(stale) = s.children(parent).first().copied() else {
        panic!("the parent kept its link");
    };
    assert_eq!(s.resolve(stale), Err(Error::StaleHandle));
}

#[test]
fn f12_the_child_array_is_reclaimed_by_clear_and_not_by_reclaim() {
    // Honest about what reclamation does: a fragment's slot comes
    // back, its run of child links does not, because the run is named
    // by an offset nothing else may move. The store is class (a)
    // derived data, so what reclaims the array is dropping it.
    let mut s = Store::new();
    let a = leaf(&mut s, 1);
    let Ok(root) = s.push(Fragment::box_fragment(BoxId(0), size(10, 10)), &[a]) else {
        panic!("it pushes");
    };
    assert_eq!(s.child_links(), 1);
    assert_eq!(s.reclaim_tree(root), Ok(2));
    assert_eq!(s.live(), 0);
    assert_eq!(s.child_links(), 1, "the run is still there");
    s.clear();
    assert_eq!(s.child_links(), 0);
    assert_eq!(s.capacity(), 0);
}
