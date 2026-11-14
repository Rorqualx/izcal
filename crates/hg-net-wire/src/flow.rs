//! QUIC flow control (RFC 9000 §4).
//!
//! Two numbers per direction: how much the sender has sent, and the
//! most the receiver has said it may send. The arithmetic is small and
//! every piece of it reads a peer-supplied value.
//!
//! Three rules from §4 are enforced here rather than left to each
//! caller, because each has been a vulnerability somewhere:
//!
//! - **A limit never decreases** (§4.1): "A receiver MUST close the
//!   connection with an error of type `FLOW_CONTROL_ERROR` if the
//!   sender violates the advertised … limits", and a `MAX_DATA` frame
//!   that lowers the limit "MUST be ignored". Ignoring is not the same
//!   as accepting, and a window that could move down is a window that
//!   can be made to wrap.
//! - **A stream's final size never changes** (§4.5): two different
//!   final sizes for one stream is a `FINAL_SIZE_ERROR`.
//! - **Sending beyond the limit is an error, not a clamp** (§4.1).
//!
//! Nothing here allocates or blocks; it is arithmetic over two `u64`s,
//! proved total by Kani for every pair (G33).

/// Why a flow-control operation was refused.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum FlowError {
    /// The peer sent past the limit it was given
    /// (RFC 9000 §4.1: `FLOW_CONTROL_ERROR`).
    Exceeded {
        /// What was already consumed plus what arrived.
        would_be: u64,
        /// The limit in force.
        limit: u64,
    },
    /// A stream's final size changed, or data arrived past a final
    /// size already known (RFC 9000 §4.5: `FINAL_SIZE_ERROR`).
    FinalSize {
        /// The final size already recorded.
        known: u64,
        /// The size now claimed.
        claimed: u64,
    },
}

/// One direction of one flow-control window.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Window {
    consumed: u64,
    limit: u64,
    final_size: Option<u64>,
}

impl Window {
    /// A window with an initial limit and nothing consumed.
    #[must_use]
    pub const fn new(initial_limit: u64) -> Self {
        Self {
            consumed: 0,
            limit: initial_limit,
            final_size: None,
        }
    }

    /// How much has been consumed.
    #[must_use]
    pub const fn consumed(&self) -> u64 {
        self.consumed
    }

    /// The limit in force.
    #[must_use]
    pub const fn limit(&self) -> u64 {
        self.limit
    }

    /// How much may still be sent. Zero when the limit is reached;
    /// never negative, because [`Self::consume`] never passes it.
    #[must_use]
    pub const fn available(&self) -> u64 {
        self.limit.saturating_sub(self.consumed)
    }

    /// The final size, once it is known.
    #[must_use]
    pub const fn final_size(&self) -> Option<u64> {
        self.final_size
    }

    /// Raises the limit, or ignores a frame that would lower it.
    ///
    /// Returns whether the limit moved. §4.1: a `MAX_DATA` that does
    /// not increase the limit "MUST be ignored", so this is not an
    /// error — a retransmitted frame arriving out of order is ordinary
    /// and would otherwise close the connection.
    pub const fn raise_limit(&mut self, new_limit: u64) -> bool {
        if new_limit > self.limit {
            self.limit = new_limit;
            true
        } else {
            false
        }
    }

    /// Consumes `n` more bytes.
    ///
    /// # Errors
    /// [`FlowError::Exceeded`] when `n` would pass the limit, and
    /// [`FlowError::FinalSize`] when it would pass a final size
    /// already known. Nothing is consumed when it fails.
    pub const fn consume(&mut self, n: u64) -> Result<(), FlowError> {
        let would_be = match self.consumed.checked_add(n) {
            Some(v) => v,
            // Only reachable past 2^64 bytes, which the limit check
            // below would refuse anyway; reported as the same error so
            // a caller has one case to handle.
            None => {
                return Err(FlowError::Exceeded {
                    would_be: u64::MAX,
                    limit: self.limit,
                });
            }
        };
        if let Some(fin) = self.final_size
            && would_be > fin
        {
            return Err(FlowError::FinalSize {
                known: fin,
                claimed: would_be,
            });
        }
        if would_be > self.limit {
            return Err(FlowError::Exceeded {
                would_be,
                limit: self.limit,
            });
        }
        self.consumed = would_be;
        Ok(())
    }

    /// Records the stream's final size.
    ///
    /// # Errors
    /// [`FlowError::FinalSize`] when a different final size is already
    /// known, or when more has been consumed than this size allows;
    /// [`FlowError::Exceeded`] when the final size is past the limit,
    /// which §4.5 also makes a flow-control error.
    pub const fn set_final_size(&mut self, size: u64) -> Result<(), FlowError> {
        if let Some(known) = self.final_size {
            if known == size {
                return Ok(());
            }
            return Err(FlowError::FinalSize {
                known,
                claimed: size,
            });
        }
        if size < self.consumed {
            return Err(FlowError::FinalSize {
                known: self.consumed,
                claimed: size,
            });
        }
        if size > self.limit {
            return Err(FlowError::Exceeded {
                would_be: size,
                limit: self.limit,
            });
        }
        self.final_size = Some(size);
        Ok(())
    }

    /// Whether to send a window update yet.
    ///
    /// The rule is the usual one and it is Izcal's own choice, not the
    /// standard's: update once the peer has used at least half of what
    /// it was given since the last update. Updating on every byte
    /// floods the connection with `MAX_DATA`; updating only at the
    /// limit stalls the sender for a round trip.
    #[must_use]
    pub const fn should_update(&self, window_size: u64) -> bool {
        self.available() <= window_size / 2
    }
}
