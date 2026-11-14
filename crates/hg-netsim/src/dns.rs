//! A stub resolver's peer: a zone, and the ways it fails.
//!
//! §6.6 gives `hg-dns` the stub resolver, the encrypted transports and
//! HTTPS/SVCB records. What a resolver has to be tested against is not
//! a correct name server — those are easy — but the ones that answer
//! slowly, truncate, return `SERVFAIL`, or return nothing at all, and
//! a Happy Eyeballs implementation has to be right about which of
//! those it waited for.
//!
//! So the model here is a zone plus a per-name **behaviour**, and the
//! answer carries the delay the caller must wait before it arrives.
//! No wire format: `hg-dns` owns the encoding, and a simulation that
//! encoded and decoded would be testing the encoder twice.

use std::collections::BTreeMap;

use hg_substrate::RandomStream;
use hg_types::Duration;

/// What a query asks for.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Query {
    /// An IPv4 address (`A`).
    A,
    /// An IPv6 address (`AAAA`).
    Aaaa,
    /// Service binding (`HTTPS`), which §6.6 uses for ECH and ALPN.
    Https,
}

/// One record in the zone.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Record {
    /// Four bytes.
    A([u8; 4]),
    /// Sixteen bytes.
    Aaaa([u8; 16]),
    /// A service binding: priority, target name, and the parameters as
    /// opaque bytes, which `hg-dns` parses and this does not.
    Https {
        /// `SvcPriority`; 0 is an alias record.
        priority: u16,
        /// `TargetName`.
        target: String,
        /// `SvcParams`, unparsed.
        params: Vec<u8>,
    },
}

/// How a name behaves.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Behaviour {
    /// Answers from the zone.
    #[default]
    Normal,
    /// `NXDOMAIN`: the name does not exist.
    NoSuchName,
    /// `SERVFAIL`.
    ServerFailure,
    /// The answer does not fit and the truncation bit is set, so a
    /// stub resolver must retry over TCP.
    Truncated,
    /// No answer at all: the query times out.
    Silent,
}

/// What a query gets back.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Answer {
    /// Records, after `after`.
    Records {
        /// What was found. Empty is a `NOERROR` with no records, which
        /// is a different thing from `NXDOMAIN` and which resolvers
        /// routinely confuse.
        records: Vec<Record>,
        /// How long the caller waits.
        after: Duration,
    },
    /// `NXDOMAIN`, after `after`.
    NoSuchName {
        /// How long the caller waits.
        after: Duration,
    },
    /// `SERVFAIL`, after `after`.
    ServerFailure {
        /// How long the caller waits.
        after: Duration,
    },
    /// Truncated: the caller must retry over TCP.
    Truncated {
        /// How long the caller waits.
        after: Duration,
    },
    /// Nothing arrives.
    Silent,
}

/// A name server for one zone.
#[derive(Debug, Clone, Default)]
pub struct Resolver {
    zone: BTreeMap<(String, Query), Vec<Record>>,
    behaviour: BTreeMap<String, Behaviour>,
    /// Round-trip time to the server, before jitter.
    pub rtt: Duration,
    /// How much the round trip varies, in either direction.
    pub jitter: Duration,
    queries: u64,
}

impl Resolver {
    /// An empty zone that answers immediately.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds a record. Names are compared as written: a caller that
    /// cares about case-insensitivity lowercases first, because that
    /// is `hg-dns`'s rule to apply and not this one's to assume.
    pub fn add(&mut self, name: &str, query: Query, record: Record) -> &mut Self {
        self.zone
            .entry((name.to_owned(), query))
            .or_default()
            .push(record);
        self
    }

    /// Makes `name` behave badly.
    pub fn set_behaviour(&mut self, name: &str, behaviour: Behaviour) -> &mut Self {
        self.behaviour.insert(name.to_owned(), behaviour);
        self
    }

    /// How many queries have been asked.
    #[must_use]
    pub const fn queries(&self) -> u64 {
        self.queries
    }

    /// Answers one query.
    ///
    /// Every answer draws once from `rng` for its round trip, whatever
    /// the behaviour, so adding a failing name to a zone does not
    /// change the timing of the names around it.
    pub fn ask(&mut self, name: &str, query: Query, rng: &mut dyn RandomStream) -> Answer {
        self.queries = self.queries.saturating_add(1);
        let after = draw_rtt(rng, self.rtt, self.jitter);
        match self.behaviour.get(name).copied().unwrap_or_default() {
            Behaviour::Silent => Answer::Silent,
            Behaviour::NoSuchName => Answer::NoSuchName { after },
            Behaviour::ServerFailure => Answer::ServerFailure { after },
            Behaviour::Truncated => Answer::Truncated { after },
            Behaviour::Normal => {
                // A name with no record of the asked type but some
                // other type is NOERROR with no records, not NXDOMAIN.
                let known = self.zone.keys().any(|(n, _)| n == name);
                let records = self
                    .zone
                    .get(&(name.to_owned(), query))
                    .cloned()
                    .unwrap_or_default();
                if records.is_empty() && !known {
                    Answer::NoSuchName { after }
                } else {
                    Answer::Records { records, after }
                }
            }
        }
    }
}

fn draw_rtt(rng: &mut dyn RandomStream, rtt: Duration, jitter: Duration) -> Duration {
    let span = jitter.as_nanos();
    if span == 0 {
        let _ = rng.next_u64();
        return rtt;
    }
    let span = u64::try_from(span).unwrap_or(u64::MAX);
    let full = span.saturating_mul(2).saturating_add(1);
    let drawn = rng.next_u64().checked_rem(full).unwrap_or(0);
    if drawn >= span {
        rtt.saturating_add(Duration::from_nanos(drawn.saturating_sub(span)))
    } else {
        rtt.saturating_sub(Duration::from_nanos(span.saturating_sub(drawn)))
    }
}
