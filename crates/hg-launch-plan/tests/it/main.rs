//! `hg-launch-plan` integration tests (coding standard CS-TE-2).

use hg_launch_plan::{Exe, Kind, REPLY_LEN, REQUEST_LEN, Reply, Request, Stage, Tier, WireError};

#[test]
fn every_request_round_trips() {
    for kind in Kind::ALL {
        for id in [0, 1, u64::MAX, 0x0102_0304_0506_0708] {
            let r = Request::Spawn { id, kind };
            assert_eq!(Request::decode(&r.encode()), Ok(r));
        }
    }
}

#[test]
fn every_reply_round_trips() {
    let mut all = Vec::new();
    for tier in Tier::ALL {
        all.push(Reply::Ready { tier });
        all.push(Reply::Spawned {
            id: 7,
            tier,
            outer_pid: 4_194_303,
            ns_pid: 1,
        });
    }
    for stage in [
        Stage::Request,
        Stage::Clone,
        Stage::Descriptors,
        Stage::Landlock,
        Stage::Seccomp,
        Stage::Exec,
        Stage::Unknown,
    ] {
        all.push(Reply::Failed {
            id: u64::MAX,
            stage,
            errno: 13,
        });
    }
    for r in all {
        assert_eq!(Reply::decode(&r.encode()), Ok(r), "{r:?}");
    }
}

#[test]
fn malformed_requests_are_refused() {
    let good = Request::Spawn {
        id: 9,
        kind: Kind::Renderer,
    }
    .encode();
    assert_eq!(Request::decode(&good[..15]), Err(WireError::Length));
    let mut long = good.to_vec();
    long.push(0);
    assert_eq!(Request::decode(&long), Err(WireError::Length));
    let mut b = good;
    b[0] = b'X';
    assert_eq!(Request::decode(&b), Err(WireError::Header));
    let mut b = good;
    b[4] = 2;
    assert_eq!(Request::decode(&b), Err(WireError::Header));
    let mut b = good;
    b[5] = 2;
    assert_eq!(Request::decode(&b), Err(WireError::Op));
    let mut b = good;
    b[6] = 0;
    assert_eq!(Request::decode(&b), Err(WireError::Value));
    let mut b = good;
    b[6] = 5;
    assert_eq!(Request::decode(&b), Err(WireError::Value));
    let mut b = good;
    b[7] = 1;
    assert_eq!(Request::decode(&b), Err(WireError::Reserved));
}

#[test]
fn malformed_replies_are_refused() {
    let ready = Reply::Ready {
        tier: Tier::RestrictedUserns,
    }
    .encode();
    assert_eq!(
        Reply::decode(&ready[..REPLY_LEN - 1]),
        Err(WireError::Length)
    );
    let mut b = ready;
    b[8] = 1; // a ready record carries no id
    assert_eq!(Reply::decode(&b), Err(WireError::Reserved));
    let mut b = ready;
    b[6] = 4; // no tier 4
    assert_eq!(Reply::decode(&b), Err(WireError::Value));
    let mut b = ready;
    b[5] = 9;
    assert_eq!(Reply::decode(&b), Err(WireError::Op));
    let failed = Reply::Failed {
        id: 1,
        stage: Stage::Exec,
        errno: 2,
    }
    .encode();
    let mut b = failed;
    b[20] = 1; // x must be zero in a failed record
    assert_eq!(Reply::decode(&b), Err(WireError::Reserved));
    let mut b = failed;
    b[6] = 8;
    assert_eq!(Reply::decode(&b), Err(WireError::Value));
    let mut b = failed;
    b[7] = 1;
    assert_eq!(Reply::decode(&b), Err(WireError::Reserved));
}

#[test]
fn descriptor_counts_are_per_record() {
    assert_eq!(Request::FDS, 2);
    assert_eq!(
        Reply::Spawned {
            id: 1,
            tier: Tier::Full,
            outer_pid: 2,
            ns_pid: 1
        }
        .fds(),
        1
    );
    assert_eq!(
        Reply::Failed {
            id: 1,
            stage: Stage::Clone,
            errno: 1
        }
        .fds(),
        0
    );
    assert_eq!(
        Reply::Ready {
            tier: Tier::NoUserns
        }
        .fds(),
        0
    );
}

#[test]
fn kinds_derive_their_executable_and_floor() {
    assert_eq!(Kind::Renderer.exe(), Exe::Renderer);
    for k in [
        Kind::NetworkSystem,
        Kind::NetworkContainer,
        Kind::NetworkPartition,
    ] {
        assert_eq!(k.exe(), Exe::Net);
    }
    assert_eq!(Exe::Renderer.file_name(), "hg-renderer-main");
    assert_eq!(Exe::Net.file_name(), "hg-net-main");
    for k in Kind::ALL {
        assert_eq!(
            k.floor(),
            Tier::NoUserns,
            "seccomp and Landlock are every kind's floor"
        );
    }
}

#[test]
fn tiers_are_ordered_weakest_first() {
    assert!(Tier::NoUserns < Tier::RestrictedUserns);
    assert!(Tier::RestrictedUserns < Tier::Full);
    assert_eq!(Tier::RestrictedUserns.name(), "linux-restricted-userns");
    assert_eq!(REQUEST_LEN, 16);
}
