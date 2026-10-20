// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Bounded synchronous discovery. Run on a blocking worker; no receiver wiring.
//!
//! Matching is supplied precompiled by configuration. Filesystem observations
//! never establish durable identity, disappearance, or reader lifecycle. Delivery
//! is synchronous and ordered; a slow consumer stops traversal, with no handles
//! retained across delivery. Cancellation cannot interrupt a blocked kernel call.

mod batch;
mod budget;
mod linux;
mod probe;
mod roots;
mod walk;

pub use budget::{ProbeBudget, ProbeReservation};
pub use linux::Scanner;

use super::source::{FileLocator, SymlinkPolicy};
use std::path::{Path, PathBuf};

/// Precompiled configuration policy. Implementations must not accumulate paths.
///
/// Prefixes are supplied separately to [`Scanner::new`]. Mapping an exclusion
/// root must evaluate the full pattern, not exclude all descendants of its prefix.
pub trait MatchPolicy {
    /// Whether this lexical native path is a file candidate.
    fn includes(&self, path: &Path) -> bool;
    /// Whether traversal below this path can find a configured include.
    fn may_descend(&self, path: &Path) -> bool;
    /// Whether the lexical or verified target path matches an exclusion.
    fn excludes(&self, path: &Path) -> bool;
    /// Whether an entire subtree is excluded (not merely its root name).
    fn excludes_subtree(&self, path: &Path) -> bool;
    /// Evaluate a suffix through an exclusion root's canonical mapping.
    fn excludes_mapped(&self, root_index: usize, suffix: &Path) -> bool;
}

/// Hard bounds checked before scan storage is allocated.
#[derive(Clone, Copy, Debug)]
pub struct ScanLimits {
    /// Maximum directory stack depth, including the traversal root (1..=1024).
    pub depth: usize,
    /// Maximum bytes in a native path, including verification scratch (1..=65536).
    pub path_bytes: usize,
    /// Maximum root count for each include/exclude input (1..=1024).
    pub roots: usize,
    /// Maximum observations delivered in one batch (1..=1024).
    pub batch: usize,
    /// Raw fingerprint window after the ignored header (16..=65535).
    pub fingerprint_bytes: usize,
    /// Bytes skipped before the fingerprint.
    pub ignored_header_bytes: u64,
    /// Maximum enumeration/replay steps and traversal-directory opens per pass; nonzero.
    /// Bounds alias fan-out and replay work, not wall time inside kernel calls.
    pub work_units: u64,
}

impl Default for ScanLimits {
    fn default() -> Self {
        Self {
            depth: 64,
            path_bytes: 4096,
            roots: 128,
            batch: 64,
            fingerprint_bytes: 1000,
            ignored_header_bytes: 0,
            work_units: 1_000_000,
        }
    }
}

/// Immutable scanner inputs; roots are absolute lexical directory prefixes.
#[derive(Debug)]
pub struct ScanConfig {
    /// Includes may overlap; observations are not globally deduplicated here.
    pub include_roots: Vec<PathBuf>,
    /// Fixed exclusion prefixes, in the order used by `excludes_mapped`.
    pub exclusion_roots: Vec<PathBuf>,
    /// Final-entry and descendant-directory link policy.
    pub symlinks: SymlinkPolicy,
    /// Actual checkpoint namespace directory identity, not the whole state root.
    pub checkpoint_namespace: FileLocator,
    /// Storage and traversal bounds.
    pub limits: ScanLimits,
}

/// Evidence requested by the caller's bounded retained-state lookup.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ProbeRequest {
    /// Required for admission, reconnection, or validated reappearance.
    Full,
    /// Observe a known locator and policy without reading source content.
    /// A different locator automatically requires full candidate validation.
    PresenceOnly {
        /// Expected runtime locator, not a durable identity assertion.
        expected: FileLocator,
    },
}

/// Bounded outcome reason. Original errno is carried separately on observations.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ScanReason {
    /// Successfully inspected directory; supports retained binding-parent checks.
    Directory,
    /// A single policy-checked pin matched the requested locator. No admission,
    /// reconnection, or cancellation of pending disappearance is authorized.
    PresenceOnly,
    /// Required openat2 verification is unavailable in this execution context.
    VerificationUnavailable,
    /// Individual name no longer resolves in otherwise valid traversal context.
    Missing,
    /// Ordinary leaf link chain cannot resolve; no absence assertion.
    LeafLoop,
    /// Object is not a permitted regular source.
    Unsupported,
    /// An include root could not be established.
    RootUnavailable,
    /// An established include root now names a different directory.
    RootChanged,
    /// Required directory could not be enumerated or resumed reliably.
    Coverage,
    /// Saved enumeration evidence changed or a seek could not be completed.
    Resumption,
    /// This backend has not been qualified for close/reopen cookie resumption.
    UnqualifiedResumption,
    /// Directory cycle in the current ancestor stack.
    Cycle,
    /// A configured resource bound was reached.
    Limit,
    /// Required resolved-target policy could not be evaluated.
    Policy,
    /// Positive exclusion without opening content.
    Excluded,
    /// Pinned source could not be read; presence remains usable.
    Content,
    /// Sequential observations disagree.
    Unstable,
    /// Shared transient allowance was unavailable; retry later.
    ProbeBusy,
    /// Required open failed because the OS descriptor pool was exhausted.
    DescriptorExhausted,
    /// Caller cancelled; the pass cannot authorize negative conclusions.
    Cancelled,
}

impl ScanReason {
    /// Whether this outcome invalidates required scan coverage or policy evidence.
    ///
    /// Classify the operation's context before assigning a reason. A false result
    /// does not authorize admission or prove absence. Locator presence is separate
    /// evidence and can accompany both coverage failures and candidate-local failures.
    #[must_use]
    pub const fn invalidates_coverage(self) -> bool {
        match self {
            Self::VerificationUnavailable
            | Self::RootUnavailable
            | Self::RootChanged
            | Self::Coverage
            | Self::Resumption
            | Self::UnqualifiedResumption
            | Self::Cycle
            | Self::Limit
            | Self::Policy
            | Self::Unstable
            | Self::ProbeBusy
            | Self::DescriptorExhausted
            | Self::Cancelled => true,
            Self::Directory
            | Self::PresenceOnly
            | Self::Missing
            | Self::LeafLoop
            | Self::Unsupported
            | Self::Excluded
            | Self::Content => false,
        }
    }
}

/// Evidence from a single name. Contains no handles, leases, or durable identity.
#[derive(Debug)]
pub struct Observation {
    /// Native lexical name, bounded by `path_bytes`.
    pub path: PathBuf,
    /// Consistent locator evidence; never a durable identity assertion.
    pub locator: Option<FileLocator>,
    /// Handle-derived size, published only for fully validated candidates.
    pub size: Option<u64>,
    /// Same-handle modification time for a fully validated candidate, if supported.
    pub modified: Option<std::time::SystemTime>,
    /// Validated raw fingerprint prefix; empty for noneligible observations.
    pub fingerprint: Vec<u8>,
    /// `None` indicates a candidate that passed both probes and policy.
    pub reason: Option<ScanReason>,
    /// Original OS error number when present.
    pub errno: Option<i32>,
}

/// Ordered end-of-pass marker, emitted only after all batches are consumed.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ScanCompletion {
    /// Caller-supplied pass generation, echoed without scheduling policy.
    pub generation: u64,
    /// Required coverage and policy succeeded; not a snapshot or identity proof.
    pub complete: bool,
    /// Pass was stopped before traversal finished.
    pub cancelled: bool,
    /// Total names emitted, using saturating accounting.
    pub observations: u64,
}

/// Invalid setup is rejected before scanning or allocating bounded scratch.
#[derive(Debug, thiserror::Error)]
#[error("invalid discovery limits or nonabsolute/oversized root")]
pub struct InvalidScanConfig;

#[cfg(test)]
mod tests;
