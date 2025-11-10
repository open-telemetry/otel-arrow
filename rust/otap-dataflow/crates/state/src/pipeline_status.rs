// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Observed pipeline status and aggregation logic per core.

use crate::CoreId;
use crate::conditions::{
    Condition, ConditionKind, ConditionReason, ConditionState, ConditionStatus,
};
use crate::phase::PipelinePhase;
use crate::pipeline_rt_status::PipelineRuntimeStatus;
use otap_df_config::health::{HealthPolicy, PhaseKind, Quorum};
use serde::Serialize;
use serde::ser::SerializeStruct;
use std::collections::HashMap;
use std::time::SystemTime;

/// Aggregated, controller-synthesized view for a pipeline across all targeted
/// cores. This is what external APIs will return for `status`.
#[derive(Debug, Clone)]
pub struct PipelineStatus {
    /// Per-core details to aid debugging and aggregation.
    pub(crate) cores: HashMap<CoreId, PipelineRuntimeStatus>,

    health_policy: HealthPolicy,
}

impl PipelineStatus {
    pub(crate) fn new(health_policy: HealthPolicy) -> Self {
        Self {
            cores: HashMap::new(),
            health_policy,
        }
    }

    /// Returns the current per-core status map.
    #[must_use]
    pub fn per_core(&self) -> &HashMap<CoreId, PipelineRuntimeStatus> {
        &self.cores
    }

    #[must_use]
    /// Returns the number of cores currently tracked for this pipeline.
    pub fn total_cores(&self) -> usize {
        self.cores.len()
    }

    #[must_use]
    /// Returns how many cores are presently in the running phase.
    pub fn running_cores(&self) -> usize {
        self.cores
            .values()
            .filter(|c| matches!(c.phase, PipelinePhase::Running))
            .count()
    }

    #[must_use]
    /// Returns the aggregated/synthesized pipeline-level conditions.
    pub fn conditions(&self) -> Vec<Condition> {
        vec![
            self.aggregate_accepted_condition(),
            self.aggregate_ready_condition(),
        ]
    }

    fn aggregate_accepted_condition(&self) -> Condition {
        if self.cores.is_empty() {
            return Condition {
                kind: ConditionKind::Accepted,
                status: ConditionStatus::Unknown,
                reason: Some(ConditionReason::NoPipelineRuntime),
                message: Some("No runtime (core) observed for this pipeline.".to_string()),
                last_transition_time: None,
            };
        }

        let mut latest_false: Option<ConditionState> = None;
        let mut latest_false_time: Option<SystemTime> = None;
        let mut any_unknown: Option<ConditionState> = None;
        let mut latest_true_time: Option<SystemTime> = None;

        for runtime in self.cores.values() {
            let cond = runtime.accepted_condition().clone();
            match cond.status {
                ConditionStatus::True => {
                    latest_true_time = max_time(latest_true_time, cond.last_transition_time);
                }
                ConditionStatus::False => {
                    if latest_false.is_none()
                        || is_time_newer(cond.last_transition_time, latest_false_time)
                    {
                        latest_false_time = cond.last_transition_time;
                        latest_false = Some(cond);
                    }
                }
                ConditionStatus::Unknown => {
                    if any_unknown.is_none()
                        || is_time_newer(
                            cond.last_transition_time,
                            any_unknown.as_ref().and_then(|c| c.last_transition_time),
                        )
                    {
                        any_unknown = Some(cond);
                    }
                }
            }
        }

        if let Some(state) = latest_false {
            return Condition {
                kind: ConditionKind::Accepted,
                status: ConditionStatus::False,
                reason: state.reason.clone().or(Some(ConditionReason::NotAccepted)),
                message: state.message.clone().or_else(|| {
                    Some("One or more cores have not accepted the configuration.".to_string())
                }),
                last_transition_time: state.last_transition_time,
            };
        }

        if let Some(state) = any_unknown {
            return Condition {
                kind: ConditionKind::Accepted,
                status: ConditionStatus::Unknown,
                reason: state
                    .reason
                    .clone()
                    .or_else(|| Some(ConditionReason::unknown("Unknown"))),
                message: state
                    .message
                    .clone()
                    .or_else(|| Some("Acceptance is unknown for one or more cores.".to_string())),
                last_transition_time: state.last_transition_time,
            };
        }

        Condition {
            kind: ConditionKind::Accepted,
            status: ConditionStatus::True,
            reason: Some(ConditionReason::ConfigValid),
            message: Some(
                "Pipeline configuration validated and resources quota is not exceeded.".to_string(),
            ),
            last_transition_time: latest_true_time,
        }
    }

    fn aggregate_ready_condition(&self) -> Condition {
        if self.cores.is_empty() {
            return Condition {
                kind: ConditionKind::Ready,
                status: ConditionStatus::Unknown,
                reason: Some(ConditionReason::NoPipelineRuntime),
                message: Some("No pipeline runtime (core) observed for this pipeline.".to_string()),
                last_transition_time: None,
            };
        }

        let (ready_numer, ready_denom) = self.count_quorum(|c| {
            c.phase.kind() != PhaseKind::Deleted && self.health_policy.is_ready(c.phase.kind())
        });
        let required = required_ready_count(self.health_policy.ready_quorum, ready_denom);
        let readiness_met = ready_denom > 0 && ready_numer >= required;

        let mut latest_true_time: Option<SystemTime> = None;
        let mut latest_false: Option<ConditionState> = None;
        let mut latest_false_time: Option<SystemTime> = None;
        let mut latest_unknown: Option<ConditionState> = None;

        for runtime in self.cores.values() {
            let cond = runtime.ready_condition().clone();
            match cond.status {
                ConditionStatus::True => {
                    latest_true_time = max_time(latest_true_time, cond.last_transition_time);
                }
                ConditionStatus::False => {
                    if latest_false.is_none()
                        || is_time_newer(cond.last_transition_time, latest_false_time)
                    {
                        latest_false_time = cond.last_transition_time;
                        latest_false = Some(cond);
                    }
                }
                ConditionStatus::Unknown => {
                    if latest_unknown.is_none()
                        || is_time_newer(
                            cond.last_transition_time,
                            latest_unknown.as_ref().and_then(|c| c.last_transition_time),
                        )
                    {
                        latest_unknown = Some(cond);
                    }
                }
            }
        }

        if readiness_met {
            return Condition {
                kind: ConditionKind::Ready,
                status: ConditionStatus::True,
                reason: Some(ConditionReason::QuorumMet),
                message: Some("Pipeline is ready to receive and process data.".to_string()),
                last_transition_time: latest_true_time,
            };
        }

        if ready_denom == 0 {
            let last_time = latest_false
                .as_ref()
                .and_then(|c| c.last_transition_time)
                .or_else(|| latest_unknown.as_ref().and_then(|c| c.last_transition_time));
            return Condition {
                kind: ConditionKind::Ready,
                status: ConditionStatus::False,
                reason: Some(ConditionReason::NoActiveCores),
                message: Some("No active cores are available to evaluate readiness.".to_string()),
                last_transition_time: last_time,
            };
        }

        if let Some(state) = latest_false {
            let message = format!(
                "Pipeline is not ready; ready quorum {} not met ({} of {} cores ready).",
                describe_quorum(self.health_policy.ready_quorum, required),
                ready_numer,
                ready_denom
            );
            return Condition {
                kind: ConditionKind::Ready,
                status: ConditionStatus::False,
                reason: Some(ConditionReason::QuorumNotMet),
                message: Some(message),
                last_transition_time: state.last_transition_time,
            };
        }

        if let Some(state) = latest_unknown {
            return Condition {
                kind: ConditionKind::Ready,
                status: ConditionStatus::Unknown,
                reason: state
                    .reason
                    .clone()
                    .or_else(|| Some(ConditionReason::unknown("Unknown"))),
                message: state
                    .message
                    .clone()
                    .or_else(|| Some("Readiness is unknown for one or more cores.".to_string())),
                last_transition_time: state.last_transition_time,
            };
        }

        Condition {
            kind: ConditionKind::Ready,
            status: ConditionStatus::False,
            reason: Some(ConditionReason::QuorumNotMet),
            message: Some("Pipeline is not ready; reason could not be determined.".to_string()),
            last_transition_time: None,
        }
    }

    /// Returns a boolean representing the liveness across cores, governed by the aggregation
    /// policy.
    #[must_use]
    pub fn liveness(&self) -> bool {
        let (numer, denom) = self.count_quorum(|c| self.health_policy.is_live(c.phase.kind()));
        quorum_satisfied(numer, denom, self.health_policy.live_quorum)
    }

    /// Returns a boolean representing the readiness across cores, governed by the aggregation
    /// policy.
    #[must_use]
    pub fn readiness(&self) -> bool {
        let (numer, denom) = self.count_quorum(|c| {
            c.phase.kind() != PhaseKind::Deleted && self.health_policy.is_ready(c.phase.kind())
        });
        denom > 0 && quorum_satisfied(numer, denom, self.health_policy.ready_quorum)
    }

    /// Counts how many cores satisfy the given predicate, returning (numerator, denominator).
    ///
    /// The denominator excludes cores in `Deleted` phase.
    /// The numerator excludes cores in `Deleted` phase and counts only cores satisfying the
    /// predicate. The predicate is usually checking for liveness or readiness.
    fn count_quorum<F>(&self, pred: F) -> (usize, usize)
    where
        F: Fn(&PipelineRuntimeStatus) -> bool,
    {
        let denom = self
            .cores
            .values()
            .filter(|c| c.phase.kind() != PhaseKind::Deleted)
            .count();
        let numer = self
            .cores
            .values()
            .filter(|c| c.phase.kind() != PhaseKind::Deleted)
            .filter(|c| pred(c))
            .count();
        (numer, denom)
    }
}

/// Decide if (numerator/denominator) satisfies a quorum.
fn quorum_satisfied(numer: usize, denom: usize, q: Quorum) -> bool {
    match q {
        Quorum::AtLeast(n) => numer >= n,
        Quorum::Percent(p) => {
            if denom == 0 {
                return false;
            }
            let needed = (denom * usize::from(p)).div_ceil(100);
            numer >= needed
        }
    }
}

fn max_time(a: Option<SystemTime>, b: Option<SystemTime>) -> Option<SystemTime> {
    match (a, b) {
        (Some(x), Some(y)) => Some(if x >= y { x } else { y }),
        (Some(x), None) => Some(x),
        (None, Some(y)) => Some(y),
        (None, None) => None,
    }
}

fn is_time_newer(candidate: Option<SystemTime>, current: Option<SystemTime>) -> bool {
    match (candidate, current) {
        (Some(candidate), Some(current)) => candidate > current,
        (Some(_), None) => true,
        _ => false,
    }
}

fn required_ready_count(quorum: Quorum, denom: usize) -> usize {
    match quorum {
        Quorum::AtLeast(n) => n.min(denom),
        Quorum::Percent(p) => {
            if denom == 0 {
                0
            } else {
                (denom * usize::from(p)).div_ceil(100)
            }
        }
    }
}

fn describe_quorum(quorum: Quorum, required: usize) -> String {
    match quorum {
        Quorum::AtLeast(n) => format!("at least {n} cores ready"),
        Quorum::Percent(p) => format!("{p}% of cores ready (>= {required})"),
    }
}

impl Serialize for PipelineStatus {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut state = serializer.serialize_struct("PipelineStatus", 5)?;
        let conditions = self.conditions();
        state.serialize_field("conditions", &conditions)?;
        state.serialize_field("totalCores", &self.total_cores())?;
        state.serialize_field("runningCores", &self.running_cores())?;
        state.serialize_field("cores", &self.cores)?;
        state.end()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::conditions::{ConditionKind, ConditionReason, ConditionState, ConditionStatus};
    use crate::phase::FailReason;
    use std::collections::HashMap;
    use std::time::{Duration, SystemTime};

    fn runtime(phase: PipelinePhase) -> PipelineRuntimeStatus {
        PipelineRuntimeStatus {
            phase,
            ..PipelineRuntimeStatus::default()
        }
    }

    fn new_status(policy: HealthPolicy) -> PipelineStatus {
        PipelineStatus {
            cores: HashMap::new(),
            health_policy: policy,
        }
    }

    fn runtime_with_conditions(
        phase: PipelinePhase,
        accepted_status: ConditionStatus,
        ready_status: ConditionStatus,
        accepted_reason: Option<ConditionReason>,
        ready_reason: Option<ConditionReason>,
        accepted_time: Option<SystemTime>,
        ready_time: Option<SystemTime>,
    ) -> PipelineRuntimeStatus {
        PipelineRuntimeStatus {
            phase,
            accepted_condition: ConditionState::new(
                accepted_status,
                accepted_reason.clone(),
                None::<String>,
                accepted_time,
            ),
            ready_condition: ConditionState::new(
                ready_status,
                ready_reason,
                None::<String>,
                ready_time,
            ),
            ..Default::default()
        }
    }

    fn ts(secs: u64) -> SystemTime {
        SystemTime::UNIX_EPOCH + Duration::from_secs(secs)
    }

    #[test]
    fn liveness_respects_percent_quorum_and_excludes_deleted() {
        let policy = HealthPolicy {
            live_if: vec![PhaseKind::Running],
            ready_if: vec![PhaseKind::Running],
            live_quorum: Quorum::Percent(60),
            ready_quorum: Quorum::Percent(100),
        };
        let mut status = new_status(policy);
        _ = status.cores.insert(0, runtime(PipelinePhase::Running));
        _ = status.cores.insert(1, runtime(PipelinePhase::Running));
        _ = status
            .cores
            .insert(2, runtime(PipelinePhase::Failed(FailReason::RuntimeError)));
        _ = status.cores.insert(3, runtime(PipelinePhase::Deleted));

        assert!(status.liveness());

        _ = status
            .cores
            .insert(1, runtime(PipelinePhase::Failed(FailReason::RuntimeError)));

        assert!(!status.liveness());
    }

    #[test]
    fn readiness_requires_all_non_deleted_cores_to_be_ready() {
        let policy = HealthPolicy {
            live_if: vec![PhaseKind::Running],
            ready_if: vec![PhaseKind::Running],
            live_quorum: Quorum::AtLeast(1),
            ready_quorum: Quorum::Percent(100),
        };
        let mut status = new_status(policy);
        _ = status.cores.insert(0, runtime(PipelinePhase::Running));
        _ = status.cores.insert(1, runtime(PipelinePhase::Running));

        assert!(status.readiness());

        _ = status.cores.insert(1, runtime(PipelinePhase::Updating));

        assert!(!status.readiness());
    }

    #[test]
    fn aggregated_accept_condition_false_if_any_core_not_accepted() {
        let policy = HealthPolicy::default();
        let mut status = new_status(policy);
        _ = status.cores.insert(
            0,
            runtime_with_conditions(
                PipelinePhase::Running,
                ConditionStatus::True,
                ConditionStatus::True,
                Some(ConditionReason::ConfigValid),
                Some(ConditionReason::Running),
                Some(ts(10)),
                Some(ts(10)),
            ),
        );
        _ = status.cores.insert(
            1,
            runtime_with_conditions(
                PipelinePhase::Pending,
                ConditionStatus::False,
                ConditionStatus::False,
                Some(ConditionReason::Pending),
                Some(ConditionReason::Initializing),
                Some(ts(20)),
                Some(ts(20)),
            ),
        );

        let accepted = status
            .conditions()
            .into_iter()
            .find(|c| matches!(c.kind, ConditionKind::Accepted))
            .expect("missing Accepted condition");

        assert_eq!(accepted.status, ConditionStatus::False);
        assert!(matches!(accepted.reason, Some(ConditionReason::Pending)));
        assert_eq!(accepted.last_transition_time, Some(ts(20)));
    }

    #[test]
    fn aggregated_ready_condition_reports_quorum_not_met() {
        let policy = HealthPolicy {
            live_if: vec![PhaseKind::Running],
            ready_if: vec![PhaseKind::Running],
            live_quorum: Quorum::AtLeast(1),
            ready_quorum: Quorum::Percent(100),
        };
        let mut status = new_status(policy);
        _ = status.cores.insert(
            0,
            runtime_with_conditions(
                PipelinePhase::Running,
                ConditionStatus::True,
                ConditionStatus::True,
                Some(ConditionReason::ConfigValid),
                Some(ConditionReason::Running),
                Some(ts(5)),
                Some(ts(5)),
            ),
        );
        _ = status.cores.insert(
            1,
            runtime_with_conditions(
                PipelinePhase::Failed(FailReason::RuntimeError),
                ConditionStatus::True,
                ConditionStatus::False,
                Some(ConditionReason::ConfigValid),
                Some(ConditionReason::RuntimeError),
                Some(ts(6)),
                Some(ts(12)),
            ),
        );

        let ready = status
            .conditions()
            .into_iter()
            .find(|c| matches!(c.kind, ConditionKind::Ready))
            .expect("missing Ready condition");

        assert_eq!(ready.status, ConditionStatus::False);
        assert!(matches!(ready.reason, Some(ConditionReason::QuorumNotMet)));
        assert!(
            ready
                .message
                .as_deref()
                .is_some_and(|msg| msg.contains("not ready"))
        );
        assert_eq!(ready.last_transition_time, Some(ts(12)));
    }
}
