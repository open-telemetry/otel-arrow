// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Observed per-core pipeline runtime status and phase transition logic.

use crate::error::Error;
use crate::error::Error::InvalidTransition;
use crate::event::{ErrorEvent as ErrEv, RequestEvent as Req, SuccessEvent as OkEv};
use crate::event::{EventType, ObservedEvent, ObservedEventRingBuffer};
use crate::phase::{DeletionMode, FailReason, PipelinePhase};
use crate::store::ts_to_rfc3339;
use serde::Serialize;
use std::time::SystemTime;

/// The per-core status of a pipeline runtime.
#[derive(Debug, Serialize, Clone)]
pub struct PipelineRuntimeStatus {
    /// Current phase of the pipeline instance.
    pub(crate) phase: PipelinePhase,

    /// Timestamp of the most recent event/heartbeat received for this core.
    #[serde(serialize_with = "ts_to_rfc3339")]
    pub(crate) last_beat: SystemTime,

    /// Latest observed event from this core
    #[serde(skip_serializing_if = "ObservedEventRingBuffer::is_empty", default)]
    pub(crate) recent_events: ObservedEventRingBuffer,

    /// Set when a *graceful* delete has been requested while the pipeline is
    /// actively handling work. The Draining state will transition to Deleting
    /// as soon as we see `Drained` with this flag set.
    pub(crate) delete_pending: bool,
}

/// Outcome of applying an event to the current phase.
#[derive(Debug, Clone, PartialEq, Eq)]
#[allow(variant_size_differences)]
pub enum ApplyOutcome {
    /// The event was ignored or idempotent; no phase or flags changed.
    NoChange,
    /// The phase changed in response to the event.
    Transition {
        from: PipelinePhase,
        to: PipelinePhase,
    },
    ///
    /// Only an internal flag changed (e.g. `delete_pending`) while the phase stayed the same.
    FlagChange { name: &'static str, value: bool },
}

impl Default for PipelineRuntimeStatus {
    fn default() -> Self {
        Self {
            phase: PipelinePhase::Pending,
            last_beat: SystemTime::now(),
            recent_events: ObservedEventRingBuffer::new(5),
            delete_pending: false,
        }
    }
}

impl PipelineRuntimeStatus {
    pub(crate) fn apply_event(&mut self, event: ObservedEvent) -> Result<ApplyOutcome, Error> {
        let outcome = self.apply(event.r#type.clone());
        self.last_beat = event.time;
        self.recent_events.push(event);
        outcome
    }

    /// Apply a single observed event to this pipeline.
    /// Returns what changed (if anything) or an Error::InvalidTransition.
    pub(crate) fn apply(&mut self, event_type: EventType) -> Result<ApplyOutcome, Error> {
        let current_phase = self.phase;
        let outcome = match (current_phase, event_type) {
            // ----- Pending
            (PipelinePhase::Pending, EventType::Success(OkEv::Admitted)) => {
                self.goto(PipelinePhase::Starting)
            }
            (PipelinePhase::Pending, EventType::Error(ErrEv::AdmissionError(_))) => {
                self.goto(PipelinePhase::Failed(FailReason::AdmissionError))
            }
            (PipelinePhase::Pending, EventType::Request(Req::DeleteRequested)) => {
                self.go_to_deleting(DeletionMode::Graceful)
            }
            (PipelinePhase::Pending, EventType::Request(Req::ForceDeleteRequested)) => {
                self.go_to_deleting(DeletionMode::Forced)
            }
            (PipelinePhase::Pending, EventType::Request(Req::StartRequested)) => {
                ApplyOutcome::NoChange
            }

            // ----- Starting
            (PipelinePhase::Starting, EventType::Success(OkEv::Ready)) => {
                self.goto(PipelinePhase::Running)
            }
            (PipelinePhase::Starting, EventType::Error(ErrEv::ConfigRejected(_))) => {
                self.goto(PipelinePhase::Failed(FailReason::ConfigRejected))
            }
            (PipelinePhase::Starting, EventType::Request(Req::DeleteRequested)) => {
                self.go_to_deleting(DeletionMode::Graceful)
            }
            (PipelinePhase::Starting, EventType::Request(Req::ForceDeleteRequested)) => {
                self.go_to_deleting(DeletionMode::Forced)
            }

            // ----- Running
            (PipelinePhase::Running, EventType::Success(OkEv::UpdateAdmitted)) => {
                self.goto(PipelinePhase::Updating)
            }
            (PipelinePhase::Running, EventType::Request(Req::ShutdownRequested)) => {
                self.goto(PipelinePhase::Draining)
            }
            (PipelinePhase::Running, EventType::Error(ErrEv::RuntimeError(_))) => {
                self.goto(PipelinePhase::Failed(FailReason::RuntimeError))
            }
            (PipelinePhase::Running, EventType::Request(Req::DeleteRequested)) => {
                self.delete_pending = true;
                self.goto(PipelinePhase::Draining)
            }
            (PipelinePhase::Running, EventType::Request(Req::ForceDeleteRequested)) => {
                self.go_to_deleting(DeletionMode::Forced)
            }

            // ----- Updating
            (PipelinePhase::Updating, EventType::Success(OkEv::UpdateApplied)) => {
                self.goto(PipelinePhase::Running)
            }
            (PipelinePhase::Updating, EventType::Error(ErrEv::UpdateFailed(_))) => {
                self.goto(PipelinePhase::RollingBack)
            }
            (PipelinePhase::Updating, EventType::Request(Req::DeleteRequested)) => {
                self.delete_pending = true;
                self.goto(PipelinePhase::Draining)
            }
            (PipelinePhase::Updating, EventType::Request(Req::ForceDeleteRequested)) => {
                self.go_to_deleting(DeletionMode::Forced)
            }

            // ----- RollingBack
            (PipelinePhase::RollingBack, EventType::Success(OkEv::RollbackComplete)) => {
                self.goto(PipelinePhase::Running)
            }
            (PipelinePhase::RollingBack, EventType::Error(ErrEv::RollbackFailed(_))) => {
                self.goto(PipelinePhase::Failed(FailReason::RollbackFailed))
            }
            (PipelinePhase::RollingBack, EventType::Request(Req::DeleteRequested)) => {
                self.delete_pending = true;
                self.goto(PipelinePhase::Draining)
            }
            (PipelinePhase::RollingBack, EventType::Request(Req::ForceDeleteRequested)) => {
                self.go_to_deleting(DeletionMode::Forced)
            }

            // ----- Draining
            (PipelinePhase::Draining, EventType::Success(OkEv::Drained)) if self.delete_pending => {
                self.delete_pending = false;
                self.goto(PipelinePhase::Deleting(DeletionMode::Graceful))
            }
            (PipelinePhase::Draining, EventType::Success(OkEv::Drained)) => {
                self.goto(PipelinePhase::Stopped)
            }
            (PipelinePhase::Draining, EventType::Error(ErrEv::DrainError(_))) => {
                self.goto(PipelinePhase::Failed(FailReason::DrainError))
            }
            (PipelinePhase::Draining, EventType::Request(Req::DeleteRequested)) => {
                if !self.delete_pending {
                    self.delete_pending = true;
                    ApplyOutcome::FlagChange {
                        name: "delete_pending",
                        value: true,
                    }
                } else {
                    ApplyOutcome::NoChange
                }
            }
            (PipelinePhase::Draining, EventType::Request(Req::ForceDeleteRequested)) => {
                self.go_to_deleting(DeletionMode::Forced)
            }

            // ----- Stopped
            (PipelinePhase::Stopped, EventType::Request(Req::StartRequested)) => {
                self.goto(PipelinePhase::Pending)
            }
            (PipelinePhase::Stopped, EventType::Request(Req::DeleteRequested)) => {
                self.go_to_deleting(DeletionMode::Graceful)
            }
            (PipelinePhase::Stopped, EventType::Request(Req::ForceDeleteRequested)) => {
                self.go_to_deleting(DeletionMode::Forced)
            }

            // ----- Rejected
            (PipelinePhase::Rejected(_), EventType::Request(Req::StartRequested)) => {
                self.goto(PipelinePhase::Pending)
            }
            (PipelinePhase::Rejected(_), EventType::Request(Req::DeleteRequested)) => {
                self.go_to_deleting(DeletionMode::Graceful)
            }
            (PipelinePhase::Rejected(_), EventType::Request(Req::ForceDeleteRequested)) => {
                self.go_to_deleting(DeletionMode::Forced)
            }

            // ----- Failed (can still delete)
            (PipelinePhase::Failed(_), EventType::Request(Req::StartRequested)) => {
                self.goto(PipelinePhase::Pending)
            }
            (PipelinePhase::Failed(_), EventType::Request(Req::DeleteRequested)) => {
                self.go_to_deleting(DeletionMode::Graceful)
            }
            (PipelinePhase::Failed(_), EventType::Request(Req::ForceDeleteRequested)) => {
                self.go_to_deleting(DeletionMode::Forced)
            }

            // ----- Deleting (idempotent delete requests)
            (PipelinePhase::Deleting(_), EventType::Success(OkEv::Deleted)) => {
                self.goto(PipelinePhase::Deleted)
            }
            (PipelinePhase::Deleting(_), EventType::Error(ErrEv::DeleteError(_))) => {
                self.goto(PipelinePhase::Failed(FailReason::DeleteError))
            }
            (PipelinePhase::Deleting(_), EventType::Request(Req::DeleteRequested))
            | (PipelinePhase::Deleting(_), EventType::Request(Req::ForceDeleteRequested)) => {
                ApplyOutcome::NoChange
            }

            // ----- Deleted (terminal)
            (PipelinePhase::Deleted, _) => {
                /* Ignore everything; already terminal */
                ApplyOutcome::NoChange
            }

            // ----- Idempotent or benign no-ops
            (PipelinePhase::Starting, EventType::Success(OkEv::Admitted))
            | (PipelinePhase::Running, EventType::Success(OkEv::Ready))
            | (PipelinePhase::Updating, EventType::Success(OkEv::UpdateAdmitted))
            | (PipelinePhase::RollingBack, EventType::Error(ErrEv::UpdateFailed(_)))
            | (PipelinePhase::Draining, EventType::Request(Req::ShutdownRequested))
            | (PipelinePhase::Stopped, EventType::Success(OkEv::Drained)) => ApplyOutcome::NoChange,

            // Everything else is considered programmer error (strict mode).
            (phase, ev) => {
                return Err(InvalidTransition {
                    phase,
                    event: Box::new(ev),
                    message: "event not valid for current phase",
                });
            }
        };

        Ok(outcome)
    }

    /// Apply many events in sequence (oldest -> newest).
    pub fn apply_many<I: IntoIterator<Item = EventType>>(
        &mut self,
        events: I,
    ) -> Result<(), Error> {
        for ev in events {
            _ = self.apply(ev)?;
        }
        Ok(())
    }

    // Helper function: record a phase transition.
    fn goto(&mut self, to: PipelinePhase) -> ApplyOutcome {
        if to != self.phase {
            let from = self.phase;
            self.phase = to;
            match (from, to) {
                (PipelinePhase::Failed(from_reason), PipelinePhase::Failed(to_reason)) => {
                    if from_reason != to_reason {
                        ApplyOutcome::FlagChange {
                            name: "fail_reason",
                            value: true,
                        }
                    } else {
                        ApplyOutcome::NoChange
                    }
                }
                (PipelinePhase::Deleting(from_del_mode), PipelinePhase::Deleting(to_del_mpde)) => {
                    if from_del_mode != to_del_mpde {
                        ApplyOutcome::FlagChange {
                            name: "deletion_mode",
                            value: true,
                        }
                    } else {
                        ApplyOutcome::NoChange
                    }
                }
                _ => ApplyOutcome::Transition { from, to }, /* valid transition */
            }
        } else {
            // Idempotency: repeated events in the same phase are no-ops.
            ApplyOutcome::NoChange
        }
    }

    // Centralizes entering Deleting and resets flags.
    fn go_to_deleting(&mut self, mode: DeletionMode) -> ApplyOutcome {
        self.delete_pending = false;
        self.goto(PipelinePhase::Deleting(mode))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::event::{
        ErrorEvent as ErrEv, ErrorSummary, RequestEvent as Req, SuccessEvent as OkEv,
    };

    #[test]
    fn happy_path_to_stopped() {
        let mut p = PipelineRuntimeStatus::default();
        p.apply_many([
            EventType::Success(OkEv::Admitted),
            EventType::Success(OkEv::Ready),
            EventType::Request(Req::ShutdownRequested),
            EventType::Success(OkEv::Drained),
        ])
        .unwrap();
        assert_eq!(p.phase, PipelinePhase::Stopped);
    }

    #[test]
    fn update_then_rollback_then_run() {
        let mut p = PipelineRuntimeStatus::default();
        p.apply_many([
            EventType::Success(OkEv::Admitted),
            EventType::Success(OkEv::Ready),
        ])
        .unwrap();
        p.apply_many([
            EventType::Success(OkEv::UpdateAdmitted),
            EventType::Error(ErrEv::UpdateFailed(ErrorSummary::Pipeline {
                error_kind: "".to_string(),
                message: "".to_string(),
                source: None,
            })),
        ])
        .unwrap();
        assert_eq!(p.phase, PipelinePhase::RollingBack);
        _ = p.apply(EventType::Success(OkEv::RollbackComplete)).unwrap();
        assert_eq!(p.phase, PipelinePhase::Running);
    }

    #[test]
    fn graceful_delete_from_running() {
        let mut p = PipelineRuntimeStatus::default();
        p.apply_many([
            EventType::Success(OkEv::Admitted),
            EventType::Success(OkEv::Ready),
        ])
        .unwrap();
        _ = p.apply(EventType::Request(Req::DeleteRequested)).unwrap();
        assert_eq!(p.phase, PipelinePhase::Draining);
        _ = p.apply(EventType::Success(OkEv::Drained)).unwrap();
        assert_eq!(p.phase, PipelinePhase::Deleting(DeletionMode::Graceful));
    }

    #[test]
    fn force_delete_from_updating() {
        let mut p = PipelineRuntimeStatus::default();
        p.apply_many([
            EventType::Success(OkEv::Admitted),
            EventType::Success(OkEv::Ready),
            EventType::Success(OkEv::UpdateAdmitted),
        ])
        .unwrap();
        _ = p
            .apply(EventType::Request(Req::ForceDeleteRequested))
            .unwrap();
        assert_eq!(p.phase, PipelinePhase::Deleting(DeletionMode::Forced));
    }

    #[test]
    fn deleting_to_deleted_and_error() {
        let mut p = PipelineRuntimeStatus {
            phase: PipelinePhase::Deleting(DeletionMode::Graceful),
            last_beat: SystemTime::now(),
            recent_events: ObservedEventRingBuffer::new(5),
            delete_pending: false,
        };
        _ = p.apply(EventType::Success(OkEv::Deleted)).unwrap();
        assert_eq!(p.phase, PipelinePhase::Deleted);

        let mut p2 = PipelineRuntimeStatus {
            phase: PipelinePhase::Deleting(DeletionMode::Forced),
            last_beat: SystemTime::now(),
            recent_events: ObservedEventRingBuffer::new(5),
            delete_pending: false,
        };
        _ = p2
            .apply(EventType::Error(ErrEv::DeleteError(
                ErrorSummary::Pipeline {
                    error_kind: "".to_string(),
                    message: "".to_string(),
                    source: None,
                },
            )))
            .unwrap();
        assert_eq!(p2.phase, PipelinePhase::Failed(FailReason::DeleteError));
    }

    #[test]
    fn invalid_transition_is_error() {
        let mut p = PipelineRuntimeStatus::default(); // Pending
        let err = p.apply(EventType::Success(OkEv::Ready)).unwrap_err();
        assert!(matches!(err, InvalidTransition { .. }));
        assert_eq!(p.phase, PipelinePhase::Pending); // unchanged
    }
}
