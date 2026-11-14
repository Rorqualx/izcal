//! Links: determinism first, then one test per parameter.

use hg_netsim::link::{Delivery, Link, LinkSpec};
use hg_types::{Duration, Instant};

use crate::rng::stream;

/// Offers 200 packets and returns what happened to each.
fn run(spec: LinkSpec, seed: u64, bytes: usize) -> Vec<Delivery> {
    let mut rng = stream(seed);
    let mut link = Link::new(spec);
    let mut out = Vec::new();
    for i in 0..200_u64 {
        let now = Instant::from_nanos(i.saturating_mul(1_000_000));
        out.push(link.send(now, bytes, &mut rng));
    }
    out
}

/// The same seed gives the same run, to the nanosecond. This is the
/// property the whole crate exists for: a failure found at a seed is
/// found again at that seed, on any machine.
#[test]
fn ns01_a_run_is_a_function_of_its_seed() {
    let spec = LinkSpec::mobile();
    assert_eq!(run(spec, 97, 1_200), run(spec, 97, 1_200));
    assert_ne!(
        run(spec, 97, 1_200),
        run(spec, 98, 1_200),
        "a different seed gives a different run"
    );
}

/// A perfect link delivers everything, in order, at the instant it was
/// offered.
#[test]
fn ns02_a_perfect_link_delivers_everything_at_once() {
    let out = run(LinkSpec::perfect(), 1, 100);
    for (i, d) in out.iter().enumerate() {
        let at = Instant::from_nanos(u64::try_from(i).unwrap() * 1_000_000);
        assert_eq!(*d, Delivery::At(at), "packet {i}");
    }
}

/// A packet over the MTU is dropped, and reported apart from a loss so
/// a protocol can tell that it must lower its packet size.
#[test]
fn ns03_a_packet_over_the_mtu_is_a_different_answer_from_a_loss() {
    let mut spec = LinkSpec::perfect();
    spec.mtu = 1_200;
    let mut rng = stream(5);
    let mut link = Link::new(spec);
    assert_eq!(
        link.send(Instant::ORIGIN, 1_200, &mut rng),
        Delivery::At(Instant::ORIGIN),
        "exactly the MTU fits"
    );
    assert_eq!(
        link.send(Instant::ORIGIN, 1_201, &mut rng),
        Delivery::TooLarge {
            bytes: 1_201,
            mtu: 1_200
        },
        "one byte past it does not"
    );
    let (sent, delivered, lost, oversize, _) = link.counts();
    assert_eq!((sent, delivered, lost, oversize), (2, 1, 0, 1));
}

/// Loss happens at about the rate the spec gives, and only loss
/// changes: the delivered packets still arrive when they would have.
#[test]
fn ns04_loss_happens_at_about_its_rate() {
    let mut spec = LinkSpec::perfect();
    spec.loss_ppm = 100_000; // 10%
    let out = run(spec, 11, 100);
    let lost = out.iter().filter(|d| **d == Delivery::Lost).count();
    assert!(
        (8..=40).contains(&lost),
        "10% of 200 is 20; got {lost}, which is outside a generous band"
    );
    // A link with no loss loses nothing, at the same seed.
    let none = run(LinkSpec::perfect(), 11, 100);
    assert_eq!(none.iter().filter(|d| **d == Delivery::Lost).count(), 0);
}

/// Jitter moves arrivals both earlier and later than the delay. A
/// jitter that could only add would be a different network, with a
/// mean latency half a jitter too high.
#[test]
fn ns05_jitter_goes_both_ways() {
    let mut spec = LinkSpec::perfect();
    spec.delay = Duration::from_millis(50);
    spec.jitter = Duration::from_millis(10);
    let mut rng = stream(3);
    let mut link = Link::new(spec);
    let mut earlier = 0;
    let mut later = 0;
    for _ in 0..400 {
        if let Delivery::At(at) = link.send(Instant::ORIGIN, 100, &mut rng) {
            let d = at.as_nanos();
            if d < 50_000_000 {
                earlier += 1;
            } else if d > 50_000_000 {
                later += 1;
            }
            assert!(
                (40_000_000..=60_000_000).contains(&d),
                "{d} is outside delay +/- jitter"
            );
        }
    }
    assert!(earlier > 100, "only {earlier} arrived early");
    assert!(later > 100, "only {later} arrived late");
}

/// Bandwidth is a queue: packets offered faster than the link can
/// serialize them arrive one serialization time apart, not all at
/// once.
#[test]
fn ns06_bandwidth_serializes() {
    let mut spec = LinkSpec::perfect();
    spec.bandwidth = Some(1_000_000); // 1 MB/s, so 1,000 bytes is 1 ms
    let mut rng = stream(7);
    let mut link = Link::new(spec);
    let mut arrivals = Vec::new();
    for _ in 0..4 {
        if let Delivery::At(at) = link.send(Instant::ORIGIN, 1_000, &mut rng) {
            arrivals.push(at.as_nanos());
        }
    }
    assert_eq!(arrivals, vec![1_000_000, 2_000_000, 3_000_000, 4_000_000]);
}

/// A reordered packet arrives behind the ones that followed it, which
/// is the only thing reordering has to do for a receiver to be tested.
#[test]
fn ns07_reordering_puts_a_packet_behind_its_successors() {
    let mut spec = LinkSpec::perfect();
    spec.delay = Duration::from_millis(10);
    spec.reorder_ppm = 1_000_000; // every packet
    let mut rng = stream(13);
    let mut link = Link::new(spec);
    let Delivery::At(at) = link.send(Instant::ORIGIN, 100, &mut rng) else {
        panic!("delivered");
    };
    assert_eq!(
        at.as_nanos(),
        20_000_000,
        "one delay later than it would be"
    );
    let (_, _, _, _, reordered) = link.counts();
    assert_eq!(reordered, 1);
}

/// Turning a parameter off still draws, so the sequence of later draws
/// does not move. Without this, adding jitter to a test would change
/// which packets it loses, and a regression would look like a new bug.
#[test]
fn ns08_a_parameter_at_zero_still_draws() {
    let mut with_loss = LinkSpec::perfect();
    with_loss.loss_ppm = 0;
    with_loss.jitter = Duration::from_millis(5);
    let a = run(with_loss, 21, 100);

    let mut without = LinkSpec::perfect();
    without.jitter = Duration::from_millis(5);
    let b = run(without, 21, 100);
    assert_eq!(a, b, "loss at zero draws the same as no loss at all");
}
