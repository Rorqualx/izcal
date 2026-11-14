//! The client handshake: states, inputs, the table, and the two rules
//! that are not in it.
//!
//! From RFC 8446 §4 and Appendix A.1. Nothing here is derived from any
//! implementation (M-1 part 3).

use core::fmt;

/// A client handshake state, as Appendix A.1 names them.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum State {
    /// Nothing sent yet.
    Start,
    /// A ClientHello is out; waiting for ServerHello.
    WaitSh,
    /// ServerHello seen; waiting for EncryptedExtensions.
    WaitEe,
    /// Waiting for a CertificateRequest or the Certificate.
    WaitCertCr,
    /// A CertificateRequest was seen; waiting for the Certificate.
    WaitCert,
    /// Waiting for CertificateVerify.
    WaitCv,
    /// Waiting for the server's Finished.
    WaitFinished,
    /// The handshake is complete.
    Connected,
    /// An alert was sent; nothing more is read.
    Closed,
}

impl fmt::Display for State {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Self::Start => "START",
            Self::WaitSh => "WAIT_SH",
            Self::WaitEe => "WAIT_EE",
            Self::WaitCertCr => "WAIT_CERT_CR",
            Self::WaitCert => "WAIT_CERT",
            Self::WaitCv => "WAIT_CV",
            Self::WaitFinished => "WAIT_FINISHED",
            Self::Connected => "CONNECTED",
            Self::Closed => "CLOSED",
        };
        f.write_str(s)
    }
}

/// A handshake message the client may receive.
///
/// The messages a *client* never receives (ClientHello,
/// EndOfEarlyData) are absent: a type that can hold them would need a
/// rule for them in every state, and the rule is the same as for a
/// type the standard has not defined.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Input {
    /// `server_hello` (2).
    ServerHello,
    /// A ServerHello whose random is the special value of §4.1.3,
    /// which makes it a HelloRetryRequest.
    HelloRetryRequest,
    /// `encrypted_extensions` (8).
    EncryptedExtensions,
    /// `certificate_request` (13).
    CertificateRequest,
    /// `certificate` (11).
    Certificate,
    /// `certificate_verify` (15).
    CertificateVerify,
    /// `finished` (20).
    Finished,
    /// `new_session_ticket` (4), post-handshake only.
    NewSessionTicket,
    /// `key_update` (24), post-handshake only.
    KeyUpdate,
}

impl Input {
    /// Its `HandshakeType` number, from §4.
    ///
    /// A HelloRetryRequest is a ServerHello on the wire, so the two
    /// share a number; the difference is the random, which §4.1.3
    /// fixes.
    #[must_use]
    pub const fn handshake_type(self) -> u8 {
        match self {
            Self::ServerHello | Self::HelloRetryRequest => 2,
            Self::NewSessionTicket => 4,
            Self::EncryptedExtensions => 8,
            Self::Certificate => 11,
            Self::CertificateRequest => 13,
            Self::CertificateVerify => 15,
            Self::Finished => 20,
            Self::KeyUpdate => 24,
        }
    }
}

/// What the client does on a transition.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    /// Send a ClientHello. The first one, from `START`.
    SendClientHello,
    /// Send a second ClientHello, in answer to a HelloRetryRequest.
    SendSecondClientHello,
    /// Send the client's own flight — EndOfEarlyData if early data was
    /// offered, the client Certificate and CertificateVerify if one
    /// was requested, and Finished — and the handshake is complete.
    SendClientFlight {
        /// Whether a CertificateRequest was seen, so the flight
        /// carries a Certificate (possibly empty) and, if it is not
        /// empty, a CertificateVerify.
        certificate_requested: bool,
    },
    /// Take the message and wait for the next.
    Continue,
    /// A post-handshake message was handled; nothing is sent.
    PostHandshake,
}

/// Why the handshake was aborted, by the alert the standard names.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum Alert {
    /// §4: a message arrived in a state where the standard does not
    /// allow it. This is the default for everything the table does not
    /// name, which is the point of the table.
    UnexpectedMessage {
        /// The state it arrived in.
        state: State,
        /// What arrived.
        input: Input,
    },
    /// §4.1.4: a second HelloRetryRequest.
    SecondHelloRetryRequest,
}

impl fmt::Display for Alert {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Self::UnexpectedMessage { state, input } => {
                write!(f, "unexpected_message: {input:?} in {state}")
            }
            Self::SecondHelloRetryRequest => {
                f.write_str("unexpected_message: a second HelloRetryRequest (RFC 8446 4.1.4)")
            }
        }
    }
}

impl core::error::Error for Alert {}

/// Every transition the standard allows, as `(from, input, to)`.
///
/// **This is the whole of what is allowed.** A pair not here is an
/// `unexpected_message` alert, in every state, for every message —
/// including the ones added to [`Input`] later, which is why the table
/// is the safe shape and a match is not.
///
/// Two edges out of `WAIT_EE` share an input and differ by the
/// handshake's own state; [`Handshake::on_message`] resolves them, and
/// both appear here so the table stays the complete list.
pub const TRANSITIONS: &[(State, Input, State)] = &[
    // Appendix A.1, in its order.
    (State::WaitSh, Input::HelloRetryRequest, State::WaitSh),
    (State::WaitSh, Input::ServerHello, State::WaitEe),
    // Certificate-authenticated and PSK-resumed, respectively.
    (State::WaitEe, Input::EncryptedExtensions, State::WaitCertCr),
    (
        State::WaitEe,
        Input::EncryptedExtensions,
        State::WaitFinished,
    ),
    (
        State::WaitCertCr,
        Input::CertificateRequest,
        State::WaitCert,
    ),
    (State::WaitCertCr, Input::Certificate, State::WaitCv),
    (State::WaitCert, Input::Certificate, State::WaitCv),
    (State::WaitCv, Input::CertificateVerify, State::WaitFinished),
    (State::WaitFinished, Input::Finished, State::Connected),
    // §4.6: post-handshake messages, which are legal only once the
    // handshake is complete.
    (State::Connected, Input::NewSessionTicket, State::Connected),
    (State::Connected, Input::KeyUpdate, State::Connected),
];

/// The client's handshake.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Handshake {
    state: State,
    /// §4.1.4: at most one HelloRetryRequest.
    hello_retry_seen: bool,
    /// Whether the server is expected to authenticate with a
    /// certificate. False for a resumed or PSK handshake, which is the
    /// branch out of `WAIT_EE`.
    certificate_expected: bool,
    /// Whether a CertificateRequest was seen, which the client's own
    /// flight depends on.
    certificate_requested: bool,
    messages: u64,
}

impl Handshake {
    /// A handshake that expects the server to authenticate with a
    /// certificate.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            state: State::Start,
            hello_retry_seen: false,
            certificate_expected: true,
            certificate_requested: false,
            messages: 0,
        }
    }

    /// A handshake resumed from a pre-shared key, where the server
    /// does not send a Certificate.
    #[must_use]
    pub const fn resumed() -> Self {
        Self {
            state: State::Start,
            hello_retry_seen: false,
            certificate_expected: false,
            certificate_requested: false,
            messages: 0,
        }
    }

    /// Where it is.
    #[must_use]
    pub const fn state(&self) -> State {
        self.state
    }

    /// Whether a HelloRetryRequest has been seen.
    #[must_use]
    pub const fn hello_retry_seen(&self) -> bool {
        self.hello_retry_seen
    }

    /// How many messages have been accepted.
    #[must_use]
    pub const fn messages(&self) -> u64 {
        self.messages
    }

    /// Sends the first ClientHello.
    ///
    /// # Errors
    /// [`Alert::UnexpectedMessage`] when the handshake has already
    /// started, because a ClientHello sent twice by the client's own
    /// code is the same confusion as one accepted twice from a peer.
    pub fn start(&mut self) -> Result<Action, Alert> {
        if self.state != State::Start {
            return Err(Alert::UnexpectedMessage {
                state: self.state,
                input: Input::ServerHello,
            });
        }
        self.state = State::WaitSh;
        Ok(Action::SendClientHello)
    }

    /// Takes one handshake message.
    ///
    /// # Errors
    /// [`Alert`] when the message is not one the standard allows in
    /// this state. The handshake then moves to [`State::Closed`] and
    /// refuses everything after, because a peer that sent one message
    /// out of order is not one to keep reading.
    pub fn on_message(&mut self, input: Input) -> Result<Action, Alert> {
        let from = self.state;
        if from == State::Closed {
            return Err(self.abort(Alert::UnexpectedMessage { state: from, input }));
        }

        // The rule that is not a transition: at most one
        // HelloRetryRequest (§4.1.4).
        if input == Input::HelloRetryRequest && self.hello_retry_seen {
            return Err(self.abort(Alert::SecondHelloRetryRequest));
        }

        let allowed = TRANSITIONS
            .iter()
            .any(|(f, i, _)| *f == from && *i == input);
        if !allowed {
            return Err(self.abort(Alert::UnexpectedMessage { state: from, input }));
        }

        // The other rule that is not a transition: `WAIT_EE` has two
        // edges on one input, chosen by whether the server
        // authenticates with a certificate.
        let to = if from == State::WaitEe && input == Input::EncryptedExtensions {
            if self.certificate_expected {
                State::WaitCertCr
            } else {
                State::WaitFinished
            }
        } else {
            TRANSITIONS
                .iter()
                .find(|(f, i, _)| *f == from && *i == input)
                .map_or(State::Closed, |(_, _, t)| *t)
        };

        self.messages = self.messages.saturating_add(1);
        if input == Input::HelloRetryRequest {
            self.hello_retry_seen = true;
        }
        if input == Input::CertificateRequest {
            self.certificate_requested = true;
        }
        self.state = to;

        Ok(match input {
            Input::HelloRetryRequest => Action::SendSecondClientHello,
            Input::Finished if to == State::Connected => Action::SendClientFlight {
                certificate_requested: self.certificate_requested,
            },
            Input::NewSessionTicket | Input::KeyUpdate => Action::PostHandshake,
            // Listed rather than wildcarded: a message added to `Input`
            // later must be given an action here, and the compiler is
            // the only reviewer that never forgets to ask.
            Input::ServerHello
            | Input::EncryptedExtensions
            | Input::CertificateRequest
            | Input::Certificate
            | Input::CertificateVerify
            | Input::Finished => Action::Continue,
        })
    }

    fn abort(&mut self, alert: Alert) -> Alert {
        self.state = State::Closed;
        alert
    }
}

impl Default for Handshake {
    fn default() -> Self {
        Self::new()
    }
}
