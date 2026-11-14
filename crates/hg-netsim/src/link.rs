//! A one-way link: delay, bandwidth, loss, reordering and an MTU.
//!
//! Every draw comes from the caller's seeded stream, so a run is a
//! function of its seed. The link holds no queue and starts no timer;
//! it answers "when does this arrive, if at all", and the caller's
//! event loop does the rest.

use hg_substrate::RandomStream;
use hg_types::{Duration, Instant};

/// What a link does to a packet.
///
/// Every field is a plain number rather than a distribution, because a
/// simulation whose parameters are distributions is one whose failures
/// cannot be described in a bug report. A caller that wants a
/// distribution varies the spec between runs.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LinkSpec {
    /// One-way propagation delay.
    pub delay: Duration,
    /// How much `delay` may vary, uniformly, in either direction.
    pub jitter: Duration,
    /// Bytes per second, or `None` for a link that does not serialize.
    pub bandwidth: Option<u64>,
    /// Loss, in parts per million, so the field is exact.
    pub loss_ppm: u32,
    /// Reordering, in parts per million: a packet that is reordered
    /// arrives one `delay` later than it would have.
    pub reorder_ppm: u32,
    /// The largest packet the link carries. A larger one is dropped,
    /// as a path without fragmentation drops it.
    pub mtu: usize,
}

impl LinkSpec {
    /// A link that delivers everything, immediately and in order, with
    /// a 1,500-byte MTU. The starting point a test varies from.
    #[must_use]
    pub const fn perfect() -> Self {
        Self {
            delay: Duration::ZERO,
            jitter: Duration::ZERO,
            bandwidth: None,
            loss_ppm: 0,
            reorder_ppm: 0,
            mtu: 1_500,
        }
    }

    /// A typical wired path: 20 ms each way, no loss.
    #[must_use]
    pub const fn wired() -> Self {
        Self {
            delay: Duration::from_millis(20),
            jitter: Duration::from_millis(1),
            bandwidth: Some(12_500_000),
            loss_ppm: 0,
            reorder_ppm: 0,
            mtu: 1_500,
        }
    }

    /// A congested mobile path: 80 ms, 1% loss, visible reordering and
    /// a smaller MTU.
    #[must_use]
    pub const fn mobile() -> Self {
        Self {
            delay: Duration::from_millis(80),
            jitter: Duration::from_millis(30),
            bandwidth: Some(250_000),
            loss_ppm: 10_000,
            reorder_ppm: 20_000,
            mtu: 1_400,
        }
    }
}

/// What happened to one packet.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Delivery {
    /// It arrives at this instant.
    At(Instant),
    /// It was lost.
    Lost,
    /// It was larger than the MTU, so the path dropped it. Reported
    /// apart from [`Self::Lost`] because a protocol should discover
    /// this one and lower its packet size, and a test that cannot tell
    /// the two apart cannot check that it did.
    TooLarge {
        /// How many bytes were offered.
        bytes: usize,
        /// The link's MTU.
        mtu: usize,
    },
}

/// One direction of one path.
#[derive(Debug, Clone)]
pub struct Link {
    spec: LinkSpec,
    /// When the link is next free to begin serializing, which is what
    /// makes bandwidth a queue rather than a per-packet delay.
    busy_until: Instant,
    sent: u64,
    delivered: u64,
    lost: u64,
    oversize: u64,
    reordered: u64,
}

impl Link {
    /// A link with `spec`, idle at the clock's origin.
    #[must_use]
    pub const fn new(spec: LinkSpec) -> Self {
        Self {
            spec,
            busy_until: Instant::ORIGIN,
            sent: 0,
            delivered: 0,
            lost: 0,
            oversize: 0,
            reordered: 0,
        }
    }

    /// Its spec.
    #[must_use]
    pub const fn spec(&self) -> LinkSpec {
        self.spec
    }

    /// Packets offered, delivered, lost, dropped for size, and
    /// reordered.
    #[must_use]
    pub const fn counts(&self) -> (u64, u64, u64, u64, u64) {
        (
            self.sent,
            self.delivered,
            self.lost,
            self.oversize,
            self.reordered,
        )
    }

    /// Offers `bytes` bytes at `now`.
    ///
    /// The order of the draws is fixed — size, loss, reorder, jitter —
    /// and every packet makes all of them that its path reaches, so a
    /// change to one parameter does not move the others' sequence.
    pub fn send(&mut self, now: Instant, bytes: usize, rng: &mut dyn RandomStream) -> Delivery {
        self.sent = self.sent.saturating_add(1);

        if bytes > self.spec.mtu {
            self.oversize = self.oversize.saturating_add(1);
            return Delivery::TooLarge {
                bytes,
                mtu: self.spec.mtu,
            };
        }

        if draw_ppm(rng, self.spec.loss_ppm) {
            self.lost = self.lost.saturating_add(1);
            return Delivery::Lost;
        }

        let reordered = draw_ppm(rng, self.spec.reorder_ppm);
        let travel_base = draw_delay(rng, self.spec.delay, self.spec.jitter);

        // Serialization: the link carries one packet at a time, so a
        // packet offered while the link is busy begins when it frees.
        let begins = if self.busy_until > now {
            self.busy_until
        } else {
            now
        };
        let serialize = match self.spec.bandwidth {
            None => Duration::ZERO,
            Some(0) => Duration::ZERO,
            Some(rate) => {
                let nanos = u64::try_from(bytes)
                    .unwrap_or(u64::MAX)
                    .saturating_mul(1_000_000_000)
                    .checked_div(rate)
                    .unwrap_or(0);
                Duration::from_nanos(nanos)
            }
        };
        self.busy_until = begins.checked_add(serialize).unwrap_or(begins);

        let mut travel = travel_base;
        if reordered {
            self.reordered = self.reordered.saturating_add(1);
            // A reordered packet arrives one whole delay later, so it
            // lands behind the packets that followed it.
            travel = travel.saturating_add(self.spec.delay);
        }
        let arrival = self
            .busy_until
            .checked_add(travel)
            .unwrap_or(self.busy_until);
        self.delivered = self.delivered.saturating_add(1);
        Delivery::At(arrival)
    }
}

/// Draws a yes with probability `ppm` in a million.
///
/// `ppm` above a million is a link that drops everything, which is a
/// useful thing to be able to say.
fn draw_ppm(rng: &mut dyn RandomStream, ppm: u32) -> bool {
    if ppm == 0 {
        // Still a draw, so that turning loss off does not shift every
        // later draw in the run.
        let _ = rng.next_u64();
        return false;
    }
    let v = rng.next_u64().checked_rem(1_000_000).unwrap_or(0);
    v < u64::from(ppm)
}

/// Draws a delay in `[delay - jitter, delay + jitter]`, clamped at
/// zero.
///
/// The draw is over the whole span and the shift is applied here,
/// rather than returning an amount to add: returning an unsigned
/// amount would make every jitter positive, which is a link that is
/// only ever slower than its delay and never faster -- a different
/// network from the one the spec describes, and one whose mean
/// latency is half a jitter too high.
fn draw_delay(rng: &mut dyn RandomStream, delay: Duration, jitter: Duration) -> Duration {
    let span = jitter.as_nanos();
    if span == 0 {
        // Still a draw, so that turning jitter off does not shift every
        // later draw in the run.
        let _ = rng.next_u64();
        return delay;
    }
    let span = u64::try_from(span).unwrap_or(u64::MAX);
    let full = span.saturating_mul(2).saturating_add(1);
    let drawn = rng.next_u64().checked_rem(full).unwrap_or(0);
    if drawn >= span {
        delay.saturating_add(Duration::from_nanos(drawn.saturating_sub(span)))
    } else {
        delay.saturating_sub(Duration::from_nanos(span.saturating_sub(drawn)))
    }
}
