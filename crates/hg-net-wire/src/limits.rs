//! Every limit, in one module (NW-15).
//!
//! NW-15 is a T0 target: "100% of limits in one module, each tested at
//! limit and limit+1". One module is the measurable part — a limit
//! written inline in the crate that enforces it cannot be reviewed as a
//! set, and a crawl cannot validate what it cannot enumerate.
//!
//! **Every value here is a placeholder†.** §6.2: they "start from the
//! security note's proposals (R3–R15) and are validated against a crawl
//! before freezing (Q-NW-9)". A placeholder that is enforced and
//! measured is worth more than an unbounded buffer, and marking it is
//! what keeps it from being mistaken for a measurement.
//!
//! Each constant carries the [`Limit`] it is enforced as, so an
//! [`Overrun`] names the limit rather than a number.
//!
//! [`Limit`]: crate::Limit
//! [`Overrun`]: crate::Overrun

use crate::budget::Limit;

/// Header-block bytes a peer may make one connection hold **while
/// decoding** (HPACK, QPACK). Charged during decoding and never after,
/// so a compression bomb is refused at the point it expands rather
/// than after it is stored (§6.2).
pub const HEADER_BLOCK_BYTES: u64 = 256 * 1024;

/// Bytes of CONTINUATION frames one HTTP/2 header block may carry
/// before the connection is an error. The 2024 CONTINUATION flood
/// worked because the frames were free until the block ended.
pub const CONTINUATION_BYTES: u64 = 64 * 1024;

/// Reassembly gaps one QUIC or HTTP/3 stream may hold. Each gap is a
/// range a peer chose not to send, so the count is a peer-controlled
/// object, not a byte count.
pub const REASSEMBLY_GAPS: u64 = 64;

/// ACK ranges one QUIC connection may hold.
pub const ACK_RANGES: u64 = 256;

/// Connection IDs one QUIC connection may hold for its peer, which
/// RFC 9000 §18.2 bounds by `active_connection_id_limit`; this is the
/// value Izcal advertises.
pub const ACTIVE_CONNECTION_IDS: u64 = 8;

/// Pending `PATH_RESPONSE` frames one connection may queue.
pub const PENDING_PATH_RESPONSES: u64 = 8;

/// Pending `RETIRE_CONNECTION_ID` frames one connection may queue.
pub const PENDING_RETIRE_CONNECTION_IDS: u64 = 16;

/// Queued control replies (`SETTINGS` acknowledgements, `PING`
/// responses, `WINDOW_UPDATE`s) one connection may hold.
pub const QUEUED_CONTROL_REPLIES: u64 = 64;

/// Bytes one response body may expand to from a content coding before
/// the decoder refuses. §6.2 charges decompressed bytes at allocation
/// time, which is what makes a zip bomb a typed error.
pub const DECOMPRESSED_BYTES: u64 = 64 * 1024 * 1024;

/// Bytes one partition service may hold across all its connections.
/// Per-connection budgets roll up into this (§6.2).
pub const PARTITION_BYTES: u64 = 16 * 1024 * 1024;

/// Bytes one network service may hold across all its partitions.
pub const SERVICE_BYTES: u64 = 64 * 1024 * 1024;

/// Every limit, with the name an [`Overrun`] reports and the value it
/// holds. The order is the order above.
///
/// This is what NW-15 counts: a limit that is not in this table is not
/// in the module, and the test that exercises each at limit and at
/// limit + 1 walks this table rather than a list written beside it.
///
/// [`Overrun`]: crate::Overrun
pub const ALL: &[(Limit, u64)] = &[
    (Limit::HeaderBlockBytes, HEADER_BLOCK_BYTES),
    (Limit::ContinuationBytes, CONTINUATION_BYTES),
    (Limit::ReassemblyGaps, REASSEMBLY_GAPS),
    (Limit::AckRanges, ACK_RANGES),
    (Limit::ActiveConnectionIds, ACTIVE_CONNECTION_IDS),
    (Limit::PendingPathResponses, PENDING_PATH_RESPONSES),
    (
        Limit::PendingRetireConnectionIds,
        PENDING_RETIRE_CONNECTION_IDS,
    ),
    (Limit::QueuedControlReplies, QUEUED_CONTROL_REPLIES),
    (Limit::DecompressedBytes, DECOMPRESSED_BYTES),
    (Limit::PartitionBytes, PARTITION_BYTES),
    (Limit::ServiceBytes, SERVICE_BYTES),
];
