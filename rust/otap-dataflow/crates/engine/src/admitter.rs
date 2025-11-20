// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Admission Controller (single-threaded, thread-per-core friendly)
//!
//! ToDo Add a memory usage based admitter that tracks memory usage and rejects new connections/datagrams when memory usage exceeds a threshold.
//! ToDo Add other types of admitters (e.g. rate-limiting, token-bucket, ...)

use std::cell::Cell;
use std::rc::Rc;

/// The top-level admission classes the per-cpu controller understands.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AdmissionClass {
    /// A new transport connection (e.g. TCP / HTTP/2).
    Connection,
    /// A single connectionless datagram of work (e.g. UDP, QUIC, Unix datagram).
    Datagram,
}

/// The decision returned by admission attempts.
///
/// The type parameter `T` is the guard returned on success (e.g. `ConnectionGuard`,
/// `DatagramGuard`, or a wrapper `Guard` enum for convenience).
#[derive(Debug)]
#[must_use]
pub enum AdmitDecision<T> {
    /// The request was admitted. The returned guard must be kept alive for the
    /// lifetime of the work. Dropping it releases the reserved slot(s).
    Admitted(T),
    /// Temporary backpressure (limit reached but system otherwise healthy). The caller
    /// may retry, pause accepting, or send a soft error (e.g. gRPC `RESOURCE_EXHAUSTED`).
    Busy,
    /// Hard rejection (e.g. circuit breaker/policy). Consider dropping the connection or
    /// returning a definitive error.
    Reject {
        /// Stable, human-readable reason (useful for metrics/alerts).
        message: &'static str,
    },
}

/// Internal counters shared by the `Admitter` and its guards.
///
/// All counters are "single-threaded" and updated via `Cell`.
#[derive(Debug)]
struct Inner {
    // Connections
    max_conns: u32,
    inflight_conns: Cell<u32>,

    // Streams-per-connection (enforced inside each connection's state)
    max_streams_per_conn: u32,

    // Datagrams
    max_datagrams: u32,
    inflight_datagrams: Cell<u32>,

    // Per-processor circuit breaker
    breaker_open: Cell<bool>,
}

/// Admission controller (single-threaded).
///
/// This controller caps in-flight connections, in-flight streams per connection, and in-flight
/// datagrams. It also exposes a per-processor circuit breaker to hard-reject new admissions
/// immediately under global pressure.
///
/// Note: This type is `Rc`-based and not thread-safe. It is intended to be used
/// in a thread-per-core model where each core has its own `Admitter` instance. It also means
/// that cloning is cheap and can be done freely to hand out references to guards.
#[derive(Debug, Clone)]
#[must_use]
pub struct Admitter {
    inner: Rc<Inner>,
}

impl Admitter {
    /// Construct a new `Admitter`.
    ///
    /// - `max_conns`: maximum in-flight connections.
    /// - `max_streams_per_conn`: maximum in-flight streams per connection.
    /// - `max_datagrams`: maximum in-flight datagrams (connectionless work).
    pub fn new(max_conns: u32, max_streams_per_conn: u32, max_datagrams: u32) -> Self {
        Self {
            inner: Rc::new(Inner {
                max_conns,
                inflight_conns: Cell::new(0),
                max_streams_per_conn,
                max_datagrams,
                inflight_datagrams: Cell::new(0),
                breaker_open: Cell::new(false),
            }),
        }
    }

    /// Open or close the local circuit breaker.
    ///
    /// When open, all new admissions are hard-rejected (`Reject { message: "circuit_breaker_open" }`).
    pub fn set_breaker(&self, open: bool) {
        self.inner.breaker_open.set(open);
    }

    /// Admit a connection, returning a `ConnectionGuard` on success.
    pub fn try_admit_connection(&self) -> AdmitDecision<ConnectionGuard> {
        if self.inner.breaker_open.get() {
            return AdmitDecision::Reject {
                message: "circuit_breaker_open",
            };
        }
        let cur = self.inner.inflight_conns.get();
        if cur >= self.inner.max_conns {
            return AdmitDecision::Busy;
        }
        self.inner.inflight_conns.set(cur + 1);

        let state = Rc::new(ConnState {
            streams_inflight: Cell::new(0),
            max_streams: self.inner.max_streams_per_conn,
        });

        AdmitDecision::Admitted(ConnectionGuard {
            inner: Rc::clone(&self.inner),
            state,
        })
    }

    /// Admit a datagram (connectionless unit), returning a `DatagramGuard` on success.
    pub fn try_admit_datagram(&self) -> AdmitDecision<DatagramGuard> {
        if self.inner.breaker_open.get() {
            return AdmitDecision::Reject {
                message: "circuit_breaker_open",
            };
        }
        let cur = self.inner.inflight_datagrams.get();
        if cur >= self.inner.max_datagrams {
            return AdmitDecision::Busy;
        }
        self.inner.inflight_datagrams.set(cur + 1);
        AdmitDecision::Admitted(DatagramGuard {
            inner: Rc::clone(&self.inner),
        })
    }

    /// Read-only observability snapshot.
    pub fn report(&self) -> Report {
        Report {
            inflight_conns: self.inner.inflight_conns.get(),
            max_conns: self.inner.max_conns,
            inflight_datagrams: self.inner.inflight_datagrams.get(),
            max_datagrams: self.inner.max_datagrams,
            breaker_open: self.inner.breaker_open.get(),
        }
    }
}

/// Per-connection state tracking in-flight streams.
#[derive(Debug)]
struct ConnState {
    streams_inflight: Cell<u32>,
    max_streams: u32,
}

/// Guard representing a live connection.
///
/// While this guard is alive:
/// - the global in-flight connection count is incremented
/// - you can open up to `max_streams_per_conn` streams via `try_open_stream()`.
#[derive(Debug)]
pub struct ConnectionGuard {
    inner: Rc<Inner>,
    state: Rc<ConnState>,
}

impl ConnectionGuard {
    /// Attempt to open a stream on this connection.
    ///
    /// Enforces the per-connection stream limit. On success, returns a `StreamGuard`
    /// that must be held for the stream's lifetime.
    pub fn try_open_stream(&self) -> AdmitDecision<StreamGuard> {
        if self.inner.breaker_open.get() {
            return AdmitDecision::Reject {
                message: "circuit_breaker_open",
            };
        }
        let cur = self.state.streams_inflight.get();
        if cur >= self.state.max_streams {
            return AdmitDecision::Busy;
        }
        self.state.streams_inflight.set(cur + 1);
        AdmitDecision::Admitted(StreamGuard {
            state: Rc::clone(&self.state),
        })
    }

    /// Current number of in-flight streams on this connection.
    #[must_use]
    pub fn streams_inflight(&self) -> u32 {
        self.state.streams_inflight.get()
    }
}

impl Drop for ConnectionGuard {
    fn drop(&mut self) {
        // Return one connection slot.
        let cur = self.inner.inflight_conns.get();
        self.inner.inflight_conns.set(cur.saturating_sub(1));
        // Any remaining streams are assumed dropped earlier; if not,
        // they will decrement their own counters as they drop.
    }
}

/// Guard representing a live stream (logical sub-unit inside a connection).
///
/// While this guard is alive, the connection's in-flight stream count is incremented.
#[derive(Debug)]
pub struct StreamGuard {
    state: Rc<ConnState>,
}

impl Drop for StreamGuard {
    fn drop(&mut self) {
        let cur = self.state.streams_inflight.get();
        self.state.streams_inflight.set(cur.saturating_sub(1));
    }
}

/// Guard representing a live datagram (connectionless unit).
#[derive(Debug)]
pub struct DatagramGuard {
    inner: Rc<Inner>,
}

impl Drop for DatagramGuard {
    fn drop(&mut self) {
        let cur = self.inner.inflight_datagrams.get();
        self.inner.inflight_datagrams.set(cur.saturating_sub(1));
    }
}

/// Lightweight observability snapshot for counters and breaker state.
#[derive(Debug, Clone, Copy)]
#[must_use]
pub struct Report {
    /// Current number of in-flight connections.
    pub inflight_conns: u32,
    /// Configured maximum number of in-flight connections.
    pub max_conns: u32,
    /// Current number of in-flight datagrams.
    pub inflight_datagrams: u32,
    /// Configured maximum number of in-flight datagrams.
    pub max_datagrams: u32,
    /// Whether the circuit breaker is open (true) or closed (false).
    pub breaker_open: bool,
}
