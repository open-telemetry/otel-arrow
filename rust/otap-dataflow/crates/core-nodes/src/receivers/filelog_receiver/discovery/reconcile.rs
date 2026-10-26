// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Ordered, bounded discovery evidence. No filesystem I/O or lifecycle actions.
//!
//! Consume scanner callbacks with `observe`, then its exact completion with
//! `finish`. Drain candidate offers before starting another pass. Identity and
//! lifecycle owners must serialize their decisions with this synchronous stream;
//! a scan completion is not proof of permanent absence.

mod admission;
mod bindings;
mod candidates;
mod storage;
#[cfg(test)]
mod tests;

pub use admission::{AdmissionChange, AdmissionError};
pub use candidates::{Candidate, CandidateOffer, IdentityFeedback, OfferToken, Replacement};

use super::{FileLocator, Observation, ScanCompletion, ScanReason};
use candidates::Candidates;
use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::time::Duration;
use storage::{RecordIndex, Records};

/// Discovery-local identity key supplied by the identity owner.
///
/// It must remain unique for the lifetime of this reconciler, including after
/// removal. It is not derived from a path, locator, or fingerprint. The owner
/// maps this key to its durable file ID; discovery never generates durable IDs.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct TrackedId(pub u64);

/// Independent resource inputs; no dependency on receiver configuration types.
#[derive(Clone, Copy, Debug)]
pub struct ReconcileLimits {
    /// Maximum tracked records, including nonresident and terminal records.
    pub tracked: usize,
    /// Maximum retained candidates.
    pub pending: usize,
    /// Maximum candidates offered per pass, at most the caller's open-file limit.
    pub offers: usize,
    /// Maximum complete native path bytes.
    pub path_bytes: usize,
    /// Maximum fingerprint bytes per candidate.
    pub fingerprint_bytes: usize,
}

impl ReconcileLimits {
    /// Copies scanner evidence bounds, with independent population limits.
    #[must_use]
    pub fn from_scan(
        scan: &super::ScanLimits,
        tracked: usize,
        pending: usize,
        offers: usize,
    ) -> Self {
        Self {
            tracked,
            pending,
            offers,
            path_bytes: scan.path_bytes,
            fingerprint_bytes: scan.fingerprint_bytes,
        }
    }

    fn validate(self) -> Result<Self, ReconcileError> {
        if self.tracked == 0
            || self.pending == 0
            || self.offers == 0
            || !super::valid_evidence_limits(self.path_bytes, self.fingerprint_bytes)
        {
            return Err(ReconcileError::Limits);
        }
        // Conservative allocation admission, including full paths, slot/index
        // overhead and candidate copies. Three path buffers are capped by L.
        // 512 bytes cover a packed record, three ID worklists and five ID/slot
        // tables reserved at 2N (including worst power-of-two bucket rounding).
        // See the storage-bound regression when changing these types or hashbrown.
        // This is not an RSS measurement.
        let tracked = self
            .path_bytes
            .checked_mul(3)
            .and_then(|n| n.checked_add(512))
            .and_then(|n| n.checked_mul(self.tracked))
            .and_then(|n| n.checked_add(4096))
            // One path/fingerprint replacement can transiently own old and new
            // buffers. This scratch is charged once, not once per record.
            .and_then(|n| n.checked_add(self.path_bytes))
            .and_then(|n| n.checked_add(self.fingerprint_bytes));
        let candidates = self.pending.checked_add(self.offers).and_then(|n| {
            self.path_bytes
                .checked_add(self.fingerprint_bytes)
                .and_then(|bytes| bytes.checked_mul(2))
                .and_then(|bytes| bytes.checked_add(512))
                .and_then(|bytes| bytes.checked_mul(n))
        });
        if tracked
            .and_then(|a| candidates.and_then(|b| a.checked_add(b)))
            .is_none_or(|n| n > 1024 * 1024 * 1024)
        {
            return Err(ReconcileError::Limits);
        }
        Ok(self)
    }
}

/// A caller error never authorizes a negative conclusion or binding change.
#[derive(Clone, Copy, Debug, Eq, PartialEq, thiserror::Error)]
pub enum ReconcileError {
    /// Invalid limits or checked storage admission exceeded.
    #[error("invalid reconciliation limits or storage bound exceeds 1 GiB")]
    Limits,
    /// The requested operation is outside its pass phase.
    #[error("invalid reconciliation phase")]
    Phase,
    /// Generation is stale, duplicated, or not the active generation.
    #[error("invalid reconciliation generation")]
    Generation,
    /// Explicit monotonic time went backwards.
    #[error("reconciliation time went backwards")]
    Time,
    /// Scanner completion does not account for every consumed observation.
    #[error("scanner completion observation count mismatch")]
    Count,
    /// Malformed or oversized observation.
    #[error("invalid discovery observation")]
    Observation,
    /// Tracked record limit reached.
    #[error("tracked discovery capacity exhausted")]
    Capacity,
    /// Unknown, duplicate or conflicting tracked record.
    #[error("invalid tracked identity or binding")]
    Tracked,
    /// Offer was already acknowledged or belongs to another pass.
    #[error("stale discovery feedback")]
    Feedback,
    /// Invalid delay arithmetic; configuration owns operational interval bounds.
    #[error("invalid reconciliation delay or arithmetic overflow")]
    Delay,
    /// A monotonic counter cannot advance without wrapping.
    #[error("discovery sequence exhausted")]
    Exhausted,
}

/// Runtime binding state; historical paths do not claim live ownership.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BindingRole {
    /// Recovery has not yet validated the saved path.
    Unverified,
    /// Identity owner validated this path as the current distinguished binding.
    Live,
    /// A successful replacement superseded this path; preserve it for rebound work.
    Superseded,
}

/// Minimal identity-owner view. No offsets, descriptors, or checkpoint mutations.
#[derive(Debug)]
pub struct TrackedFile {
    /// Stable discovery correlation key.
    pub id: TrackedId,
    /// Runtime locator; equality alone never reconnects durable progress.
    pub locator: FileLocator,
    /// Complete native binding when available. Never put a truncated suffix here.
    pub binding: Option<PathBuf>,
    /// Role reconstructed by the identity owner on recovery.
    pub role: BindingRole,
    /// Active records participate in binding work and reasoned transitions.
    /// Terminal/quarantined records still receive conservative presence evidence.
    pub active: bool,
    /// Whether an existing reader owns this locator; otherwise offer it to identity.
    pub resident: bool,
}

#[derive(Clone, Copy, Default, Eq, PartialEq)]
enum Visit {
    #[default]
    Unseen,
    Visiting,
    Done,
    Blocked,
}

// Missing names require valid environment context; positive policy/locator
// evidence does not depend on an unrelated name being discoverable.
#[derive(Clone, Copy, Default, Eq, PartialEq)]
enum BindingIneligibility {
    #[default]
    Unknown,
    Missing,
    Positive,
}

impl BindingIneligibility {
    fn allows_exclusion(self, environment_failed: bool) -> bool {
        self == Self::Positive || (self == Self::Missing && !environment_failed)
    }

    fn record_miss(&mut self) {
        if *self == Self::Unknown {
            *self = Self::Missing;
        }
    }
}

// Policy-qualified names can protect an existing reader. Only full source
// validation can authorize identity work or binding publication.
#[derive(Clone, Copy, Debug, Default, Eq, Ord, PartialEq, PartialOrd)]
enum NameEvidence {
    #[default]
    None,
    PolicyQualified,
    Full,
}

impl NameEvidence {
    fn from_observation(observation: &Observation) -> Self {
        match observation.reason {
            None => Self::Full,
            Some(ScanReason::PresenceOnly | ScanReason::Content) => Self::PolicyQualified,
            // A locator on other failures does not prove that policy succeeded.
            Some(_) => Self::None,
        }
    }

    fn is_named(self) -> bool {
        self != Self::None
    }
    fn is_full(self) -> bool {
        self == Self::Full
    }
    fn merge(&mut self, other: Self) {
        *self = (*self).max(other);
    }
}

// A foreign locator can be observed before it is qualified for identity work.
// Keep that bounded fact for contradiction checks without exposing a weak
// observation as an actionable rebound or allowing it to supersede a binding.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
enum BindingTarget {
    #[default]
    None,
    Observed(FileLocator),
    Pending(FileLocator),
    Resolved(FileLocator),
}

impl BindingTarget {
    fn observed(self) -> Option<FileLocator> {
        match self {
            Self::None => None,
            Self::Observed(locator) | Self::Pending(locator) | Self::Resolved(locator) => {
                Some(locator)
            }
        }
    }

    fn rebound(self) -> Option<FileLocator> {
        match self {
            Self::Pending(locator) | Self::Resolved(locator) => Some(locator),
            Self::None | Self::Observed(_) => None,
        }
    }

    fn pending(self) -> Option<FileLocator> {
        match self {
            Self::Pending(locator) => Some(locator),
            _ => None,
        }
    }

    fn is_resolved(self) -> bool {
        matches!(self, Self::Resolved(_))
    }

    fn observe(&mut self, locator: FileLocator) {
        if matches!(self, Self::None) {
            *self = Self::Observed(locator);
        }
    }

    fn qualify(&mut self, locator: FileLocator, resolved: bool) -> bool {
        let new_rebound = self.rebound() != Some(locator);
        *self = if resolved {
            Self::Resolved(locator)
        } else {
            Self::Pending(locator)
        };
        new_rebound
    }

    fn resolve(&mut self) -> bool {
        if let Self::Pending(locator) = *self {
            *self = Self::Resolved(locator);
            true
        } else {
            false
        }
    }
}

#[derive(Default)]
struct TrackedPass {
    present: bool,
    excluded: bool,
    binding_evidence: NameEvidence,
    minimum_evidence: NameEvidence,
    binding_target: BindingTarget,
    binding_ineligible: BindingIneligibility,
    exclusion_sent: bool,
    conflict: bool,
    parent_seen: bool,
    // Binding evidence from a changed parent cannot recover within this pass.
    parent_unstable: bool,
    binding_checked: bool,
    alias_checked: bool,
    exclusion_blocked: bool,
    graph: Visit,
    // Disjoint-set metadata for new, uncontested rebound edges in this pass.
    rebound_parent: u64,
    rebound_rank: u8,
    check_queued: bool,
}

impl TrackedPass {
    fn name_evidence(&self) -> NameEvidence {
        self.binding_evidence.max(self.minimum_evidence)
    }
}

struct Tracked {
    file: TrackedFile,
    pass: TrackedPass,
    minimum: PathBuf,
    previous_alias: PathBuf,
    parent: Option<FileLocator>,
    parent_changed: bool,
    successor: Option<(TrackedId, FileLocator)>,
}

impl Tracked {
    fn binding_blocked(&self) -> bool {
        self.parent_changed || self.pass.parent_unstable
    }

    fn allows_replacement(&self, replacement: FileLocator) -> bool {
        !self.pass.conflict
            && !self.binding_blocked()
            && !self.pass.binding_evidence.is_named()
            && self.pass.binding_target.rebound() == Some(replacement)
    }
}

#[derive(Clone, Copy, Eq, PartialEq)]
enum Origin {
    Scan,
    Binding,
}

/// Identity work required before any complete-pass binding reselection.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Rebound {
    /// Retained predecessor whose distinguished path changed.
    pub predecessor: TrackedId,
    /// Current observed locator at that path; not a durable identity decision.
    pub replacement: FileLocator,
}

/// One currently needed targeted lookup. The path is borrowed only for the check.
#[derive(Debug)]
pub struct BindingRequest<'a> {
    /// Complete native name to check.
    pub path: &'a Path,
    /// Established distinguished-parent identity; aliases have no such baseline.
    pub parent: Option<FileLocator>,
}

/// Ordered evidence delivered by synchronous callbacks.
#[derive(Debug)]
pub enum Evidence<'a> {
    /// Required coverage failed. Delivered before completion, including during a stall.
    CoverageLost(ScanReason),
    /// The tracked binding's established parent identity changed.
    ParentChanged(TrackedId),
    /// An established include root changed; identifies the directory itself.
    RootChanged(&'a Path),
    /// Conservative presence; does not itself authorize admission or cancellation.
    Present(TrackedId),
    /// Fresh eligible evidence for identity/lifecycle validation; binding is unchanged.
    Validated(TrackedId, &'a Observation),
    /// Exclusion is distinct from disappearance and must not finalize the record.
    Excluded(TrackedId),
    /// A complete pass found no presence; lifecycle owner decides what follows.
    Disappeared(TrackedId),
    /// All observations and reasoned transitions precede this marker.
    Completed(PassSummary),
}

/// Pass interval and effective completeness, after reconciliation evidence checks.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PassSummary {
    /// Scanner generation.
    pub generation: u64,
    /// Caller-supplied monotonic pass start.
    pub started: Duration,
    /// Caller-supplied monotonic completion time.
    pub finished: Duration,
    /// Required scan and reconciliation evidence succeeded; not a snapshot.
    pub complete: bool,
    /// Observations consumed, including diagnostics and aliases.
    pub observations: u64,
}

/// Bounded accounting inputs for later telemetry integration.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct ReconcileStats {
    /// Tracked records, including terminal records.
    pub tracked: usize,
    /// Retained pending candidates.
    pub pending: usize,
    /// Age of the oldest retained pending candidate.
    pub oldest_pending_age: Duration,
    /// Overflow observations, not an exact distinct-file count.
    pub overflow_observations: u64,
    /// Passes with at least one overflow observation.
    pub overflowing_passes: u64,
    /// Last successful admission reported by the identity owner.
    pub last_admission: Option<Duration>,
}

struct Pass {
    generation: u64,
    started: Duration,
    count: u64,
    complete: bool,
    overflow: bool,
    environment_failed: bool,
    observed_at: Duration,
}

/// Single-owner state machine. Callbacks must return before subsequent operations.
///
/// Callback panics must terminate the owner; continuing after an unwound callback
/// is unsupported. Transport/channel integration must preserve this ordering.
pub struct Reconciler {
    limits: ReconcileLimits,
    tracked: Records,
    locators: RecordIndex<FileLocator>,
    bindings: RecordIndex<OsStr>,
    parents: RecordIndex<OsStr>,
    aliases: RecordIndex<OsStr>,
    candidates: Candidates,
    pass: Option<Pass>,
    last: Option<PassSummary>,
    stats: ReconcileStats,
    binding_keys: Vec<TrackedId>,
    binding_walk: Vec<TrackedId>,
    unresolved_rebounds: usize,
    binding_work: Vec<TrackedId>,
    binding_cursor: usize,
}

impl Reconciler {
    /// Validates all storage arithmetic before allocating population indexes.
    pub fn new(limits: ReconcileLimits) -> Result<Self, ReconcileError> {
        let limits = limits.validate()?;
        Ok(Self {
            limits,
            tracked: Records::new(limits.tracked),
            locators: RecordIndex::new(limits.tracked, |t| Some(&t.file.locator)),
            bindings: RecordIndex::new(limits.tracked, |t| {
                t.file.binding.as_ref().map(|p| p.as_os_str())
            }),
            parents: RecordIndex::new(limits.tracked, |t| {
                t.file
                    .binding
                    .as_ref()
                    .and_then(|p| p.parent())
                    .map(Path::as_os_str)
            }),
            aliases: RecordIndex::new(limits.tracked, |t| {
                (!t.minimum.as_os_str().is_empty()).then_some(t.minimum.as_os_str())
            }),
            candidates: Candidates::new(limits),
            pass: None,
            last: None,
            stats: ReconcileStats::default(),
            binding_keys: Vec::with_capacity(limits.tracked),
            binding_walk: Vec::with_capacity(limits.tracked),
            unresolved_rebounds: 0,
            binding_work: Vec::with_capacity(limits.tracked),
            binding_cursor: 0,
        })
    }

    /// Restores an identity-owner view during recovery/bootstrap between passes.
    /// Recovery may supply equal historical paths, but two live bindings to one
    /// path are rejected. For new admissions, use [`Self::admit`] or
    /// [`Self::admit_batch`] to preflight tracking before durable work. Do not
    /// commit a new admission first and then call this method: it can reject it.
    pub fn track(&mut self, mut file: TrackedFile) -> Result<(), ReconcileError> {
        self.validate_track(&file, None)?;
        file.binding = file.binding.as_ref().map(|path| path.to_path_buf());
        let id = file.id;
        self.insert_tracked(file);
        self.link_waiting_successors(id);
        Ok(())
    }

    fn validate_track(
        &self,
        file: &TrackedFile,
        ignore: Option<TrackedId>,
    ) -> Result<(), ReconcileError> {
        if self.pass.is_some() {
            return Err(ReconcileError::Phase);
        }
        if self.tracked.contains_key(&file.id) {
            return Err(ReconcileError::Tracked);
        }
        if self.tracked.len() == self.limits.tracked {
            return Err(ReconcileError::Capacity);
        }
        self.validate_binding(file, ignore)
    }

    fn validate_binding(
        &self,
        file: &TrackedFile,
        ignore: Option<TrackedId>,
    ) -> Result<(), ReconcileError> {
        if let Some(path) = &file.binding {
            if !super::native_path::valid_binding_path(path, self.limits.path_bytes) {
                return Err(ReconcileError::Tracked);
            }
            if file.active
                && file.role == BindingRole::Live
                && self
                    .bindings
                    .records(path.as_os_str(), &self.tracked)
                    .any(|tracked| {
                        tracked.file.id != file.id
                            && Some(tracked.file.id) != ignore
                            && tracked.file.active
                            && tracked.file.role == BindingRole::Live
                    })
            {
                return Err(ReconcileError::Tracked);
            }
        }
        Ok(())
    }

    fn parent_baseline(&self, path: Option<&Path>) -> (Option<FileLocator>, bool) {
        let Some(parent) = path.and_then(Path::parent) else {
            return (None, false);
        };
        Self::directory_baseline(&self.tracked, &self.parents, parent.as_os_str())
    }

    // observe_parent updates the entire family; insertion and rebind copy its
    // persistent state. Every member therefore agrees, including (None, changed).
    // Read one member rather than searching for a known baseline on every admission.
    fn directory_baseline(
        records: &Records,
        parents: &RecordIndex<OsStr>,
        path: &OsStr,
    ) -> (Option<FileLocator>, bool) {
        parents
            .records(path, records)
            .next()
            .map(|tracked| (tracked.parent, tracked.parent_changed))
            .unwrap_or((None, false))
    }

    // Parent identity is directory evidence, regardless of which file caused
    // the probe. Both traversal and targeted checks update the same indexed
    // family, preserving expected identities and per-pass publication blocks.
    fn observe_parent(
        &mut self,
        path: &Path,
        observed: Option<FileLocator>,
        force_changed: bool,
        emit: &mut impl FnMut(Evidence<'_>),
    ) -> bool {
        let (expected, _) =
            Self::directory_baseline(&self.tracked, &self.parents, path.as_os_str());
        let pass = self.pass.as_mut().expect("active parent observation");
        let mut changed = false;
        for id in self.parents.candidates(path.as_os_str()) {
            let tracked = self.tracked.get_mut(&id).expect("parent index");
            if !self.parents.matches(tracked, path.as_os_str()) {
                continue;
            }
            tracked.parent = tracked.parent.or(expected);
            if tracked.parent.is_none() && !force_changed {
                tracked.parent = observed;
            }
            tracked.pass.parent_seen |= observed.is_some();
            if force_changed
                || observed
                    .is_some_and(|actual| tracked.parent.is_some_and(|expected| actual != expected))
            {
                tracked.parent_changed = true;
                tracked.pass.parent_unstable = true;
                changed = true;
                pass.environment_failed = true;
                pass.complete = false;
                emit(Evidence::ParentChanged(id));
            } else if observed.is_some() {
                // Fresh restoration clears persistent state, never this pass's
                // evidence barrier: earlier observations may belong to the old view.
                tracked.parent_changed = false;
            }
        }
        // This check belongs on the already-linear family update, not the
        // per-admission lookup (which must also stay cheap in debug builds).
        debug_assert!({
            let state = Self::directory_baseline(&self.tracked, &self.parents, path.as_os_str());
            self.parents
                .records(path.as_os_str(), &self.tracked)
                .all(|tracked| (tracked.parent, tracked.parent_changed) == state)
        });
        if changed {
            emit(Evidence::CoverageLost(ScanReason::RootChanged));
        }
        changed
    }

    // Called only after preflight under an exclusive borrow. No fallible protocol
    // validation remains after the owner's durable commit.
    fn insert_tracked(&mut self, file: TrackedFile) {
        let (parent, parent_changed) = self.parent_baseline(file.binding.as_deref());
        let id = file.id;
        self.tracked.insert(
            id,
            Tracked {
                file,
                pass: TrackedPass::default(),
                minimum: PathBuf::new(),
                previous_alias: PathBuf::new(),
                parent,
                parent_changed,
                successor: None,
            },
        );
        self.locators.insert(id, &self.tracked);
        self.bindings.insert(id, &self.tracked);
        self.parents.insert(id, &self.tracked);
    }

    fn link_waiting_successors(&mut self, id: TrackedId) {
        let Some(path) = self.tracked[&id].file.binding.as_ref() else {
            return;
        };
        // Between passes the graph-key scratch is free for ID-only lookup work.
        self.binding_keys.clear();
        self.binding_keys
            .extend(self.bindings.get(path.as_os_str(), &self.tracked));
        for index in 0..self.binding_keys.len() {
            let predecessor = self.binding_keys[index];
            let tracked = &self.tracked[&predecessor];
            if tracked.file.role == BindingRole::Superseded
                && tracked.successor.is_none()
                && tracked.pass.binding_target.is_resolved()
                && tracked.pass.binding_target.rebound() == Some(self.tracked[&id].file.locator)
            {
                self.link_successor(predecessor, id);
            }
        }
        self.binding_keys.clear();
    }

    /// Removes only an explicitly retired identity-owner view, never scan-derived state.
    pub fn untrack(&mut self, id: TrackedId) -> Result<TrackedFile, ReconcileError> {
        if self.pass.is_some() {
            return Err(ReconcileError::Phase);
        }
        if !self.tracked.contains_key(&id) {
            return Err(ReconcileError::Tracked);
        }
        self.locators.remove(id, &self.tracked);
        self.bindings.remove(id, &self.tracked);
        self.parents.remove(id, &self.tracked);
        self.aliases.remove(id, &self.tracked);
        let tracked = self.tracked.remove(&id).expect("validated record");
        if tracked.pass.binding_target.pending().is_some() {
            self.unresolved_rebounds -= 1;
        }
        Ok(tracked.file)
    }

    /// Starts a strictly newer pass. `seed` varies overflow opportunity independently
    /// of traversal order; tests should inject reproducible seeds.
    pub fn begin(
        &mut self,
        generation: u64,
        started: Duration,
        seed: u64,
    ) -> Result<(), ReconcileError> {
        if self.pass.is_some() {
            return Err(ReconcileError::Phase);
        }
        if let Some(last) = self.last {
            if generation <= last.generation {
                return Err(ReconcileError::Generation);
            }
            if started < last.finished {
                return Err(ReconcileError::Time);
            }
        }
        self.candidates.begin(generation, seed);
        self.aliases.clear();
        self.binding_keys.clear();
        self.binding_work.clear();
        self.binding_cursor = 0;
        self.unresolved_rebounds = 0;
        for tracked in self.tracked.values_mut() {
            if !tracked.minimum.as_os_str().is_empty() {
                std::mem::swap(&mut tracked.previous_alias, &mut tracked.minimum);
            }
            tracked.minimum.clear();
            tracked.pass = TrackedPass {
                rebound_parent: tracked.file.id.0,
                ..TrackedPass::default()
            };
            // Parent identity changes persist until the expected environment returns.
        }
        self.pass = Some(Pass {
            generation,
            started,
            count: 0,
            complete: true,
            overflow: false,
            // A previous pass may have seen a replaced parent that this traversal
            // never reaches. Do not allow prompt negatives while that latch exists.
            environment_failed: self.tracked.values().any(|t| t.parent_changed),
            observed_at: started,
        });
        Ok(())
    }

    // Invalidate only evidence connected by retained native paths or locators.
    // This preserves independent candidates on an otherwise incomplete pass.
    fn invalidate_dependent_evidence(&mut self, observation: &Observation) {
        self.candidates.invalidate(observation);
        // The indexes borrow stored keys; only evidence flags change here.
        for index in [&self.bindings, &self.aliases] {
            for id in index.candidates(observation.path.as_os_str()) {
                let tracked = self.tracked.get_mut(&id).expect("path index");
                if index.matches(tracked, observation.path.as_os_str()) {
                    if let Some(replacement) = tracked.pass.binding_target.rebound() {
                        self.candidates.invalidate_locator(replacement);
                    }
                    tracked.pass.conflict = true;
                    tracked.pass.exclusion_blocked = true;
                }
            }
        }
        if let Some(locator) = observation.locator {
            for id in self.locators.candidates(&locator) {
                let tracked = self.tracked.get_mut(&id).expect("locator index");
                if tracked.file.locator == locator {
                    tracked.pass.conflict = true;
                    tracked.pass.exclusion_blocked = true;
                }
            }
        }
    }

    /// Consumes one borrowed scanner observation. Callback delivery is immediate and
    /// ordered. To cancel, stop feeding observations and finish with a cancelled marker.
    /// Protocol/shape errors poison completeness before returning an error.
    pub fn observe(
        &mut self,
        generation: u64,
        observation: &Observation,
        observed_at: Duration,
        emit: impl FnMut(Evidence<'_>),
    ) -> Result<(), ReconcileError> {
        self.consume(generation, observation, observed_at, Origin::Scan, emit)
    }

    fn consume(
        &mut self,
        generation: u64,
        observation: &Observation,
        observed_at: Duration,
        origin: Origin,
        mut emit: impl FnMut(Evidence<'_>),
    ) -> Result<(), ReconcileError> {
        let pass = self.pass.as_mut().ok_or(ReconcileError::Phase)?;
        if generation != pass.generation {
            pass.complete = false;
            emit(Evidence::CoverageLost(ScanReason::Coverage));
            return Err(ReconcileError::Generation);
        }
        if observed_at < pass.observed_at {
            pass.complete = false;
            emit(Evidence::CoverageLost(ScanReason::Coverage));
            return Err(ReconcileError::Time);
        }
        pass.observed_at = observed_at;
        if !valid_observation(observation, self.limits) {
            pass.complete = false;
            emit(Evidence::CoverageLost(ScanReason::Limit));
            return Err(ReconcileError::Observation);
        }
        if origin == Origin::Scan {
            let Some(count) = pass.count.checked_add(1) else {
                pass.complete = false;
                emit(Evidence::CoverageLost(ScanReason::Limit));
                return Err(ReconcileError::Exhausted);
            };
            pass.count = count;
            if let Some(reason) = observation
                .reason
                .filter(|reason| reason.invalidates_coverage())
            {
                pass.complete = false;
                pass.environment_failed |= matches!(
                    reason,
                    ScanReason::RootChanged
                        | ScanReason::RootUnavailable
                        | ScanReason::ExclusionUnavailable
                );
                if reason == ScanReason::RootChanged {
                    emit(Evidence::RootChanged(&observation.path));
                }
                emit(Evidence::CoverageLost(reason));
            }
        }
        if let Some(
            reason @ (ScanReason::Unstable
            | ScanReason::Policy
            | ScanReason::VerificationUnavailable),
        ) = observation.reason
        {
            pass.complete = false;
            self.invalidate_dependent_evidence(observation);
            if origin == Origin::Binding {
                emit(Evidence::CoverageLost(reason));
            }
        }
        if observation.reason == Some(ScanReason::Directory) {
            let _ = self.observe_parent(&observation.path, observation.locator, false, &mut emit);
            return Ok(());
        }
        let pass = self.pass.as_mut().expect("validated active pass");
        // A contextual miss is useful only with current, unchanged parent evidence.
        // Never turn an arbitrary ENOENT into a binding revocation.
        if origin == Origin::Scan && observation.reason == Some(ScanReason::Missing) {
            for id in self.bindings.candidates(observation.path.as_os_str()) {
                let tracked = self.tracked.get_mut(&id).expect("binding index");
                if !self.bindings.matches(tracked, observation.path.as_os_str()) {
                    continue;
                }
                if tracked.pass.parent_seen && !tracked.parent_changed && !pass.environment_failed {
                    tracked.pass.binding_ineligible.record_miss();
                    emit_exclusion(tracked, pass.environment_failed, &mut emit);
                }
            }
        }
        let Some(locator) = observation.locator else {
            return Ok(());
        };
        let live_owner = self
            .locators
            .records(&locator, &self.tracked)
            .find(|tracked| tracked.file.active && tracked.file.role == BindingRole::Live)
            .map(|tracked| tracked.file.id);
        let evidence = NameEvidence::from_observation(observation);
        let eligible = evidence.is_full();
        let mut path_conflict = false;
        if evidence.is_named() {
            path_conflict = self.candidates.check_path(observation);
            for id in self.aliases.candidates(observation.path.as_os_str()) {
                let tracked = self.tracked.get_mut(&id).expect("alias index");
                if self.aliases.matches(tracked, observation.path.as_os_str())
                    && tracked.file.locator != locator
                {
                    tracked.pass.conflict = true;
                    path_conflict = true;
                }
            }
            for id in self.bindings.candidates(observation.path.as_os_str()) {
                let tracked = self.tracked.get_mut(&id).expect("binding index");
                if self.bindings.matches(tracked, observation.path.as_os_str())
                    && tracked.file.active
                    && tracked.file.locator != locator
                {
                    if tracked.pass.binding_evidence.is_named()
                        || tracked
                            .pass
                            .binding_target
                            .observed()
                            .is_some_and(|prior| prior != locator)
                    {
                        tracked.pass.conflict = true;
                        path_conflict = true;
                    }
                    if !eligible {
                        tracked.pass.parent_unstable |= tracked.parent_changed;
                        tracked.pass.binding_target.observe(locator);
                    }
                }
            }
            if path_conflict {
                pass.complete = false;
                emit(Evidence::CoverageLost(ScanReason::Unstable));
            }
        }
        let mut resident = false;
        {
            for id in self.locators.candidates(&locator) {
                let tracked = self.tracked.get_mut(&id).expect("locator index");
                if tracked.file.locator != locator {
                    continue;
                }
                if !tracked.pass.present {
                    emit(Evidence::Present(id));
                }
                tracked.pass.present = true;
                if observation
                    .reason
                    .is_some_and(ScanReason::invalidates_coverage)
                {
                    // Resource/cancellation failures can retain a locator without
                    // proving policy. Preserve uncertainty, never grant authority.
                    tracked.pass.exclusion_blocked = true;
                }
                if observation.reason == Some(ScanReason::Excluded) {
                    tracked.pass.excluded = true;
                    if !tracked.pass.check_queued {
                        tracked.pass.check_queued = true;
                        self.binding_work.push(id);
                    }
                    if tracked
                        .file
                        .binding
                        .as_ref()
                        .is_some_and(|path| path.as_os_str() == observation.path.as_os_str())
                    {
                        tracked.pass.parent_unstable |= tracked.parent_changed;
                        tracked.pass.binding_ineligible = BindingIneligibility::Positive;
                    }
                    emit_exclusion(tracked, pass.environment_failed, &mut emit);
                }
                if evidence.is_named() {
                    tracked.pass.conflict |= path_conflict;
                    resident |= tracked.file.resident;
                    let at_binding = tracked
                        .file
                        .binding
                        .as_ref()
                        .is_some_and(|p| p.as_os_str() == observation.path.as_os_str());
                    if !at_binding
                        && (evidence > tracked.pass.minimum_evidence
                            || (evidence == tracked.pass.minimum_evidence
                                && (tracked.minimum.as_os_str().is_empty()
                                    || observation.path.as_os_str() < tracked.minimum.as_os_str())))
                    {
                        let path_changed =
                            tracked.minimum.as_os_str() != observation.path.as_os_str();
                        if path_changed {
                            // Rekey before changing bytes. Equal-path upgrades
                            // change only strength and never duplicate the index.
                            self.aliases.remove(id, &self.tracked);
                            replace_path(
                                &mut self.tracked.get_mut(&id).expect("alias owner").minimum,
                                &observation.path,
                                self.limits.path_bytes,
                            );
                            self.aliases.insert(id, &self.tracked);
                        }
                        self.tracked
                            .get_mut(&id)
                            .expect("alias owner")
                            .pass
                            .minimum_evidence = evidence;
                    }
                    let tracked = self.tracked.get_mut(&id).expect("locator index");
                    if at_binding {
                        tracked.pass.parent_unstable |= tracked.parent_changed;
                        tracked.pass.binding_evidence.merge(evidence);
                        if tracked.pass.binding_target.observed().is_some() {
                            tracked.pass.conflict = true;
                            pass.complete = false;
                            emit(Evidence::CoverageLost(ScanReason::Unstable));
                        }
                    }
                    if eligible {
                        emit(Evidence::Validated(id, observation));
                    }
                }
            }
        }
        let mut replacement = Replacement::Unrelated;
        if eligible {
            {
                for id in self.bindings.candidates(observation.path.as_os_str()) {
                    let tracked = self.tracked.get_mut(&id).expect("binding index");
                    if !self.bindings.matches(tracked, observation.path.as_os_str()) {
                        continue;
                    }
                    if !tracked.file.active || tracked.file.locator == locator {
                        continue;
                    }
                    if tracked.pass.binding_evidence.is_named()
                        || tracked
                            .pass
                            .binding_target
                            .observed()
                            .is_some_and(|prior| prior != locator)
                    {
                        tracked.pass.conflict = true;
                        pass.complete = false;
                        emit(Evidence::CoverageLost(ScanReason::Unstable));
                    }
                    tracked.pass.parent_unstable |= tracked.parent_changed;
                    let resolved = tracked.file.role == BindingRole::Superseded
                        && live_owner.is_some_and(|id| tracked.successor == Some((id, locator)));
                    let new_rebound = tracked.pass.binding_target.qualify(locator, resolved);
                    tracked.pass.binding_ineligible = BindingIneligibility::Positive;
                    // Historical paths retain replacement context but do not create
                    // competing live claims after repeated incomplete rotations.
                    replacement = replacement.merge(Replacement::Possible(id));
                    if new_rebound
                        && pass.complete
                        && Self::detect_new_rebound(
                            &mut self.tracked,
                            &self.locators,
                            &mut self.binding_walk,
                            id,
                        )
                    {
                        pass.complete = false;
                        emit(Evidence::CoverageLost(ScanReason::Unstable));
                    }
                    emit_exclusion(
                        self.tracked.get_mut(&id).expect("binding index"),
                        pass.environment_failed,
                        &mut emit,
                    );
                }
            }
            if !resident {
                let result = match self.candidates.observe(
                    observation,
                    observed_at,
                    replacement,
                    path_conflict,
                ) {
                    Ok(result) => result,
                    Err(error) => {
                        pass.complete = false;
                        emit(Evidence::CoverageLost(ScanReason::Limit));
                        return Err(error);
                    }
                };
                if result.conflict && !path_conflict {
                    pass.complete = false;
                    emit(Evidence::CoverageLost(ScanReason::Unstable));
                }
                if result.overflow {
                    pass.overflow = true;
                    self.stats.overflow_observations =
                        self.stats.overflow_observations.saturating_add(1);
                }
            }
        }
        Ok(())
    }

    /// IDs needing targeted evidence. Re-query `binding_request` after each result:
    /// a validated earlier alias can eliminate the distinguished-path check.
    pub fn binding_checks(&self) -> impl Iterator<Item = TrackedId> + '_ {
        self.binding_work
            .iter()
            .copied()
            .filter(|id| self.binding_request(*id).is_some())
    }

    /// Drives the preallocated worklist without rescanning tracked records.
    /// Apply the result before requesting again. An identity can require an alias
    /// check followed by a binding check; exhausted entries are skipped once.
    pub fn next_binding_check(&mut self) -> Option<TrackedId> {
        while let Some(&id) = self.binding_work.get(self.binding_cursor) {
            if self.binding_request(id).is_some() {
                return Some(id);
            }
            self.binding_cursor += 1;
        }
        None
    }

    /// Abandons this identity's remaining targeted checks after a scanner request
    /// cannot be evaluated. Preserve uncertainty, invalidate completion, and let
    /// the cursor advance. Normal scanner outcomes belong in apply_binding_check.
    /// No exclusion or disappearance can be inferred from a rejected request.
    pub fn skip_binding_checks(
        &mut self,
        generation: u64,
        id: TrackedId,
        mut emit: impl FnMut(Evidence<'_>),
    ) -> Result<(), ReconcileError> {
        let pass = self.pass.as_ref().ok_or(ReconcileError::Phase)?;
        if pass.generation != generation {
            return Err(ReconcileError::Generation);
        }
        if self.binding_request(id).is_none() {
            return Err(ReconcileError::Feedback);
        }
        let tracked = self
            .tracked
            .get_mut(&id)
            .expect("validated binding request");
        tracked.pass.alias_checked = true;
        tracked.pass.binding_checked = true;
        tracked.pass.exclusion_blocked = true;
        self.pass.as_mut().expect("active pass").complete = false;
        emit(Evidence::CoverageLost(ScanReason::Coverage));
        Ok(())
    }

    /// Borrows the next request. `BindingBatch::check` returns scanner-owned evidence,
    /// so this borrow ends before `apply_binding_check` mutates reconciliation.
    #[must_use]
    pub fn binding_request(&self, id: TrackedId) -> Option<BindingRequest<'_>> {
        let t = self.tracked.get(&id)?;
        if self.pass.is_none()
            || !t.file.active
            || !t.pass.excluded
            || t.pass.name_evidence().is_named()
        {
            return None;
        }
        if !t.previous_alias.as_os_str().is_empty() && !t.pass.alias_checked {
            return Some(BindingRequest {
                path: &t.previous_alias,
                parent: None,
            });
        }
        if !t.pass.binding_checked && t.pass.binding_ineligible == BindingIneligibility::Unknown {
            return t.file.binding.as_deref().map(|path| BindingRequest {
                path,
                parent: t.parent,
            });
        }
        None
    }

    /// Applies a binding-batch root-refresh failure even when no file requests
    /// are queued. This is not a traversal observation and changes no counters.
    pub fn apply_binding_batch_error(
        &mut self,
        generation: u64,
        error: &super::BindingBatchError<'_>,
        mut emit: impl FnMut(Evidence<'_>),
    ) -> Result<(), ReconcileError> {
        let pass = self.pass.as_mut().ok_or(ReconcileError::Phase)?;
        if pass.generation != generation {
            return Err(ReconcileError::Generation);
        }
        pass.complete = false;
        pass.environment_failed = true;
        if error.reason == ScanReason::RootChanged
            && let Some(root) = error.root
        {
            emit(Evidence::RootChanged(root));
        }
        emit(Evidence::CoverageLost(error.reason));
        Ok(())
    }

    /// Applies targeted evidence without adding to the traversal count. Binding-local
    /// uncertainty without a retained locator permits complete-pass fallback;
    /// environment or required policy failures retain their receiver-wide effect.
    pub fn apply_binding_check(
        &mut self,
        generation: u64,
        id: TrackedId,
        check: super::BindingCheck<'_>,
        observed_at: Duration,
        mut emit: impl FnMut(Evidence<'_>),
    ) -> Result<(), ReconcileError> {
        use super::BindingScope;
        let pass = self.pass.as_mut().ok_or(ReconcileError::Phase)?;
        if pass.generation != generation {
            return Err(ReconcileError::Generation);
        }
        let tracked = self.tracked.get_mut(&id).ok_or(ReconcileError::Tracked)?;
        let binding = tracked
            .file
            .binding
            .as_ref()
            .is_some_and(|p| p.as_os_str() == check.observation.path.as_os_str());
        let alias = !tracked.previous_alias.as_os_str().is_empty()
            && tracked.previous_alias.as_os_str() == check.observation.path.as_os_str();
        if !binding && !alias {
            return Err(ReconcileError::Feedback);
        }
        if observed_at < pass.observed_at || !valid_observation(check.observation, self.limits) {
            pass.complete = false;
            emit(Evidence::CoverageLost(ScanReason::Coverage));
            return Err(ReconcileError::Observation);
        }
        pass.observed_at = observed_at;
        if binding {
            tracked.pass.binding_checked = true;
        }
        if alias {
            tracked.pass.alias_checked = true;
        }
        if check.scope == BindingScope::Coverage
            || (alias
                && check
                    .observation
                    .reason
                    .is_some_and(ScanReason::invalidates_coverage))
        {
            tracked.pass.exclusion_blocked = true;
            pass.complete = false;
            let reason = check.observation.reason.unwrap_or(ScanReason::Coverage);
            pass.environment_failed |= matches!(
                reason,
                ScanReason::RootChanged
                    | ScanReason::RootUnavailable
                    | ScanReason::ExclusionUnavailable
            );
            if reason == ScanReason::RootChanged {
                emit(Evidence::RootChanged(check.diagnostic_path));
            }
            emit(Evidence::CoverageLost(reason));
            // Keep locator presence, without turning a root error into a parent latch.
            return self.consume(
                generation,
                check.observation,
                observed_at,
                Origin::Binding,
                emit,
            );
        }
        if binding {
            let reported_change = check.scope == BindingScope::ParentChanged
                || check.parent.is_some_and(|parent| {
                    tracked.parent.is_some_and(|expected| expected != parent)
                });
            if reported_change && tracked.parent_changed && tracked.pass.parent_seen {
                // This pass already propagated a changed parent to the whole
                // family. Repeated failed checks must not rescan it per file.
                return Ok(());
            }
            let refresh_peers = reported_change
                || (check.parent.is_some() && (tracked.parent.is_none() || tracked.parent_changed));
            if refresh_peers {
                let parent = check
                    .observation
                    .path
                    .parent()
                    .expect("validated binding parent");
                if self.observe_parent(parent, check.parent, reported_change, &mut emit) {
                    return Ok(());
                }
            } else if check.parent.is_some() {
                // Healthy checks with an established baseline need no family
                // walk. Initial validation already seeds unknown peers together.
                self.tracked
                    .get_mut(&id)
                    .expect("checked binding")
                    .pass
                    .parent_seen = true;
            }
        }
        let unchanged = check.parent.is_some() && check.parent == self.tracked[&id].parent;
        self.consume(
            generation,
            check.observation,
            observed_at,
            Origin::Binding,
            &mut emit,
        )?;
        let environment_failed = self.pass.as_ref().expect("active pass").environment_failed;
        let tracked = self
            .tracked
            .get_mut(&id)
            .expect("retained binding identity");
        if alias
            && check.scope == BindingScope::Binding
            && (check.observation.reason == Some(ScanReason::Missing)
                || (check.observation.reason.is_none()
                    && check.observation.locator != Some(tracked.file.locator)))
        {
            tracked.previous_alias.clear();
        }
        if binding {
            if check.observation.reason == Some(ScanReason::Excluded) {
                tracked.pass.binding_ineligible = BindingIneligibility::Positive;
            } else if !environment_failed
                && check.context_valid
                && unchanged
                && check.observation.reason == Some(ScanReason::Missing)
            {
                tracked.pass.binding_ineligible.record_miss();
            }
        }
        emit_exclusion(tracked, environment_failed, &mut emit);
        Ok(())
    }

    /// Supplies narrowly associated replacement context derived by the identity
    /// owner from its bounded advisory-path evidence (including truncated paths).
    /// This does not validate continuity, establish a live binding, or associate
    /// unrelated candidates merely because another path is unavailable.
    pub fn associate_replacement(
        &mut self,
        generation: u64,
        predecessor: TrackedId,
        candidate: FileLocator,
        mut emit: impl FnMut(Evidence<'_>),
    ) -> Result<(), ReconcileError> {
        let pass = self.pass.as_mut().ok_or(ReconcileError::Phase)?;
        if pass.generation != generation {
            return Err(ReconcileError::Generation);
        }
        let tracked = self
            .tracked
            .get_mut(&predecessor)
            .ok_or(ReconcileError::Tracked)?;
        if !tracked.file.active || tracked.file.locator == candidate {
            return Err(ReconcileError::Feedback);
        }
        self.candidates.associate(candidate, predecessor)?;
        tracked.pass.parent_unstable |= tracked.parent_changed;
        if tracked.pass.binding_evidence.is_named()
            || tracked
                .pass
                .binding_target
                .observed()
                .is_some_and(|prior| prior != candidate)
        {
            tracked.pass.conflict = true;
            pass.complete = false;
            emit(Evidence::CoverageLost(ScanReason::Unstable));
        }
        let resolved = tracked.pass.binding_target.is_resolved()
            && tracked.pass.binding_target.rebound() == Some(candidate);
        let new_rebound = tracked.pass.binding_target.qualify(candidate, resolved);
        if new_rebound
            && pass.complete
            && Self::detect_new_rebound(
                &mut self.tracked,
                &self.locators,
                &mut self.binding_walk,
                predecessor,
            )
        {
            pass.complete = false;
            emit(Evidence::CoverageLost(ScanReason::Unstable));
        }
        Ok(())
    }

    /// Consumes the exact scanner marker after all observations. Required conflict
    /// or environment evidence can make a successful traversal incomplete.
    /// Pending overflow alone does not affect coverage.
    pub fn finish(
        &mut self,
        completion: ScanCompletion,
        finished: Duration,
        mut emit: impl FnMut(Evidence<'_>),
    ) -> Result<PassSummary, ReconcileError> {
        let pass = self.pass.as_mut().ok_or(ReconcileError::Phase)?;
        let error = if completion.generation != pass.generation {
            Some(ReconcileError::Generation)
        } else if completion.observations != pass.count {
            Some(ReconcileError::Count)
        } else if finished < pass.observed_at {
            Some(ReconcileError::Time)
        } else {
            None
        };
        if let Some(error) = error {
            pass.complete = false;
            emit(Evidence::CoverageLost(ScanReason::Coverage));
            return Err(error);
        }
        let pass = self.pass.take().expect("checked active pass");
        self.detect_binding_cycles();
        self.candidates.block_conflicts(|candidate| {
            // An offer can reconnect an existing record even when another
            // historical binding at this path has a parent block. Admission
            // separately rejects new identity publication at any blocked path.
            let mut reconnect = false;
            for tracked in self.locators.records(&candidate.locator, &self.tracked) {
                if tracked.file.active {
                    reconnect = true;
                    if tracked.pass.conflict {
                        return true;
                    }
                }
            }
            self.bindings
                .records(candidate.path.as_os_str(), &self.tracked)
                .any(|tracked| {
                    tracked.file.active
                        && (tracked.pass.conflict || (!reconnect && tracked.binding_blocked()))
                })
        });
        self.unresolved_rebounds = self
            .tracked
            .values()
            .filter(|t| t.pass.binding_target.pending().is_some())
            .count();
        let complete = pass.complete
            && completion.complete
            && !completion.cancelled
            && !self.candidates.has_conflict()
            && self
                .tracked
                .values()
                .all(|tracked| !tracked.pass.conflict && !tracked.binding_blocked());
        if !complete {
            emit(Evidence::CoverageLost(ScanReason::Coverage));
        }
        // Only per-locator order is promised; unrelated identities have no global
        // ordering. No consumer may interpret these as a finalization command.
        for tracked in self.tracked.values() {
            if complete && tracked.file.active {
                if tracked.pass.excluded
                    && !tracked.pass.name_evidence().is_named()
                    && !tracked.pass.exclusion_blocked
                    && !tracked.pass.exclusion_sent
                {
                    emit(Evidence::Excluded(tracked.file.id));
                } else if !tracked.pass.present {
                    emit(Evidence::Disappeared(tracked.file.id));
                }
            }
        }
        let summary = PassSummary {
            generation: pass.generation,
            started: pass.started,
            finished,
            complete,
            observations: pass.count,
        };
        self.candidates.finish(
            complete,
            self.tracked.len() < self.limits.tracked,
            |locator| {
                self.locators
                    .records(&locator, &self.tracked)
                    .any(|tracked| tracked.file.active)
            },
        );
        if pass.overflow {
            self.stats.overflowing_passes = self.stats.overflowing_passes.saturating_add(1);
        }
        self.last = Some(summary);
        emit(Evidence::Completed(summary));
        Ok(summary)
    }

    /// Ends a failed/cancelled consumer pass without pretending unconsumed scanner
    /// observations were processed. Positive delivered evidence remains valid.
    pub fn abort(
        &mut self,
        finished: Duration,
        emit: impl FnMut(Evidence<'_>),
    ) -> Result<PassSummary, ReconcileError> {
        let pass = self.pass.as_ref().ok_or(ReconcileError::Phase)?;
        let completion = ScanCompletion {
            generation: pass.generation,
            complete: false,
            cancelled: true,
            observations: pass.count,
        };
        self.finish(completion, finished, emit)
    }

    /// Borrows the selected current-pass offers. Tracked capacity is intentionally
    /// not consulted: identity resolution may reconnect an existing record.
    pub fn offers(&self) -> Result<impl Iterator<Item = CandidateOffer<'_>>, ReconcileError> {
        if self.pass.is_some() || self.last.is_none() {
            return Err(ReconcileError::Phase);
        }
        Ok(self.candidates.offers())
    }

    /// Records offer feedback without validating or publishing tracked state.
    /// Deferred candidates preserve first discovery time; Admitted removes the
    /// pending candidate. New admissions must use [`Self::admit`] or
    /// [`Self::admit_batch`], which publish tracking and acknowledge together.
    /// This lower-level feedback method is not an admission commit boundary.
    pub fn acknowledge(
        &mut self,
        token: OfferToken,
        feedback: IdentityFeedback,
        admitted_at: Duration,
    ) -> Result<(), ReconcileError> {
        if self.pass.is_some() {
            return Err(ReconcileError::Phase);
        }
        self.candidates.acknowledge(token, feedback)?;
        if feedback == IdentityFeedback::Admitted {
            self.stats.last_admission = Some(admitted_at);
        }
        Ok(())
    }

    /// Resolves offer feedback under an exclusive borrow after validating its token.
    /// Stale offers never invoke the callback, and no new pass can start during it.
    /// This does not preflight tracked-state publication, including capacity and
    /// binding conflicts. Use [`Self::admit`] or [`Self::admit_batch`] for new
    /// admissions; use this method for handling that needs only offer feedback.
    pub fn resolve_offer(
        &mut self,
        token: OfferToken,
        resolve: impl FnOnce(CandidateOffer<'_>) -> (IdentityFeedback, Duration),
    ) -> Result<IdentityFeedback, ReconcileError> {
        if self.pass.is_some() {
            return Err(ReconcileError::Phase);
        }
        let offer = self.candidates.offer(token)?;
        let (feedback, admitted_at) = resolve(offer);
        self.acknowledge(token, feedback, admitted_at)?;
        Ok(feedback)
    }

    fn check_finished(&self, generation: u64) -> Result<(), ReconcileError> {
        if self.pass.is_some() {
            return Err(ReconcileError::Phase);
        }
        if self.last.is_none_or(|last| last.generation != generation) {
            return Err(ReconcileError::Generation);
        }
        Ok(())
    }

    /// Updates reader/lifecycle eligibility after an explicit identity-owner decision.
    /// Terminal state stops claiming a live binding but keeps presence accounting.
    /// No source offset or durable lifecycle is changed by this local view update.
    pub fn update_reader_state(
        &mut self,
        generation: u64,
        id: TrackedId,
        active: bool,
        resident: bool,
    ) -> Result<(), ReconcileError> {
        self.check_finished(generation)?;
        let tracked = self.tracked.get(&id).ok_or(ReconcileError::Tracked)?;
        if active
            && tracked.file.role == BindingRole::Live
            && tracked.file.binding.as_ref().is_some_and(|path| {
                self.bindings
                    .records(path.as_os_str(), &self.tracked)
                    .any(|other| {
                        other.file.id != id
                            && other.file.active
                            && other.file.role == BindingRole::Live
                    })
            })
        {
            return Err(ReconcileError::Tracked);
        }
        let tracked = self
            .tracked
            .get_mut(&id)
            .expect("validated tracked identity");
        tracked.file.active = active;
        tracked.file.resident = resident;
        Ok(())
    }

    /// Reads the identity-owner view without copying its path.
    #[must_use]
    pub fn tracked(&self, id: TrackedId) -> Option<&TrackedFile> {
        self.tracked.get(&id).map(|t| &t.file)
    }

    /// Population and bounded-counter snapshot. `now` is caller monotonic time.
    #[must_use]
    pub fn stats(&self, now: Duration) -> ReconcileStats {
        ReconcileStats {
            tracked: self.tracked.len(),
            pending: self.candidates.pending_len(),
            oldest_pending_age: self.candidates.oldest_age(now),
            ..self.stats
        }
    }

    /// Conservative owned-storage accounting, including unused index capacity.
    /// Hash table control bytes/allocator overhead are estimated, not exact RSS.
    #[must_use]
    pub fn retained_bytes(&self) -> usize {
        size_of::<Self>()
            + self.tracked.retained_bytes()
            + self
                .tracked
                .values()
                .map(|t| {
                    t.minimum.capacity()
                        + t.previous_alias.capacity()
                        + t.file.binding.as_ref().map_or(0, |p| p.capacity())
                })
                .sum::<usize>()
            + self.locators.retained_bytes()
            + self.bindings.retained_bytes()
            + self.parents.retained_bytes()
            + self.aliases.retained_bytes()
            + self.candidates.retained_bytes()
            + (self.binding_keys.capacity()
                + self.binding_walk.capacity()
                + self.binding_work.capacity())
                * size_of::<TrackedId>()
    }
}

fn emit_exclusion(
    tracked: &mut Tracked,
    environment_failed: bool,
    emit: &mut impl FnMut(Evidence<'_>),
) {
    if tracked.file.active
        && tracked.pass.excluded
        && tracked
            .pass
            .binding_ineligible
            .allows_exclusion(environment_failed)
        && !tracked.pass.name_evidence().is_named()
        && !tracked.pass.conflict
        && !tracked.binding_blocked()
        && !tracked.pass.exclusion_sent
        && !tracked.pass.exclusion_blocked
        && (tracked.previous_alias.as_os_str().is_empty() || tracked.pass.alias_checked)
    {
        tracked.pass.exclusion_sent = true;
        emit(Evidence::Excluded(tracked.file.id));
    }
}

// Reuse buffers, but never let geometric growth exceed the native-path limit.
// A replacement may transiently own old + new storage; admission charges one
// extra path buffer globally. Short paths are allocated lazily, not padded to L.
fn replace_path(buffer: &mut PathBuf, path: &Path, limit: usize) {
    let bytes = path.as_os_str().len();
    if buffer.capacity() < bytes {
        let capacity = buffer.capacity().saturating_mul(2).max(bytes).min(limit);
        *buffer = PathBuf::with_capacity(capacity);
    }
    buffer.clear();
    buffer.push(path);
}

// Measured against Rust 1.98.1; revalidate these assumptions and heap tests on
// toolchain upgrades. These are not guarantees of the Rust HashMap API.
// The pinned standard HashMap uses power-of-two buckets and 7/8 occupancy.
// capacity() excludes tombstones, so it cannot report allocated storage after
// removals. Bound from the construction reservation instead. At most one growth
// is possible under fixed occupancy; after doubling, deletion rehashes in place.
fn table_bytes<K, V>(reserved: usize, churn: bool) -> usize {
    if reserved == 0 {
        return 0;
    }
    let buckets = (reserved * 8).div_ceil(7).next_power_of_two().max(4);
    let buckets = if churn && reserved > buckets / 2 + 1 {
        buckets * 2
    } else {
        buckets
    };
    buckets * (size_of::<(K, V)>() + 1) + 128
}

fn valid_path(path: &Path, limit: usize) -> bool {
    super::native_path::valid_absolute_path(path, limit)
}

fn valid_observation(observation: &Observation, limits: ReconcileLimits) -> bool {
    valid_path(&observation.path, limits.path_bytes)
        && observation.fingerprint.len() <= limits.fingerprint_bytes
        && if observation.reason.is_none() {
            super::native_path::valid_binding_path(&observation.path, limits.path_bytes)
                && observation.locator.is_some()
                && observation
                    .size
                    .is_some_and(|size| observation.fingerprint.len() as u64 <= size)
                && observation.errno.is_none()
        } else {
            observation.size.is_none()
                && observation.modified.is_none()
                && observation.fingerprint.is_empty()
                && (!matches!(
                    observation.reason,
                    Some(ScanReason::PresenceOnly | ScanReason::Content)
                ) || super::native_path::valid_binding_path(
                    &observation.path,
                    limits.path_bytes,
                ))
                && (!matches!(
                    observation.reason,
                    Some(ScanReason::Directory | ScanReason::PresenceOnly | ScanReason::Content)
                ) || observation.locator.is_some())
        }
}

/// Computes a completion-based delay with caller-supplied independent randomness.
/// No timer or worker is created. The caller must check addition to its clock.
/// At 100% jitter the result can be zero; configuration must exclude this when
/// the scheduler requires a positive delay (the receiver policy caps jitter at 25%).
pub fn reconcile_delay(
    interval: Duration,
    jitter_percent: u8,
    sample: u64,
) -> Result<Duration, ReconcileError> {
    if interval.is_zero() || jitter_percent > 100 {
        return Err(ReconcileError::Delay);
    }
    let base = u64::try_from(interval.as_nanos()).map_err(|_| ReconcileError::Delay)?;
    let spread = base
        .checked_mul(u64::from(jitter_percent))
        .ok_or(ReconcileError::Delay)?
        / 100;
    let width = spread
        .checked_mul(2)
        .and_then(|n| n.checked_add(1))
        .ok_or(ReconcileError::Delay)?;
    let delay = (base - spread)
        .checked_add(sample % width)
        .ok_or(ReconcileError::Delay)?;
    Ok(Duration::from_nanos(delay))
}
