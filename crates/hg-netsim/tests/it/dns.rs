//! The resolver's peer: the answers a stub resolver has to tell apart.

use hg_netsim::dns::{Answer, Behaviour, Query, Record, Resolver};
use hg_types::Duration;

use crate::rng::stream;

fn zone() -> Resolver {
    let mut r = Resolver::new();
    r.rtt = Duration::from_millis(15);
    r.add("example.test", Query::A, Record::A([93, 184, 216, 34]))
        .add(
            "example.test",
            Query::Aaaa,
            Record::Aaaa([0x20, 0x01, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1]),
        )
        .add("v4only.test", Query::A, Record::A([10, 0, 0, 1]));
    r
}

/// A name with records answers with them, after the round trip.
#[test]
fn ns10_a_name_answers_with_its_records() {
    let mut r = zone();
    let mut rng = stream(1);
    let a = r.ask("example.test", Query::A, &mut rng);
    assert_eq!(
        a,
        Answer::Records {
            records: vec![Record::A([93, 184, 216, 34])],
            after: Duration::from_millis(15),
        }
    );
    assert_eq!(r.queries(), 1);
}

/// The distinction every resolver gets wrong at least once: a name
/// that exists but has no record of the asked type is `NOERROR` with
/// no records, not `NXDOMAIN`. Happy Eyeballs must not treat the two
/// alike — one means "try the other family", the other means "stop".
#[test]
fn ns11_no_records_is_not_no_such_name() {
    let mut r = zone();
    let mut rng = stream(2);
    assert_eq!(
        r.ask("v4only.test", Query::Aaaa, &mut rng),
        Answer::Records {
            records: vec![],
            after: Duration::from_millis(15),
        },
        "the name exists; this family does not"
    );
    assert_eq!(
        r.ask("absent.test", Query::A, &mut rng),
        Answer::NoSuchName {
            after: Duration::from_millis(15)
        },
        "the name does not exist"
    );
}

/// Each failure is its own answer, because a stub resolver does a
/// different thing with each: retry over TCP, try another server, or
/// wait for the timeout.
#[test]
fn ns12_every_failure_is_its_own_answer() {
    let mut r = zone();
    r.set_behaviour("slow.test", Behaviour::Silent)
        .set_behaviour("broken.test", Behaviour::ServerFailure)
        .set_behaviour("big.test", Behaviour::Truncated);
    let mut rng = stream(3);
    assert_eq!(r.ask("slow.test", Query::A, &mut rng), Answer::Silent);
    assert!(matches!(
        r.ask("broken.test", Query::A, &mut rng),
        Answer::ServerFailure { .. }
    ));
    assert!(matches!(
        r.ask("big.test", Query::A, &mut rng),
        Answer::Truncated { .. }
    ));
    assert_eq!(r.queries(), 3);
}

/// The round trip varies in both directions and stays inside the
/// jitter, and the same seed gives the same times.
#[test]
fn ns13_the_round_trip_is_a_function_of_the_seed() {
    let mut r = zone();
    r.jitter = Duration::from_millis(5);
    let times = |seed: u64| {
        let mut r = r.clone();
        let mut rng = stream(seed);
        (0..50)
            .map(|_| match r.ask("example.test", Query::A, &mut rng) {
                Answer::Records { after, .. } => after.as_nanos(),
                Answer::NoSuchName { .. }
                | Answer::ServerFailure { .. }
                | Answer::Truncated { .. }
                | Answer::Silent => 0,
            })
            .collect::<Vec<_>>()
    };
    let a = times(31);
    assert_eq!(a, times(31));
    assert_ne!(a, times(32));
    assert!(a.iter().any(|t| *t < 15_000_000), "some are early");
    assert!(a.iter().any(|t| *t > 15_000_000), "some are late");
    assert!(
        a.iter().all(|t| (10_000_000..=20_000_000).contains(t)),
        "all inside the jitter"
    );
}

/// A failing name does not move the timing of the names around it:
/// every answer draws once, whatever it is.
#[test]
fn ns14_adding_a_failure_does_not_shift_the_other_draws() {
    let mut rng = stream(41);
    let mut plain = zone();
    plain.jitter = Duration::from_millis(5);
    let a: Vec<_> = (0..10)
        .map(|_| plain.ask("example.test", Query::A, &mut rng))
        .collect();

    let mut rng = stream(41);
    let mut with_failure = zone();
    with_failure.jitter = Duration::from_millis(5);
    with_failure.set_behaviour("other.test", Behaviour::ServerFailure);
    let b: Vec<_> = (0..10)
        .map(|_| with_failure.ask("example.test", Query::A, &mut rng))
        .collect();
    assert_eq!(a, b);
}
