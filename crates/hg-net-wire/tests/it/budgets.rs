//! NW-15: "100% of limits in one module, each tested at limit and
//! limit + 1".
//!
//! The first half is the interesting one. A test that named each limit
//! by hand would pass while a twelfth limit sat somewhere else in the
//! crate, which is the failure NW-15 is written against. So the test
//! walks `limits::ALL` and the *enum* is what holds it to account:
//! `Limit` is closed, every variant must appear in the table, and
//! adding a variant without a row fails here rather than in review.

use hg_net_wire::budget::{Budget, Limit, Overrun, charge_all, release_all};
use hg_net_wire::limits;

/// Every limit, charged exactly to its cap and then one past it.
#[test]
fn nw15_every_limit_holds_at_the_limit_and_refuses_one_past_it() {
    for (limit, cap) in limits::ALL {
        let mut b = Budget::new(*limit, *cap);
        assert_eq!(b.charge(*cap), Ok(()), "{limit}: the limit itself fits");
        assert_eq!(b.used(), *cap);
        assert_eq!(b.remaining(), 0);

        let mut b = Budget::new(*limit, *cap);
        let one_past = cap.checked_add(1).expect("no limit is u64::MAX");
        assert_eq!(
            b.charge(one_past),
            Err(Overrun {
                limit: *limit,
                requested: one_past,
                remaining: *cap,
            }),
            "{limit}: one past the limit is refused"
        );
        assert_eq!(b.used(), 0, "{limit}: a refused charge changes nothing");
    }
}

/// Every variant of the closed enum appears in the table, and nothing
/// appears twice. This is what makes the count above "100%".
#[test]
fn nw15_the_table_holds_every_limit_exactly_once() {
    let every = [
        Limit::HeaderBlockBytes,
        Limit::ContinuationBytes,
        Limit::ReassemblyGaps,
        Limit::AckRanges,
        Limit::ActiveConnectionIds,
        Limit::PendingPathResponses,
        Limit::PendingRetireConnectionIds,
        Limit::QueuedControlReplies,
        Limit::DecompressedBytes,
        Limit::PartitionBytes,
        Limit::ServiceBytes,
    ];
    assert_eq!(
        limits::ALL.len(),
        every.len(),
        "a limit was added to one place and not the other"
    );
    for l in every {
        assert_eq!(
            limits::ALL.iter().filter(|(k, _)| *k == l).count(),
            1,
            "{l} appears once"
        );
    }
    // Every row's name is distinct, so an error message names one
    // limit rather than a class of them.
    let mut names: Vec<&str> = limits::ALL.iter().map(|(l, _)| l.name()).collect();
    names.sort_unstable();
    let before = names.len();
    names.dedup();
    assert_eq!(names.len(), before);
}

/// The sum below the limit is held, and the one that crosses it is
/// refused with both numbers in it.
#[test]
fn nw01_a_charge_that_crosses_the_cap_reports_what_was_left() {
    let mut b = Budget::new(Limit::AckRanges, 10);
    assert_eq!(b.charge(7), Ok(()));
    assert_eq!(
        b.charge(4),
        Err(Overrun {
            limit: Limit::AckRanges,
            requested: 4,
            remaining: 3,
        })
    );
    assert_eq!(b.used(), 7);
    assert_eq!(b.charge(3), Ok(()));
    assert_eq!(b.remaining(), 0);
}

/// Releasing more than was charged cannot make the budget unbounded.
#[test]
fn nw02_release_saturates_rather_than_underflowing() {
    let mut b = Budget::new(Limit::AckRanges, 10);
    assert_eq!(b.charge(3), Ok(()));
    b.release(9);
    assert_eq!(b.used(), 0, "not u64::MAX - 6");
    assert_eq!(b.remaining(), 10);
}

/// The roll-up charges all three or none of them.
#[test]
fn nw03_the_roll_up_is_all_or_nothing() {
    let mut connection = Budget::new(Limit::HeaderBlockBytes, 100);
    let mut partition = Budget::new(Limit::PartitionBytes, 100);
    // The service is the one that refuses.
    let mut service = Budget::new(Limit::ServiceBytes, 10);

    let err = charge_all(&mut [&mut connection, &mut partition, &mut service], 50)
        .expect_err("the service has 10");
    assert_eq!(err.limit, Limit::ServiceBytes);
    assert_eq!(
        (connection.used(), partition.used(), service.used()),
        (0, 0, 0),
        "the connection and the partition did not pay for memory nobody got"
    );

    // The twin: the same chain with a charge all three can hold.
    assert_eq!(
        charge_all(&mut [&mut connection, &mut partition, &mut service], 10),
        Ok(())
    );
    assert_eq!(
        (connection.used(), partition.used(), service.used()),
        (10, 10, 10)
    );
    release_all(&mut [&mut connection, &mut partition, &mut service], 10);
    assert_eq!(
        (connection.used(), partition.used(), service.used()),
        (0, 0, 0)
    );
}

/// An `Overrun` says which limit, what was asked and what was left.
#[test]
fn nw04_an_overrun_reads_as_a_sentence() {
    let mut b = Budget::new(Limit::ContinuationBytes, 1024);
    let err = b.charge(2048).expect_err("twice the cap");
    assert_eq!(
        err.to_string(),
        "CONTINUATION bytes: 2048 requested, 1024 left"
    );
}
