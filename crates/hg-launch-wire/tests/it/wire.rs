//! IZLH v1: golden vectors, versioning rules and bounds.

use hg_launch_wire::{
    Batch, Commit, EncodeError, Intent, MAX_FIELDS, MAX_OPERAND, MAX_OPERANDS, MAX_SHORT,
    MAX_TOTAL, Message, Nonce, Origin, Outcome, Profile, Reason, Refusal, Reply, Request,
    SenderBuild, Started, decode, encode,
};

fn request() -> Request {
    Request {
        intent: Intent::Open,
        urls: vec!["https://example.org/".into()],
        paths: vec!["/home/a/notes.html".into()],
        working_dir: Some("/home/a".into()),
        profile: Some(Profile::Default),
        activation_token: Some("tok-1".into()),
        origin: Some(Origin::CommandLine),
        sender_build: Some(SenderBuild {
            build_id: 0x0123_4567_89ab_cdef,
            version: "0.0.0".into(),
            channel: "dev".into(),
        }),
        nonce: Some(Nonce {
            nonce: 42,
            batch: Some(Batch { index: 0, count: 2 }),
        }),
    }
}

fn hex(b: &[u8]) -> String {
    b.iter().map(|x| format!("{x:02x}")).collect()
}

fn unhex(s: &str) -> Vec<u8> {
    let s: Vec<u8> = s.bytes().filter(u8::is_ascii_hexdigit).collect();
    s.chunks(2)
        .map(|p| u8::from_str_radix(std::str::from_utf8(p).unwrap(), 16).unwrap())
        .collect()
}

/// The messages the golden file holds, by name.
fn golden_messages() -> Vec<(&'static str, Message)> {
    vec![
        ("request-full", Message::Request(request())),
        (
            "request-activate",
            Message::Request(Request {
                intent: Intent::Activate,
                urls: vec![],
                paths: vec![],
                working_dir: None,
                profile: None,
                activation_token: None,
                origin: None,
                sender_build: None,
                nonce: None,
            }),
        ),
        (
            "reply-accepted",
            Message::Reply(Reply {
                outcome: Outcome::Accepted,
                nonce: Some(42),
            }),
        ),
        (
            "reply-busy",
            Message::Reply(Reply {
                outcome: Outcome::BusyRetry {
                    retry_after_ms: 250,
                },
                nonce: Some(42),
            }),
        ),
        (
            "reply-refused",
            Message::Reply(Reply {
                outcome: Outcome::Refused(Reason::NotAllowed),
                nonce: None,
            }),
        ),
        ("commit", Message::Commit(Commit { nonce: Some(42) })),
        (
            "started",
            Message::Started(Started {
                build: Some(SenderBuild {
                    build_id: 7,
                    version: "0.0.0".into(),
                    channel: "dev".into(),
                }),
            }),
        ),
    ]
}

/// Every vector in `golden.txt` decodes to its message, now and in every
/// later version of this crate (Appendix C: "a golden-vector file that
/// every future version must still decode").
#[test]
fn golden_vectors_decode() {
    let file = include_str!("golden.txt");
    let want = golden_messages();
    let mut seen = 0;
    for line in file
        .lines()
        .filter(|l| !l.starts_with('#') && !l.trim().is_empty())
    {
        let (name, bytes) = line.split_once(' ').unwrap();
        let (_, msg) = want.iter().find(|(n, _)| *n == name).unwrap();
        let d = decode(&unhex(bytes)).unwrap_or_else(|e| panic!("{name}: {e:?}"));
        assert_eq!(&d.message, msg, "{name}");
        assert_eq!(d.minor, 0, "{name}");
        seen += 1;
    }
    assert_eq!(seen, want.len());
}

/// This build still writes the golden bytes. If an intentional change
/// breaks this, the file keeps its old lines (they must still decode) and
/// gains new ones; it is never rewritten.
#[test]
fn encoding_matches_the_golden_vectors() {
    let file = include_str!("golden.txt");
    let mut missing = String::new();
    for (name, msg) in golden_messages() {
        let now = hex(&encode(&msg).unwrap());
        match file
            .lines()
            .find(|l| l.split_once(' ').is_some_and(|(n, _)| n == name))
        {
            Some(line) => assert_eq!(now, line.split_once(' ').unwrap().1.trim(), "{name}"),
            None => missing.push_str(&format!("{name} {now}\n")),
        }
    }
    assert!(missing.is_empty(), "golden.txt lacks:\n{missing}");
}

/// A field with tag 40 and a value, critical or not.
fn field40(critical: bool) -> Vec<u8> {
    let mut f = vec![40, 0, u8::from(critical), 0, 3, 0, 0, 0];
    f.extend_from_slice(b"new");
    f
}

/// A v1.1 message: the v1.0 encoding with minor 1 and `extra` appended.
fn v1_1(m: &Message, extra: &[u8]) -> Vec<u8> {
    let mut b = encode(m).unwrap();
    b[5] = 1;
    b.extend_from_slice(extra);
    let total = u32::try_from(b.len()).unwrap();
    b[8..12].copy_from_slice(&total.to_le_bytes());
    b
}

#[test]
fn an_unknown_non_critical_field_is_ignored() {
    let m = Message::Request(request());
    let d = decode(&v1_1(&m, &field40(false))).unwrap();
    assert_eq!(d.minor, 1);
    assert_eq!(d.message, m);
}

#[test]
fn an_unknown_critical_field_is_refused() {
    let m = Message::Request(request());
    let e = decode(&v1_1(&m, &field40(true))).unwrap_err();
    assert_eq!(e, Refusal::UnsupportedCriticalField);
    assert_eq!(e.reason(), Reason::UnsupportedCriticalField);
}

#[test]
fn another_major_version_is_refused() {
    let mut b = encode(&Message::Commit(Commit { nonce: None })).unwrap();
    b[4] = 2;
    assert_eq!(decode(&b), Err(Refusal::UnsupportedVersion));
    assert_eq!(Refusal::UnsupportedVersion.reason(), Reason::Malformed);
}

#[test]
fn header_flags_are_ignored() {
    let m = Message::Commit(Commit { nonce: Some(1) });
    let mut b = encode(&m).unwrap();
    b[7] = 0xff;
    assert_eq!(decode(&b).unwrap().message, m);
}

#[test]
fn framing_errors_are_malformed() {
    let good = encode(&Message::Request(request())).unwrap();
    // Stated length off by one either way, a truncated field, bad magic,
    // a nonzero reserved byte, an unknown message type.
    let mut long = good.clone();
    long.push(0);
    assert_eq!(decode(&long), Err(Refusal::Malformed));
    assert_eq!(decode(&good[..good.len() - 1]), Err(Refusal::Malformed));
    let mut magic = good.clone();
    magic[0] = b'X';
    assert_eq!(decode(&magic), Err(Refusal::Malformed));
    let mut reserved = good.clone();
    reserved[12 + 3] = 1;
    assert_eq!(decode(&reserved), Err(Refusal::Malformed));
    let mut kind = good.clone();
    kind[6] = 9;
    assert_eq!(decode(&kind), Err(Refusal::Malformed));
    assert_eq!(decode(&[]), Err(Refusal::Malformed));
}

#[test]
fn a_request_needs_its_intent_once() {
    let mut r = request();
    r.intent = Intent::Open;
    let b = encode(&Message::Request(r)).unwrap();
    // The intent is the first field: 8-byte header and 1 byte of value.
    let without: Vec<u8> = b[..12].iter().chain(&b[12 + 9..]).copied().collect();
    let mut without = without;
    let total = u32::try_from(without.len()).unwrap();
    without[8..12].copy_from_slice(&total.to_le_bytes());
    assert_eq!(decode(&without), Err(Refusal::Malformed));

    let mut twice: Vec<u8> = b.clone();
    twice.extend_from_slice(&b[12..12 + 9]);
    let total = u32::try_from(twice.len()).unwrap();
    twice[8..12].copy_from_slice(&total.to_le_bytes());
    assert_eq!(decode(&twice), Err(Refusal::Malformed));
}

#[test]
fn bounds_are_enforced_both_ways() {
    // Operands.
    let mut r = request();
    r.urls = vec!["https://a/".into(); MAX_OPERANDS];
    assert_eq!(
        encode(&Message::Request(r.clone())),
        Err(EncodeError::TooLarge)
    );
    r.paths.clear();
    let ok = encode(&Message::Request(r.clone())).unwrap();
    assert!(decode(&ok).is_ok());
    // One more URL field appended by hand: too many operands.
    let mut more = ok.clone();
    more.extend_from_slice(&[2, 0, 1, 0, 3, 0, 0, 0]);
    more.extend_from_slice(b"x:y");
    let total = u32::try_from(more.len()).unwrap();
    more[8..12].copy_from_slice(&total.to_le_bytes());
    assert_eq!(decode(&more), Err(Refusal::TooLarge));

    // Value lengths.
    let mut r = request();
    r.urls = vec!["h".repeat(MAX_OPERAND + 1)];
    assert_eq!(encode(&Message::Request(r)), Err(EncodeError::TooLarge));
    let mut r = request();
    r.urls = vec!["h".repeat(MAX_OPERAND)];
    assert!(decode(&encode(&Message::Request(r)).unwrap()).is_ok());
    let mut r = request();
    r.activation_token = Some("t".repeat(MAX_SHORT + 1));
    assert_eq!(encode(&Message::Request(r)), Err(EncodeError::TooLarge));

    // The total.
    let mut r = request();
    r.paths.clear();
    r.urls = vec!["u".repeat(MAX_OPERAND); MAX_OPERANDS];
    let b = encode(&Message::Request(r)).unwrap();
    assert!(b.len() <= MAX_TOTAL);
    let mut big = vec![0u8; MAX_TOTAL + 1];
    big[..4].copy_from_slice(b"IZLH");
    assert_eq!(decode(&big), Err(Refusal::TooLarge));

    // The field count, unknown fields included.
    let m = Message::Commit(Commit { nonce: None });
    let mut b = encode(&m).unwrap();
    for _ in 0..=MAX_FIELDS {
        b.extend_from_slice(&field40(false));
    }
    let total = u32::try_from(b.len()).unwrap();
    b[8..12].copy_from_slice(&total.to_le_bytes());
    assert_eq!(decode(&b), Err(Refusal::TooLarge));
}

#[test]
fn text_must_be_well_formed_and_nul_free() {
    let mut r = request();
    r.urls = vec!["a\0b".into()];
    assert_eq!(encode(&Message::Request(r)), Err(EncodeError::Invalid));
    let b = encode(&Message::Request(request())).unwrap();
    // The URL is the second field: replace its first byte with 0xff.
    let url_value = 12 + 9 + 8;
    let mut bad = b.clone();
    bad[url_value] = 0xff;
    assert_eq!(decode(&bad), Err(Refusal::Malformed));
    let mut nul = b;
    nul[url_value] = 0;
    assert_eq!(decode(&nul), Err(Refusal::Malformed));
}

#[test]
fn every_message_round_trips() {
    for (name, m) in golden_messages() {
        let d = decode(&encode(&m).unwrap()).unwrap();
        assert_eq!(d.message, m, "{name}");
    }
    let id = Message::Request(Request {
        profile: Some(Profile::Id("work".into())),
        ..request()
    });
    assert_eq!(decode(&encode(&id).unwrap()).unwrap().message, id);
    let bad = Message::Request(Request {
        profile: Some(Profile::Id("default".into())),
        ..request()
    });
    assert_eq!(encode(&bad), Err(EncodeError::Invalid));
}
