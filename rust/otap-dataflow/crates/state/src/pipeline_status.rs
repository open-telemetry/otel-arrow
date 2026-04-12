// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Observed pipeline status and aggregation logic per runtime instance.

use crate::conditions::{
    Condition, ConditionKind, ConditionReason, ConditionState, ConditionStatus,
};
use crate::phase::PipelinePhase;
use crate::pipeline_rt_status::PipelineRuntimeStatus;
use otap_df_config::CoreId;
use otap_df_config::health::{HealthPolicy, PhaseKind, Quorum};
use serde::Serialize;
use serde::ser::SerializeStruct;
use std::collections::{HashMap, HashSet};
use std::time::SystemTime;

/// Unique runtime-instance key for a logical pipeline.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct RuntimeInstanceKey {
    /// CPU core hosting the runtime instance.
    pub core_id: CoreId,
    /// Deployment generation for this runtime instance.
    pub deployment_generation: u64,
}

/// Rollout state summary exposed on pipeline status snapshots.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PipelineRolloutState {
    /// Rollout has been accepted but work has not started yet.
    Pending,
    /// Rollout is actively applying changes.
    Running,
    /// Rollout completed successfully and the target generation is serving.
    Succeeded,
    /// Rollout failed before completion.
    Failed,
    /// Automatic rollback is in progress.
    RollingBack,
    /// Rollback could not restore a fully healthy serving set.
    RollbackFailed,
}

/// Lightweight rollout summary embedded into `/status` pipeline payloads.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct PipelineRolloutSummary {
    /// Controller-assigned rollout identifier.
    pub rollout_id: String,
    /// Current rollout lifecycle state.
    pub state: PipelineRolloutState,
    /// Candidate generation being rolled out.
    pub target_generation: u64,
    /// RFC3339 timestamp for rollout creation.
    pub started_at: String,
    /// RFC3339 timestamp for the latest rollout state transition.
    pub updated_at: String,
    /// Human-readable failure or rollback reason when present.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub failure_reason: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct RuntimeInstanceStatusView<'a> {
    core_id: CoreId,
    deployment_generation: u64,
    status: &'a PipelineRuntimeStatus,
}

/// Aggregated, controller-synthesized view for a logical pipeline.
#[derive(Debug, Clone)]
pub struct PipelineStatus {
    /// Per-instance details to aid debugging and overlap-aware generation aggregation.
    pub(crate) instances: HashMap<RuntimeInstanceKey, PipelineRuntimeStatus>,

    /// Serving generation selected per core by the controller during rollout.
    pub(crate) serving_generations: HashMap<CoreId, u64>,

    /// Last committed generation for this logical pipeline.
    pub(crate) active_generation: Option<u64>,

    /// Committed core footprint for the active generation when no rollout-specific
    /// per-core serving override is active.
    pub(crate) active_cores: HashSet<CoreId>,

    /// Optional rollout summary for UI/API consumers.
    pub(crate) rollout: Option<PipelineRolloutSummary>,

    health_policy: HealthPolicy,
}

impl PipelineStatus {
    pub(crate) fn new(health_policy: HealthPolicy) -> Self {
        Self {
            instances: HashMap::new(),
            serving_generations: HashMap::new(),
            active_generation: None,
            active_cores: HashSet::new(),
            rollout: None,
            health_policy,
        }
    }

    /// Returns the current per-instance status map.
    #[must_use]
    pub const fn per_instance(&self) -> &HashMap<RuntimeInstanceKey, PipelineRuntimeStatus> {
        &self.instances
    }

    /// Returns the current serving generation map keyed by core.
    #[must_use]
    pub const fn serving_generations(&self) -> &HashMap<CoreId, u64> {
        &self.serving_generations
    }

    /// Returns the committed active generation, if known.
    #[must_use]
    pub const fn active_generation(&self) -> Option<u64> {
        self.active_generation
    }

    /// Returns the runtime status for a specific `(core, generation)`.
    #[must_use]
    /// Returns the status for one observed runtime instance generation on a core.
    pub fn instance_status(
        &self,
        core_id: CoreId,
        deployment_generation: u64,
    ) -> Option<&PipelineRuntimeStatus> {
        self.instances.get(&RuntimeInstanceKey {
            core_id,
            deployment_generation,
        })
    }

    /// Records the committed active generation for this logical pipeline.
    pub(crate) fn set_active_generation(&mut self, generation: u64) {
        self.active_generation = Some(generation);
    }

    /// Records the committed serving core footprint for the active generation.
    pub(crate) fn set_active_cores<I>(&mut self, core_ids: I)
    where
        I: IntoIterator<Item = CoreId>,
    {
        self.active_cores = core_ids.into_iter().collect();
    }

    /// Pins the serving generation chosen for one logical core.
    pub(crate) fn set_serving_generation(&mut self, core_id: CoreId, generation: u64) {
        _ = self.serving_generations.insert(core_id, generation);
    }

    /// Removes the serving-generation override for one logical core.
    pub(crate) fn clear_serving_generation(&mut self, core_id: CoreId) {
        let _ = self.serving_generations.remove(&core_id);
    }

    /// Stores the rollout summary currently exposed for this pipeline.
    pub(crate) fn set_rollout_summary(&mut self, rollout: PipelineRolloutSummary) {
        self.rollout = Some(rollout);
    }

    /// Clears the rollout summary once no rollout is active anymore.
    pub(crate) fn clear_rollout_summary(&mut self) {
        self.rollout = None;
    }

    /// Compacts retained runtime instances down to the generations currently
    /// selected for status aggregation.
    pub(crate) fn compact_instances_to_selected(&mut self) {
        let retained: HashSet<_> = self.selected_runtime_keys().into_iter().collect();
        self.instances.retain(|key, _| retained.contains(key));
    }

    #[must_use]
    /// Returns the number of currently serving cores for this logical pipeline.
    pub fn total_cores(&self) -> usize {
        self.selected_runtimes().len()
    }

    #[must_use]
    /// Returns how many serving cores are presently in the running phase.
    pub fn running_cores(&self) -> usize {
        self.selected_runtimes()
            .into_iter()
            .filter(|(_, runtime)| matches!(runtime.phase, PipelinePhase::Running))
            .count()
    }

    #[must_use]
    /// Returns true if all observed runtime instances have reached a terminal state.
    pub fn is_terminated(&self) -> bool {
        if self.instances.is_empty() {
            return false;
        }
        self.instances
            .values()
            .all(|runtime| runtime.phase.is_terminal())
    }

    #[must_use]
    /// Returns the aggregated/synthesized pipeline-level conditions.
    pub fn conditions(&self) -> Vec<Condition> {
        vec![
            self.aggregate_accepted_condition(),
            self.aggregate_ready_condition(),
        ]
    }

    /// Aggregates the accepted condition across the selected serving runtimes.
    fn aggregate_accepted_condition(&self) -> Condition {
        let selected = self.selected_runtimes();
        if selected.is_empty() {
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

        for (_, runtime) in selected {
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
                    Some(
                        "One or more serving cores have not accepted the configuration."
                            .to_string(),
                    )
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
                message: state.message.clone().or_else(|| {
                    Some("Acceptance is unknown for one or more serving cores.".to_string())
                }),
                last_transition_time: state.last_transition_time,
            };
        }

        Condition {
            kind: ConditionKind::Accepted,
            status: ConditionStatus::True,
            reason: Some(ConditionReason::ConfigValid),
            message: Some(
                "Serving pipeline configuration validated and resource policy constraints are satisfied."
                    .to_string(),
            ),
            last_transition_time: latest_true_time,
        }
    }

    /// Aggregates the ready condition across the selected serving runtimes.
    fn aggregate_ready_condition(&self) -> Condition {
        let selected = self.selected_runtimes();
        if selected.is_empty() {
            return Condition {
                kind: ConditionKind::Ready,
                status: ConditionStatus::Unknown,
                reason: Some(ConditionReason::NoPipelineRuntime),
                message: Some("No pipeline runtime (core) observed for this pipeline.".to_string()),
                last_transition_time: None,
            };
        }

        let (ready_numer, ready_denom) = self.count_quorum_from(&selected, |runtime| {
            runtime.phase.kind() != PhaseKind::Deleted
                && self.health_policy.is_ready(runtime.phase.kind())
        });
        let required = required_ready_count(self.health_policy.ready_quorum, ready_denom);
        let readiness_met = ready_denom > 0 && ready_numer >= required;

        let mut latest_true_time: Option<SystemTime> = None;
        let mut latest_false: Option<ConditionState> = None;
        let mut latest_false_time: Option<SystemTime> = None;
        let mut latest_unknown: Option<ConditionState> = None;

        for (_, runtime) in selected {
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
                message: Some(
                    "No active serving cores are available to evaluate readiness.".to_string(),
                ),
                last_transition_time: last_time,
            };
        }

        if let Some(state) = latest_false {
            let message = format!(
                "Pipeline is not ready; ready quorum {} not met ({} of {} serving cores ready).",
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
                message: state.message.clone().or_else(|| {
                    Some("Readiness is unknown for one or more serving cores.".to_string())
                }),
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

    /// Returns a boolean representing the liveness across serving cores.
    #[must_use]
    pub fn liveness(&self) -> bool {
        let selected = self.selected_runtimes();
        let (numer, denom) = self.count_quorum_from(&selected, |runtime| {
            self.health_policy.is_live(runtime.phase.kind())
        });
        quorum_satisfied(numer, denom, self.health_policy.live_quorum)
    }

    /// Returns a boolean representing the readiness across serving cores.
    #[must_use]
    pub fn readiness(&self) -> bool {
        let selected = self.selected_runtimes();
        let (numer, denom) = self.count_quorum_from(&selected, |runtime| {
            runtime.phase.kind() != PhaseKind::Deleted
                && self.health_policy.is_ready(runtime.phase.kind())
        });
        denom > 0 && quorum_satisfied(numer, denom, self.health_policy.ready_quorum)
    }

    /// Selects the runtime instances that currently represent this logical pipeline.
    fn selected_runtime_keys(&self) -> Vec<RuntimeInstanceKey> {
        if !self.serving_generations.is_empty() {
            return self
                .serving_generations
                .iter()
                .map(|(core_id, generation)| RuntimeInstanceKey {
                    core_id: *core_id,
                    deployment_generation: *generation,
                })
                .filter(|key| self.instances.contains_key(key))
                .collect();
        }

        if let Some(active_generation) = self.active_generation {
            let selected: Vec<_> = self
                .instances
                .iter()
                .filter(|(key, _)| {
                    key.deployment_generation == active_generation
                        && (self.active_cores.is_empty()
                            || self.active_cores.contains(&key.core_id))
                })
                .map(|(key, _)| *key)
                .collect();
            if !selected.is_empty() {
                return selected;
            }
        }

        let mut per_core: HashMap<CoreId, RuntimeInstanceKey> = HashMap::new();
        for key in self.instances.keys() {
            if !self.active_cores.is_empty() && !self.active_cores.contains(&key.core_id) {
                continue;
            }
            let replace = per_core
                .get(&key.core_id)
                .is_none_or(|existing| key.deployment_generation > existing.deployment_generation);
            if replace {
                _ = per_core.insert(key.core_id, *key);
            }
        }
        per_core.into_values().collect()
    }

    /// Selects the runtime instances that should contribute to aggregated status.
    fn selected_runtimes(&self) -> Vec<(RuntimeInstanceKey, &PipelineRuntimeStatus)> {
        self.selected_runtime_keys()
            .into_iter()
            .filter_map(|key| self.instances.get(&key).map(|runtime| (key, runtime)))
            .collect()
    }

    /// Builds a per-core view of the selected runtime instances.
    fn selected_core_map(&self) -> HashMap<CoreId, PipelineRuntimeStatus> {
        self.selected_runtimes()
            .into_iter()
            .map(|(key, runtime)| (key.core_id, runtime.clone()))
            .collect()
    }

    /// Builds the retained per-instance view exposed for overlap-aware status
    /// debugging.
    fn retained_instance_views(&self) -> Vec<RuntimeInstanceStatusView<'_>> {
        let mut instances = self
            .instances
            .iter()
            .map(|(key, status)| RuntimeInstanceStatusView {
                core_id: key.core_id,
                deployment_generation: key.deployment_generation,
                status,
            })
            .collect::<Vec<_>>();
        instances.sort_by_key(|instance| (instance.core_id, instance.deployment_generation));
        instances
    }

    /// Counts how many selected runtimes satisfy a quorum predicate.
    fn count_quorum_from<F>(
        &self,
        selected: &[(RuntimeInstanceKey, &PipelineRuntimeStatus)],
        pred: F,
    ) -> (usize, usize)
    where
        F: Fn(&PipelineRuntimeStatus) -> bool,
    {
        let denom = selected
            .iter()
            .filter(|(_, runtime)| runtime.phase.kind() != PhaseKind::Deleted)
            .count();
        let numer = selected
            .iter()
            .filter(|(_, runtime)| runtime.phase.kind() != PhaseKind::Deleted)
            .filter(|(_, runtime)| pred(runtime))
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
        let selected_cores = self.selected_core_map();
        let retained_instances = self.retained_instance_views();

        let mut state = serializer.serialize_struct("PipelineStatus", 8)?;
        state.serialize_field("conditions", &self.conditions())?;
        state.serialize_field("totalCores", &self.total_cores())?;
        state.serialize_field("runningCores", &self.running_cores())?;
        state.serialize_field("cores", &selected_cores)?;
        state.serialize_field("instances", &retained_instances)?;
        state.serialize_field("activeGeneration", &self.active_generation)?;
        state.serialize_field("servingGenerations", &self.serving_generations)?;
        state.serialize_field("rollout", &self.rollout)?;
        state.end()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::conditions::{ConditionKind, ConditionReason, ConditionState, ConditionStatus};
    use crate::phase::FailReason;
    use std::time::{Duration, SystemTime};

    fn runtime(phase: PipelinePhase) -> PipelineRuntimeStatus {
        PipelineRuntimeStatus {
            phase,
            ..PipelineRuntimeStatus::default()
        }
    }

    fn insert_runtime(
        status: &mut PipelineStatus,
        core_id: CoreId,
        generation: u64,
        runtime: PipelineRuntimeStatus,
    ) {
        _ = status.instances.insert(
            RuntimeInstanceKey {
                core_id,
                deployment_generation: generation,
            },
            runtime,
        );
    }

    fn new_status(policy: HealthPolicy) -> PipelineStatus {
        PipelineStatus::new(policy)
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
        insert_runtime(&mut status, 0, 0, runtime(PipelinePhase::Running));
        insert_runtime(&mut status, 1, 0, runtime(PipelinePhase::Running));
        insert_runtime(
            &mut status,
            2,
            0,
            runtime(PipelinePhase::Failed(FailReason::RuntimeError)),
        );
        insert_runtime(&mut status, 3, 0, runtime(PipelinePhase::Deleted));
        status.set_active_generation(0);

        assert!(status.liveness());

        insert_runtime(
            &mut status,
            1,
            0,
            runtime(PipelinePhase::Failed(FailReason::RuntimeError)),
        );

        assert!(!status.liveness());
    }

    #[test]
    fn readiness_requires_all_selected_cores_to_be_ready() {
        let policy = HealthPolicy {
            live_if: vec![PhaseKind::Running],
            ready_if: vec![PhaseKind::Running],
            live_quorum: Quorum::AtLeast(1),
            ready_quorum: Quorum::Percent(100),
        };
        let mut status = new_status(policy);
        insert_runtime(&mut status, 0, 0, runtime(PipelinePhase::Running));
        insert_runtime(&mut status, 1, 0, runtime(PipelinePhase::Running));
        status.set_active_generation(0);

        assert!(status.readiness());

        insert_runtime(&mut status, 1, 0, runtime(PipelinePhase::Updating));

        assert!(!status.readiness());
    }

    #[test]
    fn aggregated_accept_condition_false_if_any_serving_core_not_accepted() {
        let policy = HealthPolicy::default();
        let mut status = new_status(policy);
        insert_runtime(
            &mut status,
            0,
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
        insert_runtime(
            &mut status,
            1,
            0,
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
        status.set_active_generation(0);

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
        insert_runtime(
            &mut status,
            0,
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
        insert_runtime(
            &mut status,
            1,
            0,
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
        status.set_active_generation(0);

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

    /// Scenario: observed state contains overlapping generations during a
    /// mixed-generation rollout and the controller marks which generation is
    /// serving on each core.
    /// Guarantees: aggregation selects the serving generation per core so
    /// total/running core counts and readiness reflect the active serving set.
    #[test]
    fn serving_generation_selection_supports_mixed_blue_green_rollout() {
        let mut status = new_status(HealthPolicy::default());
        insert_runtime(&mut status, 0, 0, runtime(PipelinePhase::Stopped));
        insert_runtime(&mut status, 0, 1, runtime(PipelinePhase::Running));
        insert_runtime(&mut status, 1, 0, runtime(PipelinePhase::Running));
        status.set_active_generation(0);
        status.set_serving_generation(0, 1);
        status.set_serving_generation(1, 0);

        assert_eq!(status.total_cores(), 2);
        assert_eq!(status.running_cores(), 2);
        assert!(status.readiness());
    }

    /// Scenario: observed state contains multiple generations for the same
    /// cores while the controller has already pinned the serving generation on
    /// each core.
    /// Guarantees: compaction retains only the serving `(core, generation)`
    /// instances and removes superseded generations from retained state.
    #[test]
    fn compact_instances_retains_only_serving_generations() {
        let mut status = new_status(HealthPolicy::default());
        insert_runtime(&mut status, 0, 0, runtime(PipelinePhase::Stopped));
        insert_runtime(&mut status, 0, 1, runtime(PipelinePhase::Running));
        insert_runtime(&mut status, 1, 0, runtime(PipelinePhase::Running));
        insert_runtime(&mut status, 1, 1, runtime(PipelinePhase::Stopped));
        status.set_active_generation(0);
        status.set_serving_generation(0, 1);
        status.set_serving_generation(1, 0);

        status.compact_instances_to_selected();

        assert_eq!(status.per_instance().len(), 2);
        assert!(status.instance_status(0, 1).is_some());
        assert!(status.instance_status(1, 0).is_some());
        assert!(status.instance_status(0, 0).is_none());
        assert!(status.instance_status(1, 1).is_none());
    }

    /// Scenario: observed state has multiple generations but there is no
    /// mixed-generation serving override and the controller has committed a new
    /// active generation.
    /// Guarantees: compaction retains only the committed active generation.
    #[test]
    fn compact_instances_retains_only_active_generation_when_no_serving_override() {
        let mut status = new_status(HealthPolicy::default());
        insert_runtime(&mut status, 0, 0, runtime(PipelinePhase::Stopped));
        insert_runtime(&mut status, 0, 1, runtime(PipelinePhase::Running));
        insert_runtime(&mut status, 1, 0, runtime(PipelinePhase::Stopped));
        insert_runtime(&mut status, 1, 1, runtime(PipelinePhase::Running));
        status.set_active_generation(1);

        status.compact_instances_to_selected();

        assert_eq!(status.per_instance().len(), 2);
        assert!(status.instance_status(0, 1).is_some());
        assert!(status.instance_status(1, 1).is_some());
        assert!(status.instance_status(0, 0).is_none());
        assert!(status.instance_status(1, 0).is_none());
    }

    /// Scenario: observed state contains only superseded generations relative
    /// to the last committed active generation.
    /// Guarantees: compaction falls back to the highest observed generation per
    /// core so status remains bounded without dropping the last known view.
    #[test]
    fn compact_instances_falls_back_to_latest_generation_per_core() {
        let mut status = new_status(HealthPolicy::default());
        insert_runtime(&mut status, 0, 0, runtime(PipelinePhase::Stopped));
        insert_runtime(&mut status, 0, 2, runtime(PipelinePhase::Stopped));
        insert_runtime(&mut status, 1, 1, runtime(PipelinePhase::Running));
        insert_runtime(&mut status, 1, 3, runtime(PipelinePhase::Stopped));
        status.set_active_generation(9);

        status.compact_instances_to_selected();

        assert_eq!(status.per_instance().len(), 2);
        assert!(status.instance_status(0, 2).is_some());
        assert!(status.instance_status(1, 3).is_some());
        assert!(status.instance_status(0, 0).is_none());
        assert!(status.instance_status(1, 1).is_none());
    }

    /// Scenario: a logical pipeline has been fully shut down and observed state
    /// still contains an older generation alongside the final stopped
    /// generation.
    /// Guarantees: compaction keeps the last stopped generation per core so
    /// `/status` continues to surface the final stopped view instead of
    /// collapsing to an empty runtime set.
    #[test]
    fn compact_instances_preserves_last_stopped_generation_view_after_shutdown() {
        let mut status = new_status(HealthPolicy::default());
        insert_runtime(&mut status, 0, 0, runtime(PipelinePhase::Stopped));
        insert_runtime(&mut status, 0, 1, runtime(PipelinePhase::Stopped));
        status.set_active_generation(1);
        status.set_active_cores([0]);

        status.compact_instances_to_selected();

        assert_eq!(status.per_instance().len(), 1);
        assert_eq!(status.total_cores(), 1);
        assert_eq!(status.running_cores(), 0);
        assert!(matches!(
            status
                .instance_status(0, 1)
                .expect("latest generation should remain")
                .phase(),
            PipelinePhase::Stopped
        ));
        assert!(status.instance_status(0, 0).is_none());
    }

    /// Scenario: a pure resize-down retires one core without changing the
    /// committed generation, so multiple retained instances share the same
    /// active generation across different cores.
    /// Guarantees: aggregated status and compaction respect the committed core
    /// footprint instead of treating every instance on the active generation as
    /// still serving.
    #[test]
    fn active_generation_selection_respects_committed_core_footprint() {
        let mut status = new_status(HealthPolicy::default());
        insert_runtime(&mut status, 0, 0, runtime(PipelinePhase::Running));
        insert_runtime(&mut status, 1, 0, runtime(PipelinePhase::Stopped));
        status.set_active_generation(0);
        status.set_active_cores([0]);

        assert_eq!(status.total_cores(), 1);
        assert_eq!(status.running_cores(), 1);

        status.compact_instances_to_selected();

        assert_eq!(status.per_instance().len(), 1);
        assert!(status.instance_status(0, 0).is_some());
        assert!(status.instance_status(1, 0).is_none());
    }

    /// Scenario: a rolling cutover overlaps the old and new generations on one
    /// core while aggregation must still reflect only the selected serving set.
    /// Guarantees: `/status.instances` preserves both retained generations for
    /// debugging, while aggregated `cores` and core counts continue to use the
    /// selected serving generation per core.
    #[test]
    fn serialization_preserves_overlap_aware_instances_while_aggregating_selected_cores() {
        let mut status = new_status(HealthPolicy::default());
        insert_runtime(&mut status, 0, 0, runtime(PipelinePhase::Stopped));
        insert_runtime(&mut status, 0, 1, runtime(PipelinePhase::Running));
        insert_runtime(&mut status, 1, 0, runtime(PipelinePhase::Running));
        status.set_active_generation(0);
        status.set_serving_generation(0, 1);
        status.set_serving_generation(1, 0);

        let json = serde_json::to_value(&status).expect("pipeline status should serialize");
        let instances = json["instances"]
            .as_array()
            .expect("instances should serialize as an array");

        assert_eq!(json["totalCores"], 2);
        assert_eq!(json["runningCores"], 2);
        assert_eq!(
            json["cores"]
                .as_object()
                .expect("cores should serialize as an object")
                .len(),
            2
        );
        assert_eq!(instances.len(), 3);
        assert_eq!(instances[0]["coreId"], 0);
        assert_eq!(instances[0]["deploymentGeneration"], 0);
        assert_eq!(instances[1]["coreId"], 0);
        assert_eq!(instances[1]["deploymentGeneration"], 1);
        assert_eq!(instances[2]["coreId"], 1);
        assert_eq!(instances[2]["deploymentGeneration"], 0);
    }
}
