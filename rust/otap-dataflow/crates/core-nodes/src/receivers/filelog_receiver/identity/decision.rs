// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Identity policy without filesystem, lease or checkpoint side effects.

use super::super::config::{RecoveryMismatch, StartAt};
use otel_arrow_dfe_filelog_checkpoint::{FileId, LifecycleState, SnapshotRecord};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

/// Start-policy context; never authority to inherit another file's progress.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Context {
    /// No relationship to retained Active state.
    Unrelated,
    /// Saved path evidence associates a changed locator without recognizing rotation.
    Associated,
    /// Discovery validated a replacement at a retained binding (7b Possible).
    Replacement,
}

/// Policies copied from validated configuration.
#[derive(Clone, Copy, Debug)]
pub struct Policy {
    /// First admission of an unrelated file only.
    pub start_at: StartAt,
    /// Uncertain continuity never inherits progress.
    pub mismatch: RecoveryMismatch,
    /// Zero disables age filtering.
    pub ignore_older_than: Duration,
}

/// Validated facts for one exact-locator live-matchable record.
pub struct Existing<'a> {
    /// Authoritative checkpoint record, not a cloned identity table.
    pub record: &'a SnapshotRecord,
    /// Selected framing/identity profile matches the stored profile.
    pub profile_compatible: bool,
    /// Prefix, frontier and size validation all succeeded.
    pub continuous: bool,
}

/// Initial durable anchor for a new logical identity.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Anchor {
    /// Offset zero and clean framing.
    Beginning,
    /// Validated handle EOF and its preceding guard window.
    End,
    /// Register at zero and quarantine in the same transaction.
    Quarantine,
}

/// Complete identity policy result. Mutation and source access are separate.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Decision {
    /// Resume the exact record after continuity validation.
    Resume(FileId),
    /// Keep immutable quarantine unchanged.
    Quarantined(FileId),
    /// Block this file without changing identity or quarantine.
    ProfileBlocked(FileId),
    /// Old unrelated file remains presence evidence; create no durable record.
    Ignored,
    /// Register a fresh ID, optionally replacing the exact-locator Active record.
    Register {
        /// Initial offset policy.
        anchor: Anchor,
        /// Remove this record atomically, never a changed-locator predecessor.
        superseded: Option<FileId>,
        /// Identity reset diagnostic, distinct from an unrelated first admission.
        reset: bool,
    },
}

/// Applies precedence from the Phase 1 matching hierarchy.
/// `age == None` means mtime was unavailable and cannot exclude the candidate.
#[must_use]
pub fn classify(
    policy: Policy,
    existing: Option<Existing<'_>>,
    context: Context,
    age: Option<Duration>,
) -> Decision {
    if let Some(existing) = existing {
        let record = existing.record;
        if let Some(decision) = blocked(record, existing.profile_compatible) {
            return match decision {
                Blocked::Quarantined(id) => Decision::Quarantined(id),
                Blocked::Profile(id) => Decision::ProfileBlocked(id),
            };
        }
        if record.lifecycle_state == LifecycleState::Active {
            if existing.continuous {
                return Decision::Resume(record.file_id);
            }
            return mismatch(policy.mismatch, Some(record.file_id));
        }
        // Finalized history is never eligible for reconnection.
    }
    match context {
        Context::Replacement => Decision::Register {
            anchor: Anchor::Beginning,
            superseded: None,
            reset: false,
        },
        Context::Associated => mismatch(policy.mismatch, None),
        Context::Unrelated => {
            if !policy.ignore_older_than.is_zero()
                && age.is_some_and(|age| age > policy.ignore_older_than)
            {
                return Decision::Ignored;
            }
            Decision::Register {
                anchor: match policy.start_at {
                    StartAt::Beginning => Anchor::Beginning,
                    StartAt::End => Anchor::End,
                },
                superseded: None,
                reset: false,
            }
        }
    }
}

/// Checkpoint status only: no fresh path validation or reader admission is implied.
pub(super) enum Blocked {
    Quarantined(FileId),
    Profile(FileId),
}

pub(super) fn blocked(record: &SnapshotRecord, profile_compatible: bool) -> Option<Blocked> {
    match record.lifecycle_state {
        LifecycleState::Quarantined => Some(Blocked::Quarantined(record.file_id)),
        LifecycleState::Active if !profile_compatible => Some(Blocked::Profile(record.file_id)),
        _ => None,
    }
}

fn mismatch(policy: RecoveryMismatch, superseded: Option<FileId>) -> Decision {
    Decision::Register {
        anchor: match policy {
            RecoveryMismatch::Beginning => Anchor::Beginning,
            RecoveryMismatch::SkipToEnd => Anchor::End,
            RecoveryMismatch::Fail => Anchor::Quarantine,
        },
        superseded,
        reset: true,
    }
}

/// Signed timestamp arithmetic preserves pre-epoch times and clamps future mtimes.
/// SystemTime differences need not be representable as an unsigned Unix timestamp.
#[must_use]
pub fn source_age(now: SystemTime, modified: SystemTime) -> Option<Duration> {
    fn nanos(t: SystemTime) -> Option<i128> {
        match t.duration_since(UNIX_EPOCH) {
            Ok(d) => i128::try_from(d.as_nanos()).ok(),
            Err(e) => i128::try_from(e.duration().as_nanos()).ok()?.checked_neg(),
        }
    }
    let age = nanos(now)?.checked_sub(nanos(modified)?)?.max(0);
    Some(Duration::new(
        u64::try_from(age / 1_000_000_000).ok()?,
        (age % 1_000_000_000) as u32,
    ))
}
