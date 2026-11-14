//! The transition table, exercised in both directions.
//!
//! The first few tests walk the handshakes RFC 8446 Appendix A.1 draws.
//! The last one is the one that matters: it walks **every** state
//! against **every** message and requires that each pair the table does
//! not name is an alert. That is 81 pairs, of which 10 are allowed —
//! the table has 11 rows, two of them the same `(state, message)` with
//! different destinations — and writing the other 71 by hand is how a
//! state machine ends up accepting one of them.

use hg_tls::handshake::{Action, Alert, Handshake, Input, State, TRANSITIONS};

const EVERY_STATE: [State; 9] = [
    State::Start,
    State::WaitSh,
    State::WaitEe,
    State::WaitCertCr,
    State::WaitCert,
    State::WaitCv,
    State::WaitFinished,
    State::Connected,
    State::Closed,
];

const EVERY_INPUT: [Input; 9] = [
    Input::ServerHello,
    Input::HelloRetryRequest,
    Input::EncryptedExtensions,
    Input::CertificateRequest,
    Input::Certificate,
    Input::CertificateVerify,
    Input::Finished,
    Input::NewSessionTicket,
    Input::KeyUpdate,
];

/// Drives a handshake to `target` by the shortest legal path, or
/// `None` when there is none (`Start` and `Closed` are reached
/// otherwise).
fn at(target: State) -> Option<Handshake> {
    let mut h = Handshake::new();
    if target == State::Start {
        return Some(h);
    }
    h.start().ok()?;
    let path: &[Input] = match target {
        State::WaitSh => &[],
        State::WaitEe => &[Input::ServerHello],
        State::WaitCertCr => &[Input::ServerHello, Input::EncryptedExtensions],
        State::WaitCert => &[
            Input::ServerHello,
            Input::EncryptedExtensions,
            Input::CertificateRequest,
        ],
        State::WaitCv => &[
            Input::ServerHello,
            Input::EncryptedExtensions,
            Input::Certificate,
        ],
        State::WaitFinished => &[
            Input::ServerHello,
            Input::EncryptedExtensions,
            Input::Certificate,
            Input::CertificateVerify,
        ],
        State::Connected => &[
            Input::ServerHello,
            Input::EncryptedExtensions,
            Input::Certificate,
            Input::CertificateVerify,
            Input::Finished,
        ],
        State::Closed => {
            // Reached by *refusing* one message, so the error is the
            // thing that gets there.
            let refused = h.on_message(Input::Finished);
            assert!(refused.is_err(), "Finished in WAIT_SH should be refused");
            return Some(h);
        }
        State::Start => return Some(h),
    };
    for i in path {
        h.on_message(*i).ok()?;
    }
    Some(h)
}

/// The full certificate handshake of Appendix A.1.
#[test]
fn tl01_the_certificate_handshake() {
    let mut h = Handshake::new();
    assert_eq!(h.start(), Ok(Action::SendClientHello));
    assert_eq!(h.state(), State::WaitSh);
    assert_eq!(h.on_message(Input::ServerHello), Ok(Action::Continue));
    assert_eq!(h.state(), State::WaitEe);
    assert_eq!(
        h.on_message(Input::EncryptedExtensions),
        Ok(Action::Continue)
    );
    assert_eq!(h.state(), State::WaitCertCr);
    assert_eq!(h.on_message(Input::Certificate), Ok(Action::Continue));
    assert_eq!(h.state(), State::WaitCv);
    assert_eq!(h.on_message(Input::CertificateVerify), Ok(Action::Continue));
    assert_eq!(h.state(), State::WaitFinished);
    assert_eq!(
        h.on_message(Input::Finished),
        Ok(Action::SendClientFlight {
            certificate_requested: false
        })
    );
    assert_eq!(h.state(), State::Connected);
    assert_eq!(h.messages(), 5);
}

/// With a CertificateRequest, the client's own flight carries a
/// Certificate.
#[test]
fn tl02_a_requested_client_certificate_changes_the_flight() {
    let mut h = Handshake::new();
    h.start().expect("start");
    for i in [
        Input::ServerHello,
        Input::EncryptedExtensions,
        Input::CertificateRequest,
        Input::Certificate,
        Input::CertificateVerify,
    ] {
        h.on_message(i).expect("legal");
    }
    assert_eq!(
        h.on_message(Input::Finished),
        Ok(Action::SendClientFlight {
            certificate_requested: true
        })
    );
}

/// A resumed handshake takes the other edge out of `WAIT_EE`: there is
/// no Certificate, and a Certificate sent anyway is an alert.
#[test]
fn tl03_a_resumed_handshake_skips_the_certificate() {
    let mut h = Handshake::resumed();
    h.start().expect("start");
    h.on_message(Input::ServerHello).expect("legal");
    h.on_message(Input::EncryptedExtensions).expect("legal");
    assert_eq!(h.state(), State::WaitFinished, "not WAIT_CERT_CR");
    assert_eq!(
        h.on_message(Input::Finished),
        Ok(Action::SendClientFlight {
            certificate_requested: false
        })
    );

    // The twin: the same messages on a certificate handshake go the
    // other way.
    let mut c = Handshake::new();
    c.start().expect("start");
    c.on_message(Input::ServerHello).expect("legal");
    c.on_message(Input::EncryptedExtensions).expect("legal");
    assert_eq!(c.state(), State::WaitCertCr);
}

/// §4.1.4: one HelloRetryRequest is allowed and a second is an alert.
#[test]
fn tl04_a_second_hello_retry_request_is_refused() {
    let mut h = Handshake::new();
    h.start().expect("start");
    assert_eq!(
        h.on_message(Input::HelloRetryRequest),
        Ok(Action::SendSecondClientHello)
    );
    assert_eq!(h.state(), State::WaitSh, "still waiting for a ServerHello");
    assert!(h.hello_retry_seen());
    assert_eq!(
        h.on_message(Input::HelloRetryRequest),
        Err(Alert::SecondHelloRetryRequest)
    );
    assert_eq!(h.state(), State::Closed);

    // The twin: one retry then a ServerHello completes.
    let mut t = Handshake::new();
    t.start().expect("start");
    t.on_message(Input::HelloRetryRequest).expect("the first");
    assert_eq!(t.on_message(Input::ServerHello), Ok(Action::Continue));
}

/// §4.6: post-handshake messages are legal once connected and not
/// before.
#[test]
fn tl05_post_handshake_messages_need_a_handshake() {
    let mut early = at(State::WaitFinished).expect("reachable");
    assert!(matches!(
        early.on_message(Input::NewSessionTicket),
        Err(Alert::UnexpectedMessage { .. })
    ));

    let mut done = at(State::Connected).expect("reachable");
    assert_eq!(
        done.on_message(Input::NewSessionTicket),
        Ok(Action::PostHandshake)
    );
    assert_eq!(done.on_message(Input::KeyUpdate), Ok(Action::PostHandshake));
    assert_eq!(done.state(), State::Connected);
}

/// After an alert nothing more is read, whatever arrives.
#[test]
fn tl06_a_closed_handshake_stays_closed() {
    let mut h = at(State::Closed).expect("reachable");
    assert_eq!(h.state(), State::Closed);
    for i in EVERY_INPUT {
        assert!(
            h.on_message(i).is_err(),
            "{i:?} was accepted after an alert"
        );
    }
}

/// **The test the table exists for.** Every state against every
/// message: a pair the table names is accepted, and every other pair
/// is an alert. There are 81 pairs and 11 transitions.
#[test]
fn tl07_everything_the_table_does_not_name_is_an_alert() {
    let mut allowed = 0_usize;
    let mut refused = 0_usize;
    for state in EVERY_STATE {
        for input in EVERY_INPUT {
            let Some(mut h) = at(state) else {
                panic!("{state} is not reachable");
            };
            assert_eq!(h.state(), state, "the walk did not reach {state}");
            let in_table = TRANSITIONS
                .iter()
                .any(|(f, i, _)| *f == state && *i == input);
            let got = h.on_message(input);
            if in_table {
                assert!(
                    got.is_ok(),
                    "{input:?} in {state} is in the table and was refused: {got:?}"
                );
                allowed = allowed.saturating_add(1);
            } else {
                assert!(
                    got.is_err(),
                    "{input:?} in {state} is NOT in the table and was accepted"
                );
                assert_eq!(h.state(), State::Closed, "a refusal closes the handshake");
                refused = refused.saturating_add(1);
            }
        }
    }
    assert_eq!(
        allowed.saturating_add(refused),
        EVERY_STATE.len().saturating_mul(EVERY_INPUT.len())
    );
    // 11 rows in the table but 10 distinct `(from, input)` pairs: the
    // `WAIT_EE` / EncryptedExtensions edge appears twice, once per
    // destination, because the standard's diagram splits it.
    let mut distinct: Vec<(State, Input)> = TRANSITIONS.iter().map(|(f, i, _)| (*f, *i)).collect();
    distinct.sort_unstable();
    distinct.dedup();
    assert_eq!(allowed, distinct.len(), "one accept per distinct pair");
    assert_eq!(
        refused,
        EVERY_STATE
            .len()
            .saturating_mul(EVERY_INPUT.len())
            .saturating_sub(distinct.len()),
        "every other pair is an alert"
    );
    assert_eq!((allowed, refused), (10, 71));
    println!(
        "hg-tls: {allowed} transitions, {refused} alerts, over {} pairs",
        allowed + refused
    );
}

/// The table holds no duplicate `(from, input)` pair except the one
/// `WAIT_EE` edge the standard itself splits, and every state it names
/// is reachable.
#[test]
fn tl08_the_table_is_well_formed() {
    let mut pairs: Vec<(State, Input)> = TRANSITIONS.iter().map(|(f, i, _)| (*f, *i)).collect();
    pairs.sort_unstable();
    let before = pairs.len();
    pairs.dedup();
    assert_eq!(
        before.saturating_sub(pairs.len()),
        1,
        "exactly one input has two edges: EncryptedExtensions out of WAIT_EE"
    );
    for (from, _, to) in TRANSITIONS {
        assert!(EVERY_STATE.contains(from), "{from} is not a listed state");
        assert!(EVERY_STATE.contains(to), "{to} is not a listed state");
        assert_ne!(*to, State::Closed, "no transition goes to CLOSED");
    }
    // Every handshake type is distinct except the ServerHello pair the
    // standard itself shares.
    let mut types: Vec<u8> = EVERY_INPUT.iter().map(|i| i.handshake_type()).collect();
    types.sort_unstable();
    let before = types.len();
    types.dedup();
    assert_eq!(
        before.saturating_sub(types.len()),
        1,
        "only ServerHello doubles"
    );
}

/// A client cannot start twice.
#[test]
fn tl09_a_second_start_is_refused() {
    let mut h = Handshake::new();
    h.start().expect("the first");
    assert!(h.start().is_err());
}
