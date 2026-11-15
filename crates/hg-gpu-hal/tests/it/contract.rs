//! The contract, property by property.

use hg_gpu_hal::device::{Capabilities, DeviceLoss, Epoch, Validation, ValidationError};
use hg_gpu_hal::fence::Timeline;
use hg_gpu_hal::graph::{Access, Pass, RenderGraph};
use hg_gpu_hal::resource::{Format, ResourceId, State, Usage};

fn res(index: u32) -> ResourceId {
    ResourceId {
        device: 1,
        epoch: 0,
        index,
    }
}

/// A fence is a value on one timeline, and a value that has not been
/// reached is not complete. The property that matters is the one that
/// must never hold: a completed fence becoming incomplete.
#[test]
fn gh01_a_completed_fence_never_becomes_incomplete() {
    let mut t = Timeline::new(1);
    let a = t.submit();
    let b = t.submit();
    assert!(!t.is_complete(a));
    assert!(t.signal(1));
    assert!(t.is_complete(a));
    assert!(!t.is_complete(b));

    // A driver reporting an older value is ignored, not obeyed.
    assert!(!t.signal(0));
    assert!(t.is_complete(a), "still complete");
    // And a value beyond what was submitted is ignored too: nothing
    // has finished that was never started.
    assert!(!t.signal(99));
    assert!(!t.is_complete(b));

    assert!(t.signal(2));
    assert!(t.is_idle());
}

/// A fence from another device is never complete here. Answering
/// `true` would be a frame drawn from a texture another device is
/// still writing.
#[test]
fn gh02_a_foreign_fence_is_not_complete() {
    let mut a = Timeline::new(1);
    let mut b = Timeline::new(2);
    let fa = a.submit();
    let _ = b.submit();
    assert!(b.signal(1));
    assert!(!b.is_complete(fa), "a different device's timeline");
    assert_eq!(fa.device(), 1);
    assert_eq!(fa.value(), 1);
}

/// Validation is sticky: the first error is kept, later ones are
/// counted, and only a rebuild clears it.
#[test]
fn gh03_validation_is_sticky() {
    let mut v = Validation::new();
    assert!(v.is_clean());
    v.record(ValidationError::WrongState);
    v.record(ValidationError::AboveLimit);
    v.record(ValidationError::AboveLimit);
    assert_eq!(
        v.first(),
        Some(ValidationError::WrongState),
        "the first one"
    );
    assert_eq!(v.count(), 3);
    assert!(!v.is_clean());
    v.reset_for(Epoch(1));
    assert!(v.is_clean(), "only a rebuild clears it");
    assert_eq!(v.count(), 0);
}

/// Device loss resolves exactly once per epoch, and the epoch moves.
#[test]
fn gh04_device_loss_resolves_once() {
    let mut d = DeviceLoss::new();
    assert_eq!(d.resolve(), None, "nothing to resolve before it is lost");
    d.lose();
    d.lose();
    assert!(d.is_lost());
    assert_eq!(d.resolve(), Some(Epoch(1)), "one caller gets the rebuild");
    assert_eq!(d.resolve(), None, "and only one");
    assert!(!d.is_lost(), "rebuilt");
    assert_eq!(d.epoch(), Epoch(1));

    // Lost again: a new epoch, resolved once again.
    d.lose();
    assert_eq!(d.resolve(), Some(Epoch(2)));
    assert_eq!(d.resolve(), None);
}

/// Capabilities name no vendor. This is checked by what the type can
/// be asked, which is the only way a rule like this stays true: a
/// field added later would have to be a capability, because there is
/// nowhere else to put it.
#[test]
fn gh05_capabilities_name_no_vendor() {
    let c = Capabilities::NONE
        .with_integer_atomics(true)
        .with_timeline_semaphores(true)
        .with_concurrent_sampling(false)
        .with_max_texture_2d(16_384)
        .with_max_buffer_bytes(1 << 30)
        .with_max_workgroup([256, 256, 64]);
    assert!(c.can_raster_canonically());
    let shown = format!("{c:?}");
    for vendor in [
        "Intel", "AMD", "NVIDIA", "Apple", "Mali", "Adreno", "vendor",
    ] {
        assert!(
            !shown.contains(vendor),
            "a capability named {vendor}: {shown}"
        );
    }
    // Without integer atomics the canonical raster is the CPU's, and
    // the bits are the same either way (GR-D3).
    assert!(!c.with_integer_atomics(false).can_raster_canonically());
    // A small device fails the other half.
    assert!(!c.with_max_texture_2d(2_048).can_raster_canonically());
    // Nothing probed is nothing advertised.
    assert_eq!(Capabilities::NONE, Capabilities::default());
    assert!(!Capabilities::NONE.can_raster_canonically());
}

/// Two reads in a row need no barrier; everything else does. This is
/// the rule a hand-written back end gets wrong in the direction that
/// works on the machine it was written on.
#[test]
fn gh06_barriers_follow_from_the_states() {
    assert!(!State::ShaderRead.needs_barrier_to(State::ShaderRead));
    assert!(State::ShaderRead.needs_barrier_to(State::RenderTarget));
    assert!(State::RenderTarget.needs_barrier_to(State::ShaderRead));
    assert!(
        State::RenderTarget.needs_barrier_to(State::RenderTarget),
        "two writes have to be ordered, and the barrier is the order"
    );
    assert!(State::Undefined.needs_barrier_to(State::ShaderRead));
    assert!(State::CopySource.needs_barrier_to(State::CopyDestination));
}

/// The graph derives the barriers a sequence of passes needs.
#[test]
fn gh07_the_graph_derives_its_barriers() {
    let mut g = RenderGraph::new(1, 0);
    g.add(Pass::new(
        "raster",
        vec![Access {
            resource: res(7),
            usage: Usage::RenderTarget,
        }],
    ))
    .add(
        Pass::new(
            "composite",
            vec![Access {
                resource: res(7),
                usage: Usage::Sample,
            }],
        )
        .after(&[0]),
    )
    .add(
        Pass::new(
            "also reads",
            vec![Access {
                resource: res(7),
                usage: Usage::Sample,
            }],
        )
        // After 1, which is after 0, so this read is ordered against
        // the write transitively -- two reads need no edge between
        // them, but a read and a write do.
        .after(&[1]),
    );
    let (barriers, validation) = g.compile();
    assert!(validation.is_clean(), "{:?}", validation.first());
    assert_eq!(barriers.len(), 2, "create and write, then write to read");
    assert_eq!(barriers[0].from, State::Undefined);
    assert_eq!(barriers[0].to, State::RenderTarget);
    assert_eq!(barriers[1].from, State::RenderTarget);
    assert_eq!(barriers[1].to, State::ShaderRead);
    assert_eq!(barriers[1].before, 1);
}

/// Reading a resource nothing has written is caught before any
/// command reaches a driver. What is in it is whatever the driver
/// left there, which is a different image on every machine and
/// sometimes another page's.
#[test]
fn gh08_reading_what_nothing_wrote_is_an_error() {
    let mut g = RenderGraph::new(1, 0);
    g.add(Pass::new(
        "reads first",
        vec![Access {
            resource: res(3),
            usage: Usage::Sample,
        }],
    ));
    let (_, validation) = g.compile();
    assert_eq!(validation.first(), Some(ValidationError::UnwrittenRead));

    // The twin: the same read, after something wrote it.
    let mut ok = RenderGraph::new(1, 0);
    ok.add(Pass::new(
        "writes",
        vec![Access {
            resource: res(3),
            usage: Usage::RenderTarget,
        }],
    ))
    .add(
        Pass::new(
            "reads",
            vec![Access {
                resource: res(3),
                usage: Usage::Sample,
            }],
        )
        .after(&[0]),
    );
    assert!(ok.compile().1.is_clean());
}

/// A resource from another device or another epoch is foreign, which
/// is how a handle that outlived a device loss is caught rather than
/// used.
#[test]
fn gh09_a_stale_or_foreign_resource_is_caught() {
    for bad in [
        ResourceId {
            device: 2,
            epoch: 0,
            index: 1,
        },
        ResourceId {
            device: 1,
            epoch: 1,
            index: 1,
        },
    ] {
        let mut g = RenderGraph::new(1, 0);
        g.add(Pass::new(
            "uses it",
            vec![Access {
                resource: bad,
                usage: Usage::RenderTarget,
            }],
        ));
        let (barriers, validation) = g.compile();
        assert_eq!(
            validation.first(),
            Some(ValidationError::ForeignResource),
            "{bad:?}"
        );
        assert!(barriers.is_empty(), "nothing is emitted for it");
    }
}

/// Formats are integer only, and their sizes are what they say.
#[test]
fn gh10_formats_are_integer_only() {
    assert_eq!(Format::R8.bytes(), 1);
    assert_eq!(Format::R16.bytes(), 2);
    assert_eq!(Format::Rgba8.bytes(), 4);
}

/// **GR-M0 §3.5's decision, and what replaced the `Cycle` variant.**
///
/// The graph gained explicit edges, and the edges point **backwards**:
/// a pass may only depend on an earlier index. So a cycle stayed
/// unexpressible, and `Cycle` -- a variant nothing could produce when
/// the graph was a sequence, and still nothing can produce now --
/// was deleted rather than kept reserved. An unreachable refusal is a
/// refusal nobody has ever seen work.
///
/// What is checked instead is the edge itself, one at a time, without
/// looking at the graph.
#[test]
fn gh11_an_edge_pointing_forwards_is_refused() {
    for bad in [
        vec![1_usize], // a later pass
        vec![0_usize], // itself, at index 0
        vec![9_usize], // past the end
    ] {
        let mut g = RenderGraph::new(1, 0);
        g.add(
            Pass::new(
                "writes",
                vec![Access {
                    resource: res(1),
                    usage: Usage::RenderTarget,
                }],
            )
            .after(&bad),
        );
        let (_, validation) = g.compile();
        assert_eq!(
            validation.first(),
            Some(ValidationError::ForwardEdge),
            "{bad:?}"
        );
    }

    // The twin: a backward edge from pass 1 to pass 0 is clean, so
    // the refusals above are about direction and not about having an
    // edge at all.
    let mut ok = RenderGraph::new(1, 0);
    ok.add(Pass::new(
        "writes",
        vec![Access {
            resource: res(1),
            usage: Usage::RenderTarget,
        }],
    ))
    .add(
        Pass::new(
            "reads",
            vec![Access {
                resource: res(1),
                usage: Usage::Sample,
            }],
        )
        .after(&[0]),
    );
    assert!(ok.compile().1.is_clean(), "{:?}", ok.compile().1.first());
}

/// **The check the sequence could not have.**
///
/// A sequence orders every pass against every other, so it can never
/// say that two passes which share a resource were never ordered on
/// purpose. With edges they can be genuinely unordered, and an
/// unordered write is a result that depends on how a driver
/// schedules -- a different image on a different machine, which is
/// what GR-D3 exists to prevent.
#[test]
fn gh12_two_unordered_passes_sharing_a_resource_are_refused() {
    // Pass 0 writes, pass 1 reads, and nothing orders them.
    let mut g = RenderGraph::new(1, 0);
    g.add(Pass::new(
        "writes",
        vec![Access {
            resource: res(5),
            usage: Usage::RenderTarget,
        }],
    ))
    .add(Pass::new(
        "reads, unordered",
        vec![Access {
            resource: res(5),
            usage: Usage::Sample,
        }],
    ));
    let (_, validation) = g.compile();
    assert_eq!(validation.first(), Some(ValidationError::UnorderedConflict));

    // Two passes that only *read* are no conflict: neither changes
    // the resource, so the order between them cannot change the
    // result. Requiring an edge there would chain every pass that
    // samples one texture.
    let mut reads = RenderGraph::new(1, 0);
    reads
        .add(Pass::new(
            "writes",
            vec![Access {
                resource: res(5),
                usage: Usage::RenderTarget,
            }],
        ))
        .add(
            Pass::new(
                "reads",
                vec![Access {
                    resource: res(5),
                    usage: Usage::Sample,
                }],
            )
            .after(&[0]),
        )
        .add(
            Pass::new(
                "also reads",
                vec![Access {
                    resource: res(5),
                    usage: Usage::Sample,
                }],
            )
            .after(&[0]),
        );
    assert!(
        reads.compile().1.is_clean(),
        "{:?}",
        reads.compile().1.first()
    );

    // And two passes touching *different* resources need no edge at
    // all, which is the independence the sequence could not express.
    let mut independent = RenderGraph::new(1, 0);
    independent
        .add(Pass::new(
            "writes one",
            vec![Access {
                resource: res(1),
                usage: Usage::RenderTarget,
            }],
        ))
        .add(Pass::new(
            "writes another",
            vec![Access {
                resource: res(2),
                usage: Usage::RenderTarget,
            }],
        ));
    assert!(independent.compile().1.is_clean());
}

/// Ordering is **transitive**: a conflict two edges apart is ordered.
///
/// This is what makes the barrier walk correct rather than plausible.
/// Because edges point backwards, submission order is always a
/// topological order; because every conflicting pair is ordered by an
/// edge *path*, submission order agrees with dependency order wherever
/// it matters.
#[test]
fn gh13_ordering_is_transitive() {
    let mut g = RenderGraph::new(1, 0);
    g.add(Pass::new(
        "writes",
        vec![Access {
            resource: res(4),
            usage: Usage::RenderTarget,
        }],
    ))
    .add(Pass::new("unrelated", vec![]).after(&[0]))
    .add(
        Pass::new(
            "reads, two edges away",
            vec![Access {
                resource: res(4),
                usage: Usage::Sample,
            }],
        )
        .after(&[1]),
    );
    assert!(g.compile().1.is_clean(), "{:?}", g.compile().1.first());

    // The twin: the same three passes with the middle edge removed
    // leaves the read unordered against the write.
    let mut broken = RenderGraph::new(1, 0);
    broken
        .add(Pass::new(
            "writes",
            vec![Access {
                resource: res(4),
                usage: Usage::RenderTarget,
            }],
        ))
        .add(Pass::new("unrelated", vec![]))
        .add(
            Pass::new(
                "reads, no path to the write",
                vec![Access {
                    resource: res(4),
                    usage: Usage::Sample,
                }],
            )
            .after(&[1]),
        );
    assert_eq!(
        broken.compile().1.first(),
        Some(ValidationError::UnorderedConflict)
    );
}
