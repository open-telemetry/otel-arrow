// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! One local transaction boundary for assignment grants and opaque source progress.
//! Blocking storage belongs on a dedicated worker. This crate knows neither
//! pipeline instances nor Filelog records and does not authorize publication.

#[cfg(all(target_os = "linux", feature = "sqlite"))]
mod store;
#[cfg(all(target_os = "linux", feature = "sqlite"))]
pub use store::Store;
#[cfg(not(all(target_os = "linux", feature = "sqlite")))]
mod unsupported;
#[cfg(not(all(target_os = "linux", feature = "sqlite")))]
pub use unsupported::Store;

/// Whether this build can open the qualified local backend.
pub const AVAILABLE: bool = cfg!(all(target_os = "linux", feature = "sqlite"));

/// Bounds checked before disk access or transaction staging.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Limits {
    /// Assignment rows, including retired terms retained during this epoch.
    pub assignments: usize,
    /// Retained source records, including records from retired assignments.
    pub sources: usize,
    /// Aggregate opaque progress bytes.
    pub progress_bytes: usize,
    /// Maximum opaque progress bytes for one source.
    pub source_bytes: usize,
    /// Maximum updates in one all-or-none publication.
    pub transaction_items: usize,
    /// Maximum retained bytes of one submitted publication.
    pub transaction_bytes: usize,
    /// Receipt sequence window per epoch; current and preceding epochs are retained.
    pub receipts: usize,
    /// Database size ceiling; reserve up to twice this size for the rollback journal.
    pub database_bytes: u64,
}
impl Default for Limits {
    fn default() -> Self {
        Self {
            assignments: 65_536,
            sources: 65_536,
            progress_bytes: 64 * 1024 * 1024,
            source_bytes: 64 * 1024,
            transaction_items: 256,
            transaction_bytes: 1024 * 1024,
            receipts: 4096,
            database_bytes: 256 * 1024 * 1024,
        }
    }
}
impl Limits {
    /// Validate arithmetic and implementation ceilings without opening storage.
    pub fn validate(self) -> Result<Self, Error> {
        if self.assignments == 0
            || self.assignments > 1_048_576
            || self.sources == 0
            || self.sources > 1_048_576
            || self.progress_bytes == 0
            || self.progress_bytes > 1024 * 1024 * 1024
            || self.source_bytes == 0
            || self.source_bytes > self.progress_bytes
            || self.transaction_items == 0
            || self.transaction_items > 4096
            || self.transaction_bytes == 0
            || self.transaction_bytes > 16 * 1024 * 1024
            || self.receipts == 0
            || self.receipts > 65_536
            || self.database_bytes < 1024 * 1024
            || self.database_bytes > 4 * 1024 * 1024 * 1024
        {
            return Err(Error::Invalid("invalid durable authority limits"));
        }
        Ok(self)
    }
}

/// Durable command identity. A new exclusive open starts a fresh epoch.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct OperationId {
    /// Epoch allocated durably by storage, never chosen by a participant.
    pub epoch: u64,
    /// Non-reused client sequence; bounded out-of-order submission is supported.
    pub sequence: u64,
}

/// Exact persistent authority. Tokens are opaque to storage and bounded to 4096 bytes.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Grant {
    /// Store epoch that can activate this grant.
    pub epoch: u64,
    /// Stable assignment ID, at most 1024 bytes.
    pub work: String,
    /// Strictly increasing term for this assignment in the current epoch.
    pub term: u64,
    /// Exact host/session/settings identity supplied by the engine.
    pub token: Vec<u8>,
}

/// One conditional source-state update. Revision zero means never registered.
#[derive(Clone, Debug)]
pub struct ProgressUpdate {
    /// Exact current assignment grant.
    pub grant: Grant,
    /// Stable, namespace-local source identity, at most 1024 bytes.
    pub source: String,
    /// Expected committed source revision.
    pub expected_revision: u64,
    /// Opaque source state. Its interpretation and Ack authorization belong to the source adapter.
    pub progress: Vec<u8>,
}

/// Committed state, preserved across assignment retirement and process epochs.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Progress {
    /// Assignment to which this source was first bound.
    pub work: String,
    /// Monotonic CAS revision.
    pub revision: u64,
    /// Opaque source state.
    pub bytes: Vec<u8>,
}

/// Definite transaction outcome; no successful partial publication exists.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Outcome {
    /// The whole command committed durably.
    Applied,
    /// Preconditions or configured resource bounds rejected the whole command.
    Rejected,
    /// No retained receipt can establish this command's outcome.
    Unknown,
}

/// Errors never authorize ownership transfer or progress advancement.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Invalid or incompatible input, before a mutation began.
    #[error("{0}")]
    Invalid(&'static str),
    /// Another owner holds the local namespace.
    #[error("durable authority namespace is locked")]
    Locked,
    /// Untrusted or inconsistent artifacts; never auto-salvaged.
    #[error("untrusted durable authority state: {0}")]
    Untrusted(&'static str),
    /// A transaction failed with an unknown outcome. This store is poisoned.
    #[error("durable authority I/O failed; outcome unknown: {0}")]
    Storage(String),
    /// No further mutations or durability claims are allowed on this instance.
    #[error("durable authority requires recovery after an I/O failure")]
    Poisoned,
    /// Filesystem access failed before a transaction began.
    #[error(transparent)]
    Io(#[from] std::io::Error),
}

/// Test-only interruption boundaries; the production build has no injected callbacks.
#[cfg(feature = "test-utils")]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FaultPoint {
    /// The first source update was staged, before later updates in the same transaction.
    AfterFirstUpdate,
    /// A mutation has touched SQLite pages, before the atomic commit.
    BeforeCommit,
    /// SQLite committed, but the caller has not received its acknowledgement.
    AfterCommit,
}
