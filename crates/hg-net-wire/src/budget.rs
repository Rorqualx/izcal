//! The budget every state machine is charged against (§6.2).
//!
//! # What a budget is for
//!
//! A protocol state machine holds things a peer chose: header bytes
//! being decoded, reassembly gaps, ACK ranges, connection IDs, queued
//! replies. Each is memory the peer spends on our behalf, and none of
//! them is bounded by the protocol alone. The budget is where that
//! spending is counted, and §6.2 fixes **when**: at allocation time,
//! before the bytes exist, never after.
//!
//! The difference matters. Charging after the fact bounds the steady
//! state and not the peak, so a peer that allocates and releases in a
//! loop stays inside a budget it has already exceeded. Charging first
//! means the refusal happens while there is still nothing to refuse.
//!
//! # What a failed charge is
//!
//! [`Overrun`], a value — "a charge that fails is a typed stream or
//! connection error, never a panic" (§6.2). Which of the two it
//! becomes is the protocol's decision, not this crate's: an HTTP/2
//! header-block overrun closes a stream and a connection-ID overrun
//! closes the connection, and nothing here can tell them apart.
//!
//! # Rolling up
//!
//! "Per-connection budgets roll up into per-partition and per-service
//! budgets." A charge that succeeds against the connection and fails
//! against the service must leave *both* untouched, or the connection
//! has silently paid for memory it never got. [`charge_all`] is that
//! operation, and it is the only way to charge more than one.

use core::fmt;

/// Which limit a charge was made against.
///
/// Closed: a new limit is a new variant and a new row in
/// [`limits::ALL`], so NW-15's count cannot drift from the enum.
///
/// [`limits::ALL`]: crate::limits::ALL
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Limit {
    /// [`limits::HEADER_BLOCK_BYTES`](crate::limits::HEADER_BLOCK_BYTES).
    HeaderBlockBytes,
    /// [`limits::CONTINUATION_BYTES`](crate::limits::CONTINUATION_BYTES).
    ContinuationBytes,
    /// [`limits::REASSEMBLY_GAPS`](crate::limits::REASSEMBLY_GAPS).
    ReassemblyGaps,
    /// [`limits::ACK_RANGES`](crate::limits::ACK_RANGES).
    AckRanges,
    /// [`limits::ACTIVE_CONNECTION_IDS`](crate::limits::ACTIVE_CONNECTION_IDS).
    ActiveConnectionIds,
    /// [`limits::PENDING_PATH_RESPONSES`](crate::limits::PENDING_PATH_RESPONSES).
    PendingPathResponses,
    /// [`limits::PENDING_RETIRE_CONNECTION_IDS`](crate::limits::PENDING_RETIRE_CONNECTION_IDS).
    PendingRetireConnectionIds,
    /// [`limits::QUEUED_CONTROL_REPLIES`](crate::limits::QUEUED_CONTROL_REPLIES).
    QueuedControlReplies,
    /// [`limits::DECOMPRESSED_BYTES`](crate::limits::DECOMPRESSED_BYTES).
    DecompressedBytes,
    /// [`limits::PARTITION_BYTES`](crate::limits::PARTITION_BYTES).
    PartitionBytes,
    /// [`limits::SERVICE_BYTES`](crate::limits::SERVICE_BYTES).
    ServiceBytes,
}

impl Limit {
    /// Its name, as an error reports it.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::HeaderBlockBytes => "header-block bytes",
            Self::ContinuationBytes => "CONTINUATION bytes",
            Self::ReassemblyGaps => "reassembly gaps",
            Self::AckRanges => "ACK ranges",
            Self::ActiveConnectionIds => "active connection IDs",
            Self::PendingPathResponses => "pending PATH_RESPONSE frames",
            Self::PendingRetireConnectionIds => "pending RETIRE_CONNECTION_ID frames",
            Self::QueuedControlReplies => "queued control replies",
            Self::DecompressedBytes => "decompressed bytes",
            Self::PartitionBytes => "partition bytes",
            Self::ServiceBytes => "service bytes",
        }
    }
}

impl fmt::Display for Limit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.name())
    }
}

/// A charge that would have exceeded its budget.
///
/// It carries what was asked and what was left, because a protocol
/// that reports only "over budget" gives its operator no way to tell a
/// limit that is too low from a peer that is hostile.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Overrun {
    /// Which limit.
    pub limit: Limit,
    /// How much the charge asked for.
    pub requested: u64,
    /// How much was left.
    pub remaining: u64,
}

impl fmt::Display for Overrun {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}: {} requested, {} left",
            self.limit, self.requested, self.remaining
        )
    }
}

impl core::error::Error for Overrun {}

/// A counter with a cap, charged before the memory exists.
///
/// Every operation is total: `charge` returns [`Overrun`] rather than
/// wrapping or panicking, and `release` saturates at zero rather than
/// going negative, because a double release is a bug in the caller and
/// an underflow here would turn it into an unbounded budget.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Budget {
    limit: Limit,
    cap: u64,
    used: u64,
}

impl Budget {
    /// A budget of `cap` against `limit`, with nothing charged.
    #[must_use]
    pub const fn new(limit: Limit, cap: u64) -> Self {
        Self {
            limit,
            cap,
            used: 0,
        }
    }

    /// Which limit it enforces.
    #[must_use]
    pub const fn limit(&self) -> Limit {
        self.limit
    }

    /// Its cap.
    #[must_use]
    pub const fn cap(&self) -> u64 {
        self.cap
    }

    /// How much is charged.
    #[must_use]
    pub const fn used(&self) -> u64 {
        self.used
    }

    /// How much is left. Never negative: `used` never exceeds `cap`,
    /// which is the invariant `charge` maintains.
    #[must_use]
    pub const fn remaining(&self) -> u64 {
        self.cap.saturating_sub(self.used)
    }

    /// Whether a charge of `n` would be refused, without making it.
    ///
    /// For the caller that must decide between two sizes before
    /// committing to either. It is not a reservation: nothing stops
    /// another charge in between, which is why `charge` checks again.
    #[must_use]
    pub const fn would_fit(&self, n: u64) -> bool {
        n <= self.remaining()
    }

    /// Charges `n`.
    ///
    /// # Errors
    /// [`Overrun`] when `n` is more than is left. Nothing is charged
    /// when it fails, so a refused charge leaves the budget exactly as
    /// it was.
    pub const fn charge(&mut self, n: u64) -> Result<(), Overrun> {
        if n > self.remaining() {
            return Err(Overrun {
                limit: self.limit,
                requested: n,
                remaining: self.remaining(),
            });
        }
        // Cannot overflow: `n <= cap - used`, so `used + n <= cap`.
        self.used = self.used.saturating_add(n);
        Ok(())
    }

    /// Releases `n`, saturating at zero.
    ///
    /// Releasing more than was charged is a bug in the caller. It is
    /// saturated rather than refused because the alternative — an
    /// error nobody can act on at a release site — is routinely
    /// ignored, and an ignored release underflow is an unbounded
    /// budget.
    pub const fn release(&mut self, n: u64) {
        self.used = self.used.saturating_sub(n);
    }

    /// Releases everything.
    pub const fn clear(&mut self) {
        self.used = 0;
    }
}

/// Charges `n` against every budget in `chain`, or none of them.
///
/// The roll-up of §6.2: a connection's budget, then its partition's,
/// then its service's. If any one refuses, the ones already charged
/// are released and the [`Overrun`] names the one that refused — so a
/// caller never ends up holding a charge for memory it did not get.
///
/// # Errors
/// [`Overrun`] from the first budget that refuses.
pub fn charge_all(chain: &mut [&mut Budget], n: u64) -> Result<(), Overrun> {
    let mut charged = 0_usize;
    for b in chain.iter_mut() {
        match b.charge(n) {
            Ok(()) => charged = charged.saturating_add(1),
            Err(e) => {
                // Roll back exactly what was charged, in any order:
                // release is commutative and each took the same `n`.
                for done in chain.iter_mut().take(charged) {
                    done.release(n);
                }
                return Err(e);
            }
        }
    }
    Ok(())
}

/// Releases `n` from every budget in `chain`.
pub fn release_all(chain: &mut [&mut Budget], n: u64) {
    for b in chain.iter_mut() {
        b.release(n);
    }
}
