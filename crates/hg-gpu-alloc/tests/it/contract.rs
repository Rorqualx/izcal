//! The interface, decision by decision.

use hg_gpu_alloc::budget::{Budget, Class, Decision, admit, default_class};
use hg_gpu_alloc::caps;
use hg_gpu_alloc::ledger::{
    AllocationId, CompositorScope, ContainerId, Kind, Ledger, LedgerError, PartitionId, Release,
    Scope, SiteId, TabId, Tag,
};
use hg_gpu_alloc::residency::{Allocation, Contents, Residency, ResidencyError};
use hg_gpu_hal::device::Epoch;

const E0: Epoch = Epoch(0);

fn key_scope() -> hg_gpu_alloc::ledger::KeyScope {
    hg_gpu_alloc::ledger::KeyScope::new(SiteId(7), ContainerId(1), PartitionId(3))
}

fn id(index: u32) -> AllocationId {
    AllocationId { epoch: E0, index }
}

/// Decision 1. A content GPU process is one renderer key, so the
/// interface gives it no way to name another site — and if a tag
/// naming one arrives by some other route, the ledger refuses it.
///
/// The half that cannot be tested at runtime is the half that matters
/// most: `KeyScope` has no method taking a `SiteId`, so the mistake
/// does not compile. This holds the other half.
#[test]
fn ga01_a_content_gpu_process_cannot_charge_another_site() {
    let scope = key_scope();
    let mut led = Ledger::new(Scope::Key(scope), E0);

    // Its own key: charged.
    let mine = scope.tag(Kind::Tile, TabId(1));
    assert_eq!(mine.site, SiteId(7), "the scope filled the site in");
    assert_eq!(led.charge(id(1), mine, 4_096), Ok(()));
    assert_eq!(led.charged(), 4_096);

    // A tag naming a different site, which this process could not
    // have made. Planted here exactly because the type stops the
    // honest route.
    let theirs = Tag {
        site: SiteId(8),
        ..mine
    };
    assert_eq!(
        led.charge(id(2), theirs, 4_096),
        Err(LedgerError::ForeignSite)
    );
    assert_eq!(led.charged(), 4_096, "and nothing was charged");

    // The container and the partition are the key too, not just the
    // site: a tag that differs in either is as foreign.
    for wrong in [
        Tag {
            container: ContainerId(2),
            ..mine
        },
        Tag {
            partition: PartitionId(4),
            ..mine
        },
    ] {
        assert_eq!(
            led.charge(id(3), wrong, 1),
            Err(LedgerError::ForeignSite),
            "container and partition are part of the key"
        );
    }
}

/// Decision 1, the other scope. The compositor GPU service holds
/// every site's pixels, so for it the site *is* a parameter, and its
/// per-site totals are what attribution reads.
#[test]
fn ga02_the_compositor_charges_the_site_whose_content_it_holds() {
    let scope = CompositorScope;
    let mut led = Ledger::new(Scope::Compositor(scope), E0);

    for (index, site) in [(1_u32, SiteId(7)), (2, SiteId(8)), (3, SiteId(7))] {
        let tag = scope.tag(site, ContainerId(1), PartitionId(3), Kind::Tile, TabId(1));
        assert_eq!(led.charge(id(index), tag, 1_000), Ok(()));
    }

    assert_eq!(led.charged(), 3_000);
    assert_eq!(led.charged_to(SiteId(7)), 2_000, "two tiles");
    assert_eq!(led.charged_to(SiteId(8)), 1_000);
    assert_eq!(led.charged_to(SiteId(9)), 0, "a site it holds nothing for");
}

/// Decision 2. Over budget is a state. A reserved allocation is
/// admitted inside it, because the frame it presents is what frees
/// the memory; working and discretionary ones are not.
#[test]
fn ga03_over_budget_still_admits_what_presentation_needs() {
    let budget = Budget {
        bytes: 1_000,
        usage: 1_500,
    };
    assert!(budget.is_over());
    assert_eq!(budget.headroom(), 0, "saturating, never wrapping");

    // Already over, by the ledger's own count.
    let charged = 1_400;
    assert_eq!(
        admit(Class::Reserved, charged, 100, &budget, 200),
        Decision::Admit,
        "refusing this is the deadlock"
    );
    assert_eq!(
        admit(Class::Working, charged, 100, &budget, 200),
        Decision::Refuse
    );
    assert_eq!(
        admit(Class::Discretionary, charged, 100, &budget, 200),
        Decision::Refuse
    );
}

/// Decision 2, continued. The reserved cap is a parameter because
/// without it discretionary work fills the budget to the last byte
/// and the swapchain that must then be admitted puts the process
/// over with no way back.
#[test]
fn ga04_discretionary_work_is_held_off_the_reserved_room() {
    let budget = Budget {
        bytes: 1_000,
        usage: 0,
    };
    let reserved_cap = 300;

    // Working may use the whole budget.
    assert_eq!(
        admit(Class::Working, 0, 1_000, &budget, reserved_cap),
        Decision::Admit
    );
    // Discretionary may not: 1,000 − 300 is its ceiling.
    assert_eq!(
        admit(Class::Discretionary, 0, 700, &budget, reserved_cap),
        Decision::Admit
    );
    assert_eq!(
        admit(Class::Discretionary, 0, 701, &budget, reserved_cap),
        Decision::Refuse,
        "one byte into the reserved room"
    );

    // And with no cap at all the two classes agree, which is the
    // degenerate interface this parameter exists to avoid.
    assert_eq!(
        admit(Class::Discretionary, 0, 1_000, &budget, 0),
        admit(Class::Working, 0, 1_000, &budget, 0)
    );
}

/// Decision 2, the arithmetic. A total that cannot be expressed is
/// not a total that fits. A wrapped one reads as a nearly empty
/// device, which is the worst possible answer.
///
/// **Reserved is refused here too**, and that is not a contradiction
/// of `ga03`. The reason reserved work is admitted over budget is
/// the presentation deadlock: refusing the swapchain refuses the
/// frame whose completion frees the tiles. An unrepresentable total
/// is not that condition — it is 18 exabytes already charged, where
/// admitting cannot help anyone — so the sum is checked before the
/// class, and every class loses.
#[test]
fn ga05_a_total_that_would_overflow_is_refused_not_wrapped() {
    let budget = Budget {
        bytes: u64::MAX,
        usage: 0,
    };
    for class in Class::ALL {
        assert_eq!(
            admit(class, u64::MAX, 1, &budget, 0),
            Decision::Refuse,
            "{class:?}: an unrepresentable total is not a budget condition"
        );
    }
    // And one byte less is representable, so the budget decides again.
    assert_eq!(
        admit(Class::Working, u64::MAX - 1, 1, &budget, 0),
        Decision::Admit
    );
}

/// Decision 3. A tile is one kind and two classes. If the class were
/// derived from the kind, the prepaint margin could not shrink first.
#[test]
fn ga06_the_class_is_not_a_function_of_the_kind() {
    let scope = key_scope();
    let tag = scope.tag(Kind::Tile, TabId(1));

    // The same kind, carried at two classes.
    let viewport = hg_gpu_alloc::budget::Request {
        tag,
        class: Class::Working,
        bytes: 64 * 1_024,
    };
    let prepaint = hg_gpu_alloc::budget::Request {
        tag,
        class: Class::Discretionary,
        bytes: 64 * 1_024,
    };
    assert_eq!(viewport.tag.kind, prepaint.tag.kind);
    assert_ne!(viewport.class, prepaint.class);

    // Under pressure they part company, and the prepaint one goes.
    let budget = Budget {
        bytes: 128 * 1_024,
        usage: 0,
    };
    let reserved_cap = 96 * 1_024;
    assert_eq!(
        admit(viewport.class, 0, viewport.bytes, &budget, reserved_cap),
        Decision::Admit
    );
    assert_eq!(
        admit(prepaint.class, 0, prepaint.bytes, &budget, reserved_cap),
        Decision::Refuse
    );

    // The default exists and is only a default: a tile's is Working,
    // which is the wrong answer for the prepaint one above.
    assert_eq!(default_class(Kind::Tile), Class::Working);
    assert_eq!(default_class(Kind::Swapchain), Class::Reserved);
    assert_eq!(default_class(Kind::Staging), Class::Discretionary);
}

/// Decision 3, the ordering. Three back ends under pressure must drop
/// the same things, or one page behaves differently on Metal and on
/// Vulkan for reasons no test would catch.
#[test]
fn ga07_reclamation_has_one_order_and_reserved_is_never_in_it() {
    let mut ranked = Class::ALL;
    ranked.sort_by_key(|c| c.reclaim_rank());
    assert_eq!(
        ranked,
        [Class::Discretionary, Class::Working, Class::Reserved]
    );
    assert!(Class::Discretionary.is_reclaimable());
    assert!(Class::Working.is_reclaimable());
    assert!(
        !Class::Reserved.is_reclaimable(),
        "reclaiming the swapchain to make room for the swapchain"
    );
}

/// Decision 4. The contract takes the weakest of the three back ends'
/// guarantees: the contents may be gone, and residency alone never
/// makes the allocation sampleable again.
#[test]
fn ga08_a_recovered_allocation_is_a_destination_until_it_is_written() {
    let scope = key_scope();
    let mut a = Allocation::new(id(1), scope.tag(Kind::Tile, TabId(1)), 4_096);

    // New memory is not Izcal's either: GR 9's lazy-clearing class.
    assert!(!a.can_sample(), "nothing has written it yet");
    assert!(a.can_write());
    assert_eq!(a.mark_written(), Ok(()));
    assert!(a.can_sample());

    // Hidden tab: volatile first.
    assert_eq!(a.make_volatile(), Ok(()));
    assert_eq!(a.residency(), Residency::Volatile);
    assert!(!a.can_sample(), "the system may already have taken it");
    assert!(!a.can_write(), "and a write could land in storage it took");

    // Back, with the pixels gone — Vulkan's answer always, and
    // Metal's whenever the system reclaimed the storage.
    assert_eq!(a.make_resident(false), Ok(Contents::Lost));
    assert!(!a.can_sample(), "this is the uninitialized-read bug");
    assert!(a.can_write());
    assert_eq!(a.mark_written(), Ok(()));
    assert!(a.can_sample(), "and only now");
}

/// Decision 4, the stronger back end. D3D12's `Evict` pages out
/// rather than discards, so it can report the contents intact — and
/// the interface is the same shape either way, which is the point.
#[test]
fn ga09_a_back_end_that_kept_the_contents_says_so_and_nothing_changes() {
    let scope = key_scope();
    let mut a = Allocation::new(id(1), scope.tag(Kind::ImageTexture, TabId(1)), 4_096);
    assert_eq!(a.mark_written(), Ok(()));
    assert_eq!(a.make_volatile(), Ok(()));
    assert_eq!(a.make_resident(true), Ok(Contents::Intact));
    assert!(a.can_sample(), "no re-raster needed on this back end");
}

/// Decision 4, the transitions that must not be legal. A released
/// allocation does not come back: its memory is not reserved for it
/// and may be anything now.
#[test]
fn ga10_a_released_allocation_never_comes_back() {
    let scope = key_scope();
    let mut a = Allocation::new(id(1), scope.tag(Kind::Canvas, TabId(1)), 4_096);
    a.release();
    assert_eq!(a.residency(), Residency::Released);
    assert!(!a.can_sample());
    assert!(!a.can_write());
    assert_eq!(a.make_resident(true), Err(ResidencyError::Released));
    assert_eq!(a.make_volatile(), Err(ResidencyError::Released));
    assert_eq!(a.mark_written(), Err(ResidencyError::Released));

    // And a state asked for twice is a caller that lost track, not a
    // no-op worth hiding.
    let mut b = Allocation::new(id(2), scope.tag(Kind::Canvas, TabId(1)), 4_096);
    assert_eq!(b.make_volatile(), Ok(()));
    assert_eq!(b.make_volatile(), Err(ResidencyError::AlreadyThere));
    assert_eq!(b.make_resident(true), Ok(Contents::Intact));
    assert_eq!(b.make_resident(true), Err(ResidencyError::AlreadyThere));
}

/// Decision 5. G71: granted once, never re-granted. The type stops
/// the duplicate; this stops the second grant.
#[test]
fn ga11_a_shared_texture_is_granted_once() {
    let scope = CompositorScope;
    let mut led = Ledger::new(Scope::Compositor(scope), E0);
    let tag = scope.tag(
        SiteId(7),
        ContainerId(1),
        PartitionId(3),
        Kind::Tile,
        TabId(1),
    );
    assert_eq!(led.charge(id(1), tag, 8_192), Ok(()));

    let grant = led.grant(id(1)).expect("the first grant");
    assert_eq!(grant.bytes(), 8_192);
    assert_eq!(grant.id(), id(1));
    assert_eq!(led.grant(id(1)), Err(LedgerError::AlreadyGranted));

    // Giving it back does not make it grantable again: G71 is about
    // the grant, not the holding.
    assert_eq!(led.unpin(grant), Ok(()));
    assert_eq!(
        led.grant(id(1)),
        Err(LedgerError::AlreadyGranted),
        "still granted once"
    );
}

/// Decision 5, the number that matters. A ledger counting only live
/// rows reports a process under budget while its pinned textures keep
/// the memory — which is the under-report that makes pinning free.
#[test]
fn ga12_a_pinned_release_stays_charged() {
    let scope = CompositorScope;
    let mut led = Ledger::new(Scope::Compositor(scope), E0);
    let tag = scope.tag(
        SiteId(7),
        ContainerId(1),
        PartitionId(3),
        Kind::Tile,
        TabId(1),
    );
    assert_eq!(led.charge(id(1), tag, 8_192), Ok(()));
    assert_eq!(led.charge(id(2), tag, 1_024), Ok(()));
    assert_eq!(led.charged(), 9_216);

    // Nothing holds id(2): releasing it frees it.
    assert_eq!(led.release(id(2)), Ok(Release::Freed));
    assert_eq!(led.charged(), 8_192);
    assert_eq!(led.pinned(), 0);

    // A holder has id(1). Releasing it does not free the memory, so
    // it does not free the charge either.
    let grant = led.grant(id(1)).expect("granted");
    assert_eq!(led.release(id(1)), Ok(Release::Pinned));
    assert_eq!(led.charged(), 8_192, "G71: the holder is charged for it");
    assert_eq!(led.pinned(), 8_192);
    assert_eq!(led.len(), 1, "the row survives its release while held");

    // When the holder gives it back, both go.
    assert_eq!(led.unpin(grant), Ok(()));
    assert_eq!(led.charged(), 0);
    assert!(led.is_empty());
}

/// Decision 5, the other order. The give-back can arrive before the
/// owner's release, and the two orders must end in the same place.
#[test]
fn ga13_the_give_back_may_precede_the_release() {
    let scope = CompositorScope;
    let mut led = Ledger::new(Scope::Compositor(scope), E0);
    let tag = scope.tag(
        SiteId(7),
        ContainerId(1),
        PartitionId(3),
        Kind::Tile,
        TabId(1),
    );
    assert_eq!(led.charge(id(1), tag, 8_192), Ok(()));

    let grant = led.grant(id(1)).expect("granted");
    assert_eq!(led.unpin(grant), Ok(()), "back before the release");
    assert_eq!(led.charged(), 8_192, "still the owner's, still live");
    assert_eq!(led.pinned(), 0, "and not pinned");

    assert_eq!(led.release(id(1)), Ok(Release::Freed), "now it frees");
    assert_eq!(led.charged(), 0);
}

/// Decision 5, the epoch. A handle held past a device loss names
/// memory that no longer exists (G45), and the reset is once per loss
/// because the HAL's `DeviceLoss::resolve` answers once per epoch.
#[test]
fn ga14_an_allocation_from_another_epoch_is_refused() {
    let scope = key_scope();
    let mut led = Ledger::new(Scope::Key(scope), E0);
    let tag = scope.tag(Kind::Tile, TabId(1));
    assert_eq!(led.charge(id(1), tag, 4_096), Ok(()));
    let grant = led.grant(id(1)).expect("granted");
    assert_eq!(led.release(id(1)), Ok(Release::Pinned));
    assert_eq!(led.charged(), 4_096, "pinned across the loss");

    assert_eq!(led.reset_for(Epoch(1)), Ok(()));
    assert_eq!(led.epoch(), Epoch(1));
    assert_eq!(
        led.charged(),
        0,
        "the device is gone, so pinned memory is gone too"
    );

    // The old handles are refused rather than naming whatever took
    // their index.
    let stale = AllocationId {
        epoch: E0,
        index: 1,
    };
    assert_eq!(led.charge(stale, tag, 4_096), Err(LedgerError::WrongEpoch));
    assert_eq!(led.grant(stale), Err(LedgerError::WrongEpoch));
    assert_eq!(led.release(stale), Err(LedgerError::WrongEpoch));
    assert_eq!(led.unpin(grant), Err(LedgerError::WrongEpoch));
}

/// Decision 5, the sum. The guard cannot use `charged()`, which
/// saturates for reporting: a saturating total reports a full budget
/// as a merely large one and admits the allocation that broke it.
#[test]
fn ga15_a_ledger_total_that_would_overflow_is_refused() {
    let scope = key_scope();
    let mut led = Ledger::new(Scope::Key(scope), E0);
    let tag = scope.tag(Kind::WebGpu, TabId(1));
    assert_eq!(led.charge(id(1), tag, u64::MAX - 1), Ok(()));
    assert_eq!(led.charge(id(2), tag, 1), Ok(()), "exactly u64::MAX fits");
    assert_eq!(led.charged(), u64::MAX);
    assert_eq!(led.charge(id(3), tag, 1), Err(LedgerError::Overflow));
    assert_eq!(led.len(), 2, "and the row did not join the table");
}

/// Decision 6. FP-D19: no cap is a function of the device's memory,
/// because "a device-dependent JS-visible limit is a fingerprint of
/// the device's memory". Read off the signatures, not the prose, so
/// that adding such a parameter fails here.
#[test]
fn ga16_fp_d19_no_cap_takes_a_budget() {
    let source = include_str!("../../src/caps.rs");
    let mut checked = 0_usize;
    for (line_no, line) in source.lines().enumerate() {
        let Some(rest) = line.split_once("pub const fn ").map(|(_, r)| r) else {
            continue;
        };
        // Only a real definition, not this file's own prose about one.
        if !line.trim_start().starts_with("pub const fn ") {
            continue;
        }
        let params = rest
            .split_once('(')
            .and_then(|(_, r)| r.split_once(')'))
            .map(|(p, _)| p)
            .unwrap_or_default();
        let lowered = params.to_lowercase();
        assert!(
            !lowered.contains("budget"),
            "caps.rs:{} takes a budget: `{}` (FP-D19)",
            line_no + 1,
            params
        );
        checked += 1;
    }
    assert_eq!(checked, 2, "both cap functions were read");
}

/// Decision 6, the numbers. The caps are FP 6.6's own, so a changed
/// constant fails here rather than drifting quietly.
#[test]
fn ga17_the_caps_are_the_numbers_the_footprint_spec_states() {
    // "≤ 2.5 × viewport pixels × 4 B"
    assert_eq!(caps::TILE_VIEWPORTS_NUM, 5);
    assert_eq!(caps::TILE_VIEWPORTS_DEN, 2);
    assert_eq!(caps::TILE_BYTES_PER_PX, 4);
    let viewport = 1_920 * 1_080;
    assert_eq!(caps::tile_bytes(viewport), viewport * 4 * 5 / 2);

    // "Two buffers by default, three only while an animation misses
    // frames"
    assert_eq!(caps::SWAPCHAIN_IMAGES_DEFAULT, 2);
    assert_eq!(caps::SWAPCHAIN_IMAGES_MAX, 3);
    assert_eq!(
        caps::reserved_cap_bytes(viewport, 2),
        viewport * 4 * 2,
        "the default swapchain"
    );
    assert_eq!(
        caps::reserved_cap_bytes(viewport, 9),
        viewport * 4 * 3,
        "clamped to the maximum, not trusted"
    );

    // "Each atlas starts at 256 KiB†" and the two pools together stay
    // "within 0.3's ≤ 2 MiB† per partition"
    assert_eq!(caps::ATLAS_START_BYTES, 256 * 1_024);
    assert_eq!(caps::ATLAS_POOLS_BYTES_PER_PARTITION, 2 * 1_024 * 1_024);

    // "At most 16† per document"
    assert_eq!(caps::COMPOSITOR_MUTABLE_NODES_MAX, 16);

    // Saturating, not wrapping: an absurd viewport gets the largest
    // budget expressible and never a small one.
    assert_eq!(caps::tile_bytes(u64::MAX), u64::MAX / 2);
    assert_eq!(caps::reserved_cap_bytes(u64::MAX, 3), u64::MAX);
}

/// FP 6.6's kind list is closed and complete, and the two glyph pools
/// are separate because it puts them "each on its own ledger line".
#[test]
fn ga18_every_kind_the_footprint_spec_names_has_its_own_line() {
    assert_eq!(Kind::ALL.len(), 11);
    let mut sorted = Kind::ALL;
    sorted.sort_unstable();
    let mut deduped = sorted;
    let unique = {
        let slice: &mut [Kind] = &mut deduped;
        let mut n = 0_usize;
        for i in 0..slice.len() {
            if i == 0 || slice.get(i) != slice.get(i - 1) {
                n += 1;
            }
        }
        n
    };
    assert_eq!(unique, 11, "no kind listed twice");
    assert_ne!(Kind::GlyphAtlasA8, Kind::GlyphAtlasRgba8);

    // And each really is its own ledger line.
    let scope = key_scope();
    let mut led = Ledger::new(Scope::Key(scope), E0);
    for (index, kind) in Kind::ALL.iter().enumerate() {
        let i = u32::try_from(index).expect("11 fits");
        assert_eq!(led.charge(id(i), scope.tag(*kind, TabId(1)), 100), Ok(()));
    }
    for kind in Kind::ALL {
        assert_eq!(led.charged_for(kind), 100, "one line each");
    }
    assert_eq!(led.charged(), 1_100);
}

/// An id the ledger does not hold is an error and not a silent
/// nothing, because the caller believed it held one.
#[test]
fn ga19_an_unknown_allocation_is_refused() {
    let scope = key_scope();
    let mut led = Ledger::new(Scope::Key(scope), E0);
    assert_eq!(led.grant(id(1)), Err(LedgerError::NoSuchAllocation));
    assert_eq!(led.release(id(1)), Err(LedgerError::NoSuchAllocation));
    assert!(led.is_empty());
}

/// An id names a row only if it is unique. Charging one twice
/// double-charges the site, and because `grant`, `release` and
/// `unpin` all act on the first match, the second row can never be
/// released: it stays charged until the epoch resets.
///
/// Found by review, 15 November 2026. The same shape as §3.4's two
/// defects — an index taken for an identity.
#[test]
fn ga20_an_allocation_id_is_charged_once() {
    let scope = key_scope();
    let mut led = Ledger::new(Scope::Key(scope), E0);
    let tag = scope.tag(Kind::Tile, TabId(1));

    assert_eq!(led.charge(id(1), tag, 4_096), Ok(()));
    assert_eq!(
        led.charge(id(1), tag, 4_096),
        Err(LedgerError::AlreadyCharged),
        "a second row under one id is a row nothing can release"
    );
    assert_eq!(led.charged(), 4_096, "charged once, not twice");
    assert_eq!(led.len(), 1);

    // And releasing it really does empty the ledger, which is the
    // property the duplicate broke.
    assert_eq!(led.release(id(1)), Ok(Release::Freed));
    assert!(led.is_empty());
}

/// An epoch that does not advance is not an epoch. Reset to the same
/// one and a `Grant` taken before the reset can unpin a *new* row
/// that reused its index — G45's (id, epoch) check defeated by the
/// epoch standing still.
///
/// Found by review, 15 November 2026.
#[test]
fn ga21_an_epoch_only_ever_goes_forward() {
    let scope = key_scope();
    let mut led = Ledger::new(Scope::Key(scope), Epoch(4));
    let tag = scope.tag(Kind::Tile, TabId(1));
    let old_id = AllocationId {
        epoch: Epoch(4),
        index: 1,
    };
    assert_eq!(led.charge(old_id, tag, 4_096), Ok(()));
    let stale_grant = led.grant(old_id).expect("granted");

    assert_eq!(
        led.reset_for(Epoch(4)),
        Err(LedgerError::EpochNotNewer),
        "the same epoch twice"
    );
    assert_eq!(
        led.reset_for(Epoch(3)),
        Err(LedgerError::EpochNotNewer),
        "and never backwards"
    );
    assert_eq!(led.reset_for(Epoch(5)), Ok(()));
    assert_eq!(led.epoch(), Epoch(5));
    assert!(led.is_empty());

    // The new device reuses index 1. The grant from the old epoch
    // must not reach it.
    let new_id = AllocationId {
        epoch: Epoch(5),
        index: 1,
    };
    assert_eq!(led.charge(new_id, tag, 8_192), Ok(()));
    assert_eq!(
        led.unpin(stale_grant),
        Err(LedgerError::WrongEpoch),
        "a grant from a dead device names nothing here"
    );
    assert_eq!(led.charged(), 8_192, "and the new row is untouched");
}
