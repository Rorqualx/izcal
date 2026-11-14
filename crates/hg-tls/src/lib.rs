//! `hg-tls`: the TLS 1.3 client handshake, as a table.
//!
//! NW-M0's row asks for one thing from this crate — "`hg-tls`'s
//! transition table" — and that is what is here. The key schedule, the
//! extensions, the record, QUIC and DTLS framings, ECH, the resumption
//! store and the ClientHello shape (NW-D22) are NW-M1's, and nothing
//! here anticipates them.
//!
//! # Why a table, rather than the handshake written out
//!
//! RFC 8446 Appendix A.1 prints the client's state machine as a
//! diagram, and the usual way to implement it is a function per state
//! with an `if` chain inside. That shape has a failure mode it cannot
//! see: **the message nobody thought about**. An `if` chain handles
//! what it was written for and falls through for everything else, and
//! what "everything else" should do — `unexpected_message`, §4 — is a
//! line somebody has to remember to write, in every state.
//!
//! A table inverts that. The transitions are data; **everything not in
//! the table is an alert**, by construction, in every state at once,
//! including the states added later. That is the property that matters
//! for a protocol whose history is largely a history of messages
//! accepted in the wrong order, and it is what [`TRANSITIONS`] is for.
//!
//! Two rules are not transitions and are written out beside it,
//! because pretending they are would make the table lie:
//!
//! - **A HelloRetryRequest may arrive once** (§4.1.4: "a client MUST
//!   abort the handshake with an `unexpected_message` alert" on a
//!   second one). A pure table would allow it forever.
//! - **EncryptedExtensions leads to one of two states**, by whether
//!   the server authenticated with a certificate or a pre-shared key
//!   (Appendix A.1's two edges out of `WAIT_EE`). The branch is on
//!   something the handshake already knows, not on the message.
//!
//! Each is a named field on [`Handshake`] and each has its own test,
//! so the exception is visible rather than buried in a match arm.

pub mod handshake;

pub use handshake::{Action, Alert, Handshake, Input, State, TRANSITIONS};
