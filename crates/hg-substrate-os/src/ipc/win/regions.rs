//! The broker's region registry on Windows (M1 phase D6; review B1).
//!
//! Windows cannot attest what a receiver most needs about a section: the
//! receiver cannot learn its exact size (fact S2), and nobody can see a
//! process's views (fact S8). So the broker, which makes every section,
//! keeps a record of each one and checks every region relay against it:
//!
//! - the pulled handle is a section (its type name), one of the broker's
//!   own (`CompareObjectHandles` against the broker's handle, fact S3),
//!   with exactly the access its kind gives (S1, S2);
//! - the declared length is exactly the registered size (review minor 1);
//! - the declared kind fits the section's state, which the broker moves
//!   on as it sees the region move:
//!
//! | State | Writer | Read-only | Consumer |
//! |---|---|---|---|
//! | `WriterOut(broker)` | broker only, no other handle: in transit | refused | refused |
//! | `WriterOut(p)`, `p` relays | refused (not the broker) | no other handle: `SealedByChild` | `Producer(p)` |
//! | `SealedByBroker` | refused | yes, immutable | refused |
//! | `SealedByChild` | refused | yes, filler-trusted, or copied | refused |
//! | `Producer(p)`, anyone relays | refused | refused | yes |
//!
//! A consumer region sent as read-only is so refused: its section is a
//! producer's (review B1's test). The producer sends its first consumer
//! (the section is then `WriterOut(p)` with `p` the relayer); after that
//! any holder may pass a consumer on, as on Linux and macOS, where a
//! consumer is a sealed object anyone holding it can send: the design
//! gives `ConsumerRegion` any number of holders (D6 review finding 4). A child that seals a writer it filled
//! may still have a view that writes (S8), so its region is
//! filler-trusted; when the record wants it `immutable` the broker copies
//! it once into a section of its own, `SealedByBroker` (review Q1).
//!
//! Pure code, proved by Kani ([`relay_rule`]).

use super::accept::Proc;
use crate::ipc::common::codec::WireKind;

/// The broker's process number.
pub(crate) const BROKER: Proc = 0;

/// `FILE_MAP_READ | FILE_MAP_WRITE` and `FILE_MAP_READ` (values from the
/// `MapViewOfFile` page).
pub(crate) const ACCESS_RW: u32 = 0x6;
pub(crate) const ACCESS_READ: u32 = 0x4;

/// A section's state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum State {
    /// A writer exists: held by this process, or in a ticket (`None`).
    WriterOut(Option<Proc>),
    /// The broker sealed it with no view of its own left: unchanging.
    SealedByBroker,
    /// A sandboxed process sealed it: its own views may still write.
    SealedByChild,
    /// A producer in this process writes it.
    Producer(Proc),
}

/// What the kernel says about a pulled handle claimed as a region.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Pulled {
    /// Its type is "Section".
    pub(crate) is_section: bool,
    /// One of the broker's own sections.
    pub(crate) ours: bool,
    /// Its exact granted access.
    pub(crate) access: u32,
    /// Handles to the section in every process, the broker's own
    /// included.
    pub(crate) handle_count: u32,
}

/// What a relay of one region does.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Outcome {
    /// The section's next state.
    pub(crate) next: State,
    /// The receiver must treat it as filler-trusted (not immutable).
    pub(crate) filler_trusted: bool,
    /// The broker sends a copy it makes now instead (`SealedByBroker`).
    pub(crate) copy: bool,
}

/// Why a region relay was refused.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum RegionRefusal {
    /// Not a section.
    NotSection,
    /// A section the broker did not make.
    NotOurs,
    /// Not exactly the access the kind gives.
    Access,
    /// The declared length is not the registered size.
    Size,
    /// A writer from a process other than the broker (review M1).
    WriterNotFromBroker,
    /// Another handle to it would remain (a writer's copy kept).
    KeptCopy,
    /// The kind does not fit the section's state, or the relayer is not
    /// its holder.
    State,
}

/// Handles a section has while exactly one process holds it: the
/// broker's own and the one pulled.
const SOLE: u32 = 2;

/// Decides one region relay by `relayer` claiming `kind` with declared
/// length `len` (`want`: the record wants it `immutable`), for a section
/// of `size` bytes in `state`.
///
/// # Errors
/// The [`RegionRefusal`]; the broker then refuses the whole relay.
pub(crate) fn relay_rule(
    kind: WireKind,
    len: u32,
    want: bool,
    relayer: Proc,
    f: &Pulled,
    size: u32,
    state: State,
) -> Result<Outcome, RegionRefusal> {
    if !f.is_section {
        return Err(RegionRefusal::NotSection);
    }
    if !f.ours {
        return Err(RegionRefusal::NotOurs);
    }
    let access = match kind {
        WireKind::RegionWriter => ACCESS_RW,
        WireKind::ReadOnlyRegion | WireKind::ConsumerRegion => ACCESS_READ,
        WireKind::Sender | WireKind::BoundSender | WireKind::Receiver => {
            return Err(RegionRefusal::NotSection);
        }
    };
    if f.access != access {
        return Err(RegionRefusal::Access);
    }
    if len != size {
        return Err(RegionRefusal::Size);
    }
    let ok = |next: State, filler_trusted: bool, copy: bool| {
        Ok(Outcome {
            next,
            filler_trusted,
            copy,
        })
    };
    match (kind, state) {
        (WireKind::RegionWriter, _) if relayer != BROKER => Err(RegionRefusal::WriterNotFromBroker),
        (WireKind::RegionWriter, State::WriterOut(Some(BROKER))) => {
            if f.handle_count != SOLE {
                return Err(RegionRefusal::KeptCopy);
            }
            ok(State::WriterOut(None), false, false)
        }
        (WireKind::ReadOnlyRegion, State::SealedByBroker) => {
            ok(State::SealedByBroker, false, false)
        }
        (WireKind::ReadOnlyRegion, State::SealedByChild) => ok(State::SealedByChild, !want, want),
        (WireKind::ReadOnlyRegion, State::WriterOut(Some(p)))
            if p == relayer && relayer != BROKER =>
        {
            if f.handle_count != SOLE {
                return Err(RegionRefusal::KeptCopy);
            }
            ok(State::SealedByChild, !want, want)
        }
        (WireKind::ConsumerRegion, State::WriterOut(Some(p))) if p == relayer => {
            ok(State::Producer(p), true, false)
        }
        (WireKind::ConsumerRegion, State::Producer(p)) => ok(State::Producer(p), true, false),
        _ => Err(RegionRefusal::State),
    }
}

/// One region in a ticket: the registry entry, how it travels, and the
/// flags the receiver is given.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Grant {
    /// The registry id.
    pub(crate) id: u64,
    /// The kind it travels as.
    pub(crate) kind: WireKind,
    /// Its exact length.
    pub(crate) len: u32,
    /// Filler-trusted: never attested immutable by the receiver.
    pub(crate) filler_trusted: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    const SECTION: Pulled = Pulled {
        is_section: true,
        ours: true,
        access: ACCESS_READ,
        handle_count: SOLE,
    };
    const W: Pulled = Pulled {
        access: ACCESS_RW,
        ..SECTION
    };

    #[test]
    fn a_consumer_sent_as_read_only_is_refused() {
        let r = relay_rule(
            WireKind::ReadOnlyRegion,
            4096,
            false,
            3,
            &SECTION,
            4096,
            State::Producer(3),
        );
        assert_eq!(r, Err(RegionRefusal::State));
        let r = relay_rule(
            WireKind::ConsumerRegion,
            4096,
            false,
            3,
            &SECTION,
            4096,
            State::Producer(3),
        );
        assert_eq!(r.map(|o| o.next), Ok(State::Producer(3)));
    }

    /// Any holder passes a consumer on; the producer stays the producer.
    /// Only the producer sends the first one, while the section is still
    /// its writer.
    #[test]
    fn any_holder_forwards_a_consumer() {
        let c = WireKind::ConsumerRegion;
        for relayer in [BROKER, 3, 5] {
            let r = relay_rule(c, 4096, false, relayer, &SECTION, 4096, State::Producer(3));
            assert_eq!(
                r.map(|o| (o.next, o.filler_trusted)),
                Ok((State::Producer(3), true))
            );
        }
        let first = State::WriterOut(Some(3));
        assert_eq!(
            relay_rule(c, 4096, false, 3, &SECTION, 4096, first).map(|o| o.next),
            Ok(State::Producer(3))
        );
        assert_eq!(
            relay_rule(c, 4096, false, 5, &SECTION, 4096, first),
            Err(RegionRefusal::State)
        );
    }

    #[test]
    fn a_writer_only_from_the_broker_and_alone() {
        let r = relay_rule(
            WireKind::RegionWriter,
            4096,
            false,
            BROKER,
            &W,
            4096,
            State::WriterOut(Some(BROKER)),
        );
        assert_eq!(r.map(|o| o.next), Ok(State::WriterOut(None)));
        let r = relay_rule(
            WireKind::RegionWriter,
            4096,
            false,
            2,
            &W,
            4096,
            State::WriterOut(Some(2)),
        );
        assert_eq!(r, Err(RegionRefusal::WriterNotFromBroker));
        let kept = Pulled {
            handle_count: 3,
            ..W
        };
        let r = relay_rule(
            WireKind::RegionWriter,
            4096,
            false,
            BROKER,
            &kept,
            4096,
            State::WriterOut(Some(BROKER)),
        );
        assert_eq!(r, Err(RegionRefusal::KeptCopy));
    }

    #[test]
    fn a_child_seal_is_filler_trusted_unless_copied() {
        let w = State::WriterOut(Some(2));
        let r = relay_rule(WireKind::ReadOnlyRegion, 4096, false, 2, &SECTION, 4096, w).unwrap();
        assert_eq!(
            (r.next, r.filler_trusted, r.copy),
            (State::SealedByChild, true, false)
        );
        let r = relay_rule(WireKind::ReadOnlyRegion, 4096, true, 2, &SECTION, 4096, w).unwrap();
        assert_eq!((r.filler_trusted, r.copy), (false, true));
        // Its writer handle kept: refused.
        let kept = Pulled {
            handle_count: 3,
            ..SECTION
        };
        assert_eq!(
            relay_rule(WireKind::ReadOnlyRegion, 4096, false, 2, &kept, 4096, w),
            Err(RegionRefusal::KeptCopy)
        );
        // Another process's writer.
        assert_eq!(
            relay_rule(WireKind::ReadOnlyRegion, 4096, false, 5, &SECTION, 4096, w),
            Err(RegionRefusal::State)
        );
    }

    #[test]
    fn length_access_and_identity_are_exact() {
        let st = State::SealedByBroker;
        let ro = WireKind::ReadOnlyRegion;
        assert_eq!(
            relay_rule(ro, 4095, false, 2, &SECTION, 4096, st),
            Err(RegionRefusal::Size)
        );
        assert_eq!(
            relay_rule(ro, 4096, false, 2, &W, 4096, st),
            Err(RegionRefusal::Access)
        );
        let theirs = Pulled {
            ours: false,
            ..SECTION
        };
        assert_eq!(
            relay_rule(ro, 4096, false, 2, &theirs, 4096, st),
            Err(RegionRefusal::NotOurs)
        );
        let pipe = Pulled {
            is_section: false,
            ..SECTION
        };
        assert_eq!(
            relay_rule(ro, 4096, false, 2, &pipe, 4096, st),
            Err(RegionRefusal::NotSection)
        );
        let r = relay_rule(ro, 4096, false, 2, &SECTION, 4096, st).unwrap();
        assert!(!r.filler_trusted && !r.copy);
    }
}

#[cfg(kani)]
mod proofs {
    use super::*;

    fn any_state() -> State {
        match kani::any::<u8>() % 4 {
            0 => State::WriterOut(if kani::any() { Some(kani::any()) } else { None }),
            1 => State::SealedByBroker,
            2 => State::SealedByChild,
            _ => State::Producer(kani::any()),
        }
    }

    /// D6 review B1 and minor 3: for every claim, fact, relayer and
    /// state, an accepted region relay has exactly the registered size
    /// and its kind's access, comes from the broker if it is a writer,
    /// never turns a producer's section into a read-only region, is
    /// attested (not filler-trusted) only when the broker sealed it or
    /// copies it, and leaves a writer alone with its holder.
    #[kani::proof]
    fn the_registry_follows_its_state_machine() {
        let kind = match kani::any::<u8>() % 3 {
            0 => WireKind::RegionWriter,
            1 => WireKind::ReadOnlyRegion,
            _ => WireKind::ConsumerRegion,
        };
        let f = Pulled {
            is_section: kani::any(),
            ours: kani::any(),
            access: kani::any(),
            handle_count: kani::any(),
        };
        let (len, size, want, relayer): (u32, u32, bool, Proc) =
            (kani::any(), kani::any(), kani::any(), kani::any());
        let state = any_state();
        let got = relay_rule(kind, len, want, relayer, &f, size, state);
        if let Ok(o) = got {
            assert!(f.is_section && f.ours && len == size);
            match kind {
                WireKind::RegionWriter => {
                    assert!(relayer == BROKER && f.access == ACCESS_RW);
                    assert!(f.handle_count == SOLE);
                    assert!(o.next == State::WriterOut(None));
                }
                WireKind::ReadOnlyRegion => {
                    assert!(f.access == ACCESS_READ);
                    assert!(!matches!(state, State::Producer(_)));
                    assert!(o.filler_trusted == (state != State::SealedByBroker && !o.copy));
                    if o.copy {
                        assert!(want);
                    }
                }
                _ => {
                    // The section was its producer's writer, sent by that
                    // producer, or is a producer's already and stays so.
                    assert!(f.access == ACCESS_READ);
                    assert!(
                        (state == State::WriterOut(Some(relayer))
                            && o.next == State::Producer(relayer))
                            || (matches!(state, State::Producer(_)) && o.next == state)
                    );
                    assert!(o.filler_trusted);
                }
            }
        }
        kani::cover!(got.is_ok() && kind == WireKind::RegionWriter, "a writer");
        kani::cover!(got.is_ok_and(|o| o.copy), "a copy");
        kani::cover!(
            got.is_ok()
                && kind == WireKind::ConsumerRegion
                && state != State::Producer(relayer)
                && matches!(state, State::Producer(_)),
            "a consumer forwarded by another holder"
        );
        kani::cover!(
            got == Err(RegionRefusal::State) && matches!(state, State::Producer(_)),
            "a consumer as read-only"
        );
    }
}
