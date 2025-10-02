// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Set of structs defining an event-driven observed state store.

use crate::config::Config;
use crate::error::Error;
use crate::reporter::ObservedEventReporter;
use crate::store::ConditionStatus::{False, True, Unknown};
use crate::{CoreId, DeployedPipelineKey, PipelineKey};
use otap_df_config::{PipelineGroupId, PipelineId};
use serde::{Serialize, Serializer};
use std::collections::{HashMap, VecDeque};
use std::fmt;
use std::fmt::Display;
use std::str::FromStr;
use std::sync::{Arc, Mutex};
use std::time::SystemTime;
use tokio_util::sync::CancellationToken;

/// High-level lifecycle of a pipeline as seen by the controller.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize)]
pub enum PipelinePhase {
    /// The pipeline is pending and has not started yet.
    Pending,
    /// The pipeline is currently running and actively processing telemetry data.
    Running,
    /// A graceful stop has been requested. Ingress is quiescing and in-flight
    /// data is draining, possibly with a deadline.
    Draining,
    /// The pipeline has been stopped.
    Stopped,
    /// Entered a terminal error state (e.g. unrecoverable apply error). The
    /// controller may attempt retries based on policy, but phase reflects the
    /// current failure.
    Failed,
    /// The controller cannot currently determine the state (e.g. missing
    /// heartbeats past the freshness window).
    Unknown,
}

impl Display for PipelinePhase {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let label = match self {
            PipelinePhase::Pending => "Pending",
            PipelinePhase::Running => "Running",
            PipelinePhase::Draining => "Draining",
            PipelinePhase::Stopped => "Stopped",
            PipelinePhase::Failed => "Failed",
            PipelinePhase::Unknown => "Unknown",
        };
        write!(f, "{label}")
    }
}

/// Types of events that can be observed from a pipeline engine instance.
#[derive(Debug)]
pub enum ObservedEvent {
    /// A pipeline phase change event.
    PipelinePhase {
        /// Unique key identifying the pipeline instance.
        key: PipelineKey,
        /// Core ID reporting the phase change.
        core_id: CoreId,
        /// New phase of the pipeline instance.
        phase: PipelinePhase,
    },

    /// A periodic heartbeat event indicating the pipeline is alive.
    Heartbeat {
        /// Unique key identifying the pipeline instance.
        key: PipelineKey,
        /// Core ID sending the heartbeat.
        core_id: CoreId,
    },

    /// A condition update emitted by the engine.
    ///
    /// The controller should **aggregate** these into pipeline-level
    /// `PipelineStatus.conditions`. For example, set `Healthy=True` only if
    /// *all* cores report `Healthy=True`; if any core is `False` then aggregate
    /// to `False`, and if any core is `Unknown` (and none are `False`), aggregate
    /// to `Unknown`.
    Condition {
        /// Unique key identifying the pipeline instance.
        key: PipelineKey,
        /// Core ID that observed this condition.
        core_id: CoreId,
        /// Condition emitted by the engine.
        condition: Condition,
    },
}

/// Status of a condition (tri-state).
///
/// Mirrors Kubernetes’ `ConditionStatus` (`True|False|Unknown`) to clearly
/// separate the **facet** (see [`ConditionType`]) from its current **truth value**.
///
/// See K8S references:
/// - API conventions – *Conditions*: https://github.com/kubernetes/community/blob/master/contributors/devel/sig-architecture/api-conventions.md#conditions
/// - Go type `ConditionStatus`: https://pkg.go.dev/k8s.io/apimachinery/pkg/apis/meta/v1#ConditionStatus
#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "PascalCase")]
pub enum ConditionStatus {
    /// Predicate holds (e.g. Healthy=True)
    True,
    /// Predicate explicitly does not hold (e.g. Healthy=False)
    False,
    /// Controller cannot determine; gate conservatively
    Unknown,
}

/// Condition *type* is an **open set**: well-known variants + plugin/vendor customs.
///
/// See K8S reference: https://github.com/kubernetes/community/blob/master/contributors/devel/sig-architecture/api-conventions.md#conditions
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ConditionType {
    /// SLO health of the pipeline (latency/errors/backpressure within limits).
    /// May be `False` while `Ready=True` (serving but degraded).
    Healthy,

    /// Admission/readiness: "can accept/switch traffic now".
    /// Recommended to gate **traffic flips** on `Ready=True`, and gate **rollout
    /// waves** on `Healthy=True`. `Ready` does **not** imply `Healthy` here.
    /// Related K8s concept: Pod readiness.
    /// Docs: https://kubernetes.io/docs/concepts/workloads/pods/pod-lifecycle/#pod-conditions
    Ready,

    /// Ingress backpressure is high (e.g. queue depth/drops exceeded thresholds).
    /// Often implies `Healthy=False`, but is kept separate for explainability.
    BackpressureHigh,

    /// Heartbeats are stale/missing past freshness window; typically forces
    /// `Ready=False` and may set `Healthy=Unknown` until state is clear.
    HeartbeatMissing,

    /// A rollout/canary/wave is currently in progress (transitional state).
    /// Helps UIs and policies distinguish rollout work from steady state.
    RolloutInProgress,

    /// Spec/plan validation/apply failed on one or more cores. Keep `status=True`
    /// until resolved; set a concise `reason` for automation.
    ConfigRejected,

    /// The pipeline encountered a panic during startup or runtime execution.
    /// Panic marks code-level bugs or safety violations.
    /// (fix config vs. **rollback code**)
    Panic,

    /// The pipeline failed to start properly due to an error
    /// encountered during its startup sequence (not a panic).
    /// StartError captures deterministic startup/config issues (bad credentials, exporter failures,
    /// quota exhaustion).
    /// (**fix config** vs. rollback code)
    StartError,

    /// Catch-all for plugin/vendor-specific facets; preserves original identifier.
    Custom(String),
}

/// A fine-grained, typed facet of status used for health/rollout gating.
///
/// **Rationale**
/// - Keep high-level `PipelinePhase` **coarse** (Pending/Running/...).
/// - Encode orthogonal axes here (health, readiness, backpressure, rollout, config errors).
/// - Use **two gates**:
///   - Advance **rollout waves** when `Healthy=True`
///   - Flip/switch **traffic** when `Ready=True`
///
/// **Ready vs Healthy: tiny rule-of-thumb table**
///
/// | Scenario                         | Ready | Healthy | Meaning                                                  |
/// |----------------------------------|:-----:|:-------:|----------------------------------------------------------|
/// | Serving, backpressure high       |  T   |   F    | Admit traffic but degraded; throttle rollouts/alerts fire|
/// | Rollout/drain pause              |  F   |   T    | Healthy but intentionally not serving (maintenance)      |
/// | All good                         |  T   |   T    | Green; safe to serve and to advance rollouts             |
/// | Sick and not serving             |  F   |   F    | Draining/crashed/overloaded                              |
///
/// References:
/// - API conventions:
///   https://github.com/kubernetes/community/blob/master/contributors/devel/sig-architecture/api-conventions.md#conditions
/// - Pod conditions example:
///   https://kubernetes.io/docs/concepts/workloads/pods/pod-lifecycle/#pod-conditions
const RECENT_EVENTS_CAPACITY: usize = 20;

/// Aggregated condition state used for pipeline-level status surfaces.
#[derive(Debug, Clone, Serialize)]
pub struct Condition {
    /// Condition category (extensible identifier).
    pub r#type: ConditionType,
    /// Tri-state truth value (`True|False|Unknown`).
    pub status: ConditionStatus,
    /// Short, stable machine-readable reason code (e.g. `AllCoresHealthy`).
    pub reason: String,
    /// Human-readable context (concise; details go to logs/metrics).
    pub message: String,
    /// Timestamp of the last `(type,status)` transition. Do **not** change when only
    /// `reason`/`message` are updated (keeps transition history meaningful).
    #[serde(serialize_with = "ts_to_rfc3339")]
    pub last_transition_time: SystemTime,
    /// Aggregate counts of per-core condition values, if computed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub counts: Option<ConditionCounts>,
    /// Additional structured context such as node-level errors.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<ConditionDetails>,
}

/// Aggregated counts of condition statuses across cores.
#[derive(Debug, Clone, Serialize, Default)]
pub struct ConditionCounts {
    /// Number of cores reporting the condition as `True`.
    pub true_count: u32,
    /// Number of cores reporting the condition as `False`.
    pub false_count: u32,
    /// Number of cores reporting the condition as `Unknown`.
    pub unknown_count: u32,
}

/// Structured diagnostics associated with a condition.
#[derive(Debug, Clone, Serialize, Default)]
pub struct ConditionDetails {
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    /// Node-level errors contributing to this condition.
    pub node_errors: Vec<NodeErrorSummary>,
}

/// Summary of a node-level error contributing to a pipeline condition.
#[derive(Debug, Clone, Serialize)]
pub struct NodeErrorSummary {
    /// Identifier of the node emitting the error.
    pub node: String,
    /// Node classification (e.g. `exporter`, `processor`).
    pub node_kind: String,
    /// High-level error category (connect/configuration/transport/etc.).
    pub error_kind: String,
    /// User-facing error message.
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    /// Flattened source chain for deeper debugging, if available.
    pub source: Option<String>,
}

/// The per-core view of a pipeline instance, as reported by an engine.
#[derive(Debug, Serialize, Clone)]
pub struct CoreStatus {
    /// Current phase of the pipeline instance.
    pub phase: PipelinePhase,

    /// Timestamp of the most recent event/heartbeat received for this core.
    #[serde(serialize_with = "ts_to_rfc3339")]
    last_beat: SystemTime,

    /// Latest observed conditions from this core, keyed by `ConditionType`.
    /// The controller should aggregate these into `PipelineStatus.conditions`.
    /// ToDo Should we skip this field for the serialization `#[serde(skip)]` ?
    #[serde(skip_serializing_if = "HashMap::is_empty", default)]
    pub conditions: HashMap<ConditionType, Condition>,
}

/// Aggregated, controller-synthesized view for a pipeline across all targeted
/// cores. This is what external APIs will return for `status`.
#[derive(Debug, Serialize, Clone)]
pub struct PipelineStatus {
    /// Coarse phase synthesized from all per-core phases.
    phase: PipelinePhase,
    /// Timestamp of the last phase transition.
    #[serde(serialize_with = "ts_to_rfc3339")]
    phase_since: SystemTime,
    /// Typed facets (health, rollout, backpressure, ...).
    /// Treat logically as a map keyed by `type`.
    conditions: Vec<Condition>,
    /// Per-core details to aid debugging and aggregation.
    per_core: HashMap<CoreId, CoreStatus>,
    /// Recently observed events to aid troubleshooting and timelines.
    #[serde(skip_serializing_if = "VecDeque::is_empty", default)]
    recent_events: VecDeque<PipelineEvent>,
}

/// Recorded event for pipeline timeline debugging.
#[derive(Debug, Serialize, Clone)]
pub struct PipelineEvent {
    #[serde(serialize_with = "ts_to_rfc3339")]
    /// Timestamp when the event was observed.
    timestamp: SystemTime,
    /// Core that reported the event.
    core_id: CoreId,
    /// Event payload describing the observation.
    kind: PipelineEventKind,
}

/// Event kind and payload for pipeline diagnostics.
#[derive(Debug, Serialize, Clone)]
#[serde(tag = "event", content = "data")]
pub enum PipelineEventKind {
    /// Phase transition observed for the pipeline.
    Phase {
        /// Phase the pipeline transitioned into.
        phase: PipelinePhase,
    },
    /// Condition update aggregated across cores.
    Condition {
        /// Condition type being reported.
        r#type: ConditionType,
        /// Aggregated condition status.
        status: ConditionStatus,
        /// Machine-readable explanation for the update.
        reason: String,
        /// Human-readable context for operators.
        message: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        /// Counts of per-core statuses contributing to the aggregate.
        counts: Option<ConditionCounts>,
        #[serde(skip_serializing_if = "Option::is_none")]
        /// Structured diagnostic details, if any.
        details: Option<ConditionDetails>,
    },
    /// Heartbeat emitted by the pipeline runtime.
    Heartbeat,
}

impl PipelineEvent {
    fn phase(timestamp: SystemTime, core_id: CoreId, phase: PipelinePhase) -> Self {
        Self {
            timestamp,
            core_id,
            kind: PipelineEventKind::Phase { phase },
        }
    }

    fn condition(
        timestamp: SystemTime,
        core_id: CoreId,
        r#type: ConditionType,
        status: ConditionStatus,
        reason: String,
        message: String,
        counts: Option<ConditionCounts>,
        details: Option<ConditionDetails>,
    ) -> Self {
        Self {
            timestamp,
            core_id,
            kind: PipelineEventKind::Condition {
                r#type,
                status,
                reason,
                message,
                counts,
                details,
            },
        }
    }

    /// Returns the timestamp of the recorded event.
    pub fn timestamp(&self) -> SystemTime {
        self.timestamp
    }

    /// Returns the event payload for inspection.
    pub fn kind(&self) -> &PipelineEventKind {
        &self.kind
    }
}

/// Event-driven observed state store representing what we know about the state of the
/// pipelines (DAG executors) controlled by the controller.
///
/// This store is cloneable and thread-safe, allowing multiple threads to record events
/// concurrently or read the current state.
#[derive(Debug, Clone, Serialize)]
pub struct ObservedStateStore {
    #[serde(skip)]
    config: Config,

    #[serde(skip)]
    sender: flume::Sender<ObservedEvent>,

    #[serde(skip)]
    receiver: flume::Receiver<ObservedEvent>,

    pipelines: Arc<Mutex<HashMap<PipelineKey, PipelineStatus>>>,
}

/// A handle to the observed state, suitable for serialization and external consumption.
#[derive(Debug, Clone, Serialize)]
pub struct ObservedStateHandle {
    pipelines: Arc<Mutex<HashMap<PipelineKey, PipelineStatus>>>,
}

impl ObservedStateHandle {
    /// Returns a cloned snapshot of the current pipeline statuses.
    pub fn snapshot(&self) -> HashMap<PipelineKey, PipelineStatus> {
        match self.pipelines.lock() {
            Ok(guard) => guard.clone(),
            Err(poisoned) => {
                log::warn!(
                    "ObservedStateHandle mutex was poisoned; returning possibly stale snapshot"
                );
                poisoned.into_inner().clone()
            }
        }
    }
}

fn ts_to_rfc3339<S>(t: &SystemTime, s: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let dt: chrono::DateTime<chrono::Utc> = (*t).into();
    s.serialize_str(&dt.to_rfc3339())
}

impl ConditionType {
    /// Returns the string representation of the condition type.
    #[inline]
    #[must_use]
    pub fn as_str(&self) -> &str {
        match self {
            ConditionType::Healthy => "Healthy",
            ConditionType::Ready => "Ready",
            ConditionType::BackpressureHigh => "BackpressureHigh",
            ConditionType::HeartbeatMissing => "HeartbeatMissing",
            ConditionType::RolloutInProgress => "RolloutInProgress",
            ConditionType::ConfigRejected => "ConfigRejected",
            ConditionType::Panic => "Panic",
            ConditionType::StartError => "StartError",
            ConditionType::Custom(s) => s.as_str(),
        }
    }
}

impl Display for ConditionType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for ConditionType {
    type Err = core::convert::Infallible;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "Healthy" => ConditionType::Healthy,
            "Ready" => ConditionType::Ready,
            "BackpressureHigh" => ConditionType::BackpressureHigh,
            "HeartbeatMissing" => ConditionType::HeartbeatMissing,
            "RolloutInProgress" => ConditionType::RolloutInProgress,
            "ConfigRejected" => ConditionType::ConfigRejected,
            "Panic" => ConditionType::Panic,
            "StartError" => ConditionType::StartError,
            other => ConditionType::Custom(other.to_string()),
        })
    }
}

impl Serialize for ConditionType {
    fn serialize<S: Serializer>(&self, ser: S) -> Result<S::Ok, S::Error> {
        ser.serialize_str(self.as_str())
    }
}

impl PipelineStatus {
    fn new(now: SystemTime) -> Self {
        Self {
            phase: PipelinePhase::Pending,
            phase_since: now,
            conditions: Vec::new(),
            per_core: HashMap::new(),
            recent_events: VecDeque::new(),
        }
    }

    fn push_event(&mut self, event: PipelineEvent) {
        if self.recent_events.len() >= RECENT_EVENTS_CAPACITY {
            let _ = self.recent_events.pop_front();
        }
        self.recent_events.push_back(event);
    }

    /// Upsert a condition with correct transition semantics.
    ///
    /// # Rationale
    /// - **Uniqueness by `type`**: there should be **at most one** entry per `ConditionType`.
    ///   This method enforces "last-writer-wins" for that key.
    /// - **Transition time discipline**: we update `last_transition_time` **only when the
    ///   `(type, status)` pair changes**. This keeps transition timestamps meaningful for
    ///   alerting, SLO burn analysis, and UIs (you don't want clock churn when only
    ///   `reason`/`message` text changes).
    /// - **Idempotence**: calling this repeatedly with the same `(type, status)` and the
    ///   same (or different) reason/message will not alter the transition time, which
    ///   makes it safe to emit on every reconcile loop or heartbeat.
    /// - **Unknown is first-class**: passing `ConditionStatus::Unknown` preserves the
    ///   invariant that lack of signal is not treated as success; use it to gate
    ///   conservatively.
    ///
    /// # Complexity
    /// - **O(n)** scan over `conditions` (a `Vec`). This is fine for small N. If the set
    ///   grows, we will consider a `HashMap<ConditionType, Condition>` internally and derive order
    ///   from keys (for stability).
    pub fn upsert_condition(
        &mut self,
        r#type: ConditionType,
        status: ConditionStatus,
        reason: impl Into<String>,
        message: impl Into<String>,
        now: SystemTime,
        counts: Option<ConditionCounts>,
        details: Option<ConditionDetails>,
    ) {
        // Try to find an existing condition with the same type.
        if let Some(c) = self.conditions.iter_mut().find(|c| c.r#type == r#type) {
            // Only bump the transition timestamp if the truth value actually changed.
            if c.status != status {
                c.status = status;
                c.last_transition_time = now;
            }
            // Always refresh reason/message so operators see the latest context,
            // without polluting the transition history.
            c.reason = reason.into();
            c.message = message.into();
            match counts {
                Some(counts) => c.counts = Some(counts),
                None => c.counts = None,
            }
            match details {
                Some(details) if !details.node_errors.is_empty() => c.details = Some(details),
                Some(_) => c.details = None,
                None => {}
            }
            return;
        }

        // Not present: create a new condition entry initialized with the current time.
        self.conditions.push(Condition {
            r#type,
            status,
            reason: reason.into(),
            message: message.into(),
            last_transition_time: now,
            counts,
            details: details.and_then(|d| {
                if d.node_errors.is_empty() {
                    None
                } else {
                    Some(d)
                }
            }),
        });
    }

    /// Returns a reference to a condition by its type, if it exists.
    #[must_use]
    pub fn get_condition(&self, t: &ConditionType) -> Option<&Condition> {
        self.conditions.iter().find(|c| &c.r#type == t)
    }

    /// Returns an iterator of all conditions.
    pub fn conditions(&self) -> impl Iterator<Item = &Condition> {
        self.conditions.iter()
    }

    /// Returns the current aggregated phase of the pipeline.
    pub fn phase(&self) -> PipelinePhase {
        self.phase
    }

    /// Returns the timestamp corresponding to the last phase transition.
    pub fn phase_since(&self) -> SystemTime {
        self.phase_since
    }

    /// Returns the current per-core status map.
    pub fn per_core(&self) -> &HashMap<CoreId, CoreStatus> {
        &self.per_core
    }

    /// Returns the recent event ring buffer for timeline visualisations.
    pub fn recent_events(&self) -> &VecDeque<PipelineEvent> {
        &self.recent_events
    }
}

impl Default for ObservedStateStore {
    fn default() -> Self {
        Self::new(Config::default())
    }
}

impl ObservedStateStore {
    /// Creates a new `ObservedStateStore` with the given configuration.
    #[must_use]
    pub fn new(config: Config) -> Self {
        let (sender, receiver) = flume::bounded::<ObservedEvent>(config.reporting_channel_size);

        Self {
            config,
            sender,
            receiver,
            pipelines: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Returns a reporter that can be used to send observed events to this store.
    #[must_use]
    pub fn reporter(&self) -> ObservedEventReporter {
        ObservedEventReporter::new(self.config.reporting_timeout, self.sender.clone())
    }

    /// Returns a handle that can be used to read the current observed state.
    #[must_use]
    pub fn handle(&self) -> ObservedStateHandle {
        ObservedStateHandle {
            pipelines: self.pipelines.clone(),
        }
    }

    /// Reports a new observed event in the store.
    fn report(&self, observed_event: ObservedEvent) {
        // Minimize lock duration by computing timestamps outside the critical section
        let now = SystemTime::now();

        match observed_event {
            ObservedEvent::PipelinePhase {
                key,
                core_id,
                phase,
            } => {
                // Prepare the core status update outside the lock
                let new_core_status = CoreStatus {
                    phase,
                    last_beat: now,
                    conditions: Default::default(),
                };

                // Single lock acquisition for the entire update
                let mut pipelines = self.pipelines.lock().unwrap_or_else(|poisoned| {
                    // Rational: We prefer to prioritize availability of the data plane over the
                    // observed state store's consistency.
                    log::warn!("ObservedStateStore mutex was poisoned; continuing with possibly inconsistent state");
                    poisoned.into_inner()
                });

                let pipeline_status = pipelines
                    .entry(key)
                    .or_insert_with(|| PipelineStatus::new(now));

                // Check if phase actually changed before expensive aggregation
                let phase_changed = pipeline_status
                    .per_core
                    .get(&core_id)
                    .is_none_or(|cs| cs.phase != phase);

                // Update the core status
                let _ = pipeline_status.per_core.insert(core_id, new_core_status);

                // Only recalculate aggregate phase if the core's phase actually changed
                if phase_changed {
                    pipeline_status.phase = aggregate_pipeline_phase(
                        pipeline_status.per_core.values().map(|c| c.phase),
                    );
                    pipeline_status.phase_since = now;
                }

                pipeline_status.push_event(PipelineEvent::phase(now, core_id, phase));
            }
            ObservedEvent::Heartbeat { key, core_id } => {
                // Gracefully handle poisoned mutex and proceed
                let mut pipelines = self.pipelines.lock().unwrap_or_else(|poisoned| {
                    // Rational: We prefer to prioritize availability of the data plane over the
                    // observed state store's consistency. Anyway, future heartbeats will correct
                    // any inconsistency.
                    log::warn!("ObservedStateStore mutex was poisoned; continuing with possibly inconsistent state");
                    poisoned.into_inner()
                });

                let pipeline_status = pipelines
                    .entry(key)
                    .or_insert_with(|| PipelineStatus::new(now));

                // For heartbeats, we only update the timestamp, not the phase
                if let Some(core_status) = pipeline_status.per_core.get_mut(&core_id) {
                    core_status.last_beat = now;
                } else {
                    let _ = pipeline_status.per_core.insert(
                        core_id,
                        CoreStatus {
                            phase: PipelinePhase::Pending,
                            last_beat: now,
                            conditions: Default::default(),
                        },
                    );
                }
            }
            ObservedEvent::Condition {
                key,
                core_id,
                condition,
            } => {
                let mut pipelines = self.pipelines.lock().unwrap_or_else(|poisoned| {
                    // Rational: We prefer to prioritize availability of the data plane over the
                    // observed state store's consistency. Anyway, future heartbeats will correct
                    // any inconsistency.
                    log::warn!("ObservedStateStore mutex was poisoned; continuing with possibly inconsistent state");
                    poisoned.into_inner()
                });

                let ps = pipelines
                    .entry(key)
                    .or_insert_with(|| PipelineStatus::new(now));

                // Upsert the core record and its condition snapshot
                let cs = ps.per_core.entry(core_id).or_insert(CoreStatus {
                    phase: PipelinePhase::Pending,
                    last_beat: condition.last_transition_time,
                    conditions: Default::default(),
                });
                cs.last_beat = condition.last_transition_time;
                let ctype = condition.r#type.clone();
                let ts = condition.last_transition_time;
                _ = cs.conditions.insert(ctype.clone(), condition);

                // Aggregate this condition type across cores
                let agg = aggregate_condition_status(
                    ps.per_core
                        .values()
                        .filter_map(|c| c.conditions.get(&ctype))
                        .map(|c| c.status),
                );

                // Compose a concise summary and upsert the pipeline-level condition
                let (mut t, mut f, mut u) = (0, 0, 0);
                let mut sample_condition: Option<Condition> = None;
                for cond in ps
                    .per_core
                    .values()
                    .filter_map(|c| c.conditions.get(&ctype))
                {
                    match cond.status {
                        True => t += 1,
                        False => f += 1,
                        Unknown => u += 1,
                    }

                    let needs_update = sample_condition
                        .as_ref()
                        .is_none_or(|s| cond.last_transition_time >= s.last_transition_time);
                    if needs_update {
                        sample_condition = Some(cond.clone());
                    }
                }
                let total = t + f + u;
                let agg_reason = match agg {
                    True => format!("AllCores{}True", ctype.as_str()),
                    False if f == total && total > 0 => {
                        format!("AllCores{}False", ctype.as_str())
                    }
                    False => format!("SomeCores{}False", ctype.as_str()),
                    Unknown if u == total && total > 0 => {
                        format!("AllCores{}Unknown", ctype.as_str())
                    }
                    Unknown => format!("SomeCores{}Unknown", ctype.as_str()),
                };
                let agg_status = match agg {
                    True => "True",
                    False => "False",
                    Unknown => "Unknown",
                };
                let agg_message = if let Some(ref sample) = sample_condition {
                    format!(
                        "{} status {} across {} cores (true={}, false={}, unknown={}). Last reason: {}. Last message: {}",
                        ctype.as_str(),
                        agg_status,
                        total,
                        t,
                        f,
                        u,
                        sample.reason.clone(),
                        sample.message.clone()
                    )
                } else {
                    format!(
                        "{} status {} across {} cores (true={}, false={}, unknown={})",
                        ctype.as_str(),
                        agg_status,
                        total,
                        t,
                        f,
                        u
                    )
                };

                let counts = ConditionCounts {
                    true_count: t as u32,
                    false_count: f as u32,
                    unknown_count: u as u32,
                };

                let details = sample_condition.as_ref().and_then(|c| c.details.clone());

                ps.upsert_condition(
                    ctype.clone(),
                    agg,
                    agg_reason.clone(),
                    agg_message.clone(),
                    ts,
                    Some(counts.clone()),
                    details.clone(),
                );

                ps.push_event(PipelineEvent::condition(
                    ts,
                    core_id,
                    ctype,
                    agg,
                    agg_reason,
                    agg_message,
                    Some(counts),
                    details,
                ));
            }
        }
    }

    /// Runs the collection loop, receiving observed events and updating the observed store.
    /// This function runs indefinitely until the channel is closed or the cancellation token is
    /// triggered.
    pub async fn run(self, cancel: CancellationToken) -> Result<(), Error> {
        tokio::select! {
            _ = async {
                // Continuously receive events and report them
                // Exit the loop if the channel is closed
                while let Ok(event) = self.receiver.recv_async().await {
                    self.report(event);
                }
            } => { /* Channel closed, exit gracefully */ }
            _ = cancel.cancelled() => { /* Cancellation requested, exit gracefully */ }
        }
        Ok(())
    }
}

impl ObservedStateHandle {
    /// Retrieves the current status of a pipeline by its key.
    /// Returns a snapshot clone of the status if present.
    #[must_use]
    pub fn pipeline_status(&self, pipeline_key: &PipelineKey) -> Option<PipelineStatus> {
        let pipelines = self.pipelines.lock().ok()?;
        pipelines.get(pipeline_key).cloned()
    }
}

impl ObservedEvent {
    /// Creates a new `PipelinePhase` event.
    #[must_use]
    pub fn pipeline_phase_event(
        pipeline_group_id: PipelineGroupId,
        pipeline_id: PipelineId,
        core_id: CoreId,
        phase: PipelinePhase,
    ) -> Self {
        ObservedEvent::PipelinePhase {
            key: PipelineKey {
                pipeline_group_id,
                pipeline_id,
            },
            core_id,
            phase,
        }
    }

    /// Creates a new `PipelinePhase::Pending` event.
    #[must_use]
    pub fn pipeline_pending(pipeline_key: DeployedPipelineKey) -> Self {
        Self::pipeline_phase_event(
            pipeline_key.pipeline_group_id,
            pipeline_key.pipeline_id,
            pipeline_key.core_id,
            PipelinePhase::Pending,
        )
    }

    /// Creates a new `PipelinePhase::Running` event.
    #[must_use]
    pub fn pipeline_running(pipeline_key: DeployedPipelineKey) -> Self {
        Self::pipeline_phase_event(
            pipeline_key.pipeline_group_id,
            pipeline_key.pipeline_id,
            pipeline_key.core_id,
            PipelinePhase::Running,
        )
    }

    /// Creates a new `PipelinePhase::Draining` event.
    #[must_use]
    pub fn pipeline_draining(pipeline_key: DeployedPipelineKey) -> Self {
        Self::pipeline_phase_event(
            pipeline_key.pipeline_group_id,
            pipeline_key.pipeline_id,
            pipeline_key.core_id,
            PipelinePhase::Draining,
        )
    }

    /// Creates a new `PipelinePhase::Stopped` event.
    #[must_use]
    pub fn pipeline_stopped(pipeline_key: DeployedPipelineKey) -> Self {
        Self::pipeline_phase_event(
            pipeline_key.pipeline_group_id,
            pipeline_key.pipeline_id,
            pipeline_key.core_id,
            PipelinePhase::Stopped,
        )
    }

    /// Creates a new `PipelinePhase::Condition` event.
    #[must_use]
    pub fn pipeline_condition(
        pipeline_key: DeployedPipelineKey,
        ctype: ConditionType,
        status: ConditionStatus,
        reason: String,
        message: String,
        details: Option<ConditionDetails>,
    ) -> Self {
        ObservedEvent::Condition {
            key: PipelineKey {
                pipeline_group_id: pipeline_key.pipeline_group_id,
                pipeline_id: pipeline_key.pipeline_id,
            },
            core_id: pipeline_key.core_id,
            condition: Condition {
                r#type: ctype,
                status,
                reason,
                message,
                last_transition_time: SystemTime::now(),
                counts: None,
                details: details.and_then(|d| {
                    if d.node_errors.is_empty() {
                        None
                    } else {
                        Some(d)
                    }
                }),
            },
        }
    }

    /// Creates a new `PipelinePhase::Failed` event.
    #[must_use]
    pub fn pipeline_failed(pipeline_key: DeployedPipelineKey) -> Self {
        Self::pipeline_phase_event(
            pipeline_key.pipeline_group_id,
            pipeline_key.pipeline_id,
            pipeline_key.core_id,
            PipelinePhase::Failed,
        )
    }

    /// Creates a new `PipelinePhase::Unknown` event.
    #[must_use]
    pub fn pipeline_unknown(pipeline_key: DeployedPipelineKey) -> Self {
        Self::pipeline_phase_event(
            pipeline_key.pipeline_group_id,
            pipeline_key.pipeline_id,
            pipeline_key.core_id,
            PipelinePhase::Unknown,
        )
    }

    /// Creates a new `Heartbeat` event.
    #[must_use]
    pub fn heartbeat_event(pipeline_key: DeployedPipelineKey) -> Self {
        ObservedEvent::Heartbeat {
            key: PipelineKey {
                pipeline_group_id: pipeline_key.pipeline_group_id,
                pipeline_id: pipeline_key.pipeline_id,
            },
            core_id: pipeline_key.core_id,
        }
    }
}

/// Aggregates multiple `PipelinePhase` values into a single representative phase.
/// This function bubble up the most consequential state so operators don't miss failures/drains in
/// progress, and don't show Stopped while some cores are actually running.
///
/// The function applies this priority order:
///
/// - Failed (if any core is Failed)
/// - Draining (if any core is Draining)
/// - Running
///   - Running if all cores are Running, or
///   - Running if any core is Running (even if others are Pending/Stopped/Unknown)
/// - Stopped (only if all cores are Stopped)
/// - Pending (if any core is Pending and none above matched)
/// - Unknown (if any core is Unknown and none above matched, or if there are no cores)
///
/// Examples:
/// - `[Running, Running]` => Running (all running)
/// - `[Running, Stopped]` => Running (some cores still working)
/// - `[Draining, Running]` => Draining (drain in progress)
/// - `[Failed, Running]` => Failed (fail fast)
/// - `[Stopped, Stopped]` => Stopped (fully stopped)
/// - `[Pending, Stopped]` => Pending (bring-up in progress)
/// - `[Unknown, Stopped]` => Unknown (no running/draining/failure and not all stopped)
fn aggregate_pipeline_phase<I>(phases: I) -> PipelinePhase
where
    I: IntoIterator<Item = PipelinePhase>,
{
    use PipelinePhase::*;

    let mut saw_any = false;
    let mut any_failed = false;
    let mut any_draining = false;
    let mut any_running = false;
    let mut any_pending = false;
    let mut any_unknown = false;

    let mut all_running = true;
    let mut all_stopped = true;

    for p in phases {
        saw_any = true;
        match p {
            Failed => {
                any_failed = true;
                all_running = false;
                all_stopped = false;
            }
            Draining => {
                any_draining = true;
                all_running = false;
                all_stopped = false;
            }
            Running => {
                any_running = true;
                all_stopped = false;
            }
            Pending => {
                any_pending = true;
                all_running = false;
                all_stopped = false;
            }
            Stopped => {
                all_running = false;
            }
            Unknown => {
                any_unknown = true;
                all_running = false;
                all_stopped = false;
            }
        }
    }

    if !saw_any {
        return Unknown;
    }
    if any_failed {
        return Failed;
    }
    if any_draining {
        return Draining;
    }
    if all_running {
        return Running;
    }
    if any_running {
        return Running;
    }
    if all_stopped {
        return Stopped;
    }
    if any_pending {
        return Pending;
    }
    if any_unknown {
        return Unknown;
    }
    Unknown
}

fn aggregate_condition_status<I>(statuses: I) -> ConditionStatus
where
    I: IntoIterator<Item = ConditionStatus>,
{
    let mut saw_any = false;
    let mut any_false = false;
    let mut any_unknown = false;

    for s in statuses {
        saw_any = true;
        match s {
            False => any_false = true,
            Unknown => any_unknown = true,
            True => {}
        }
    }

    if !saw_any {
        return Unknown;
    }
    if any_false {
        return False;
    }
    if any_unknown {
        return Unknown;
    }
    True
}

#[cfg(test)]
mod tests {
    use crate::store::PipelinePhase::*;
    use crate::store::{ConditionType, PipelinePhase, aggregate_pipeline_phase};
    use std::str::FromStr;

    #[test]
    fn aggregate_pipeline_phase_basics() {
        // empty => Unknown
        assert_eq!(
            aggregate_pipeline_phase(Vec::<PipelinePhase>::new()),
            Unknown
        );

        // all running => Running
        assert_eq!(aggregate_pipeline_phase([Running, Running]), Running);

        // mixed running/stopped => Running
        assert_eq!(aggregate_pipeline_phase([Running, Stopped]), Running);

        // any draining dominates running => Draining
        assert_eq!(aggregate_pipeline_phase([Draining, Running]), Draining);

        // any failed dominates everything => Failed
        assert_eq!(
            aggregate_pipeline_phase([Failed, Running, Draining]),
            Failed
        );

        // all stopped => Stopped
        assert_eq!(aggregate_pipeline_phase([Stopped, Stopped]), Stopped);

        // pending + stopped (no running/draining/failed) => Pending
        assert_eq!(aggregate_pipeline_phase([Pending, Stopped]), Pending);

        // unknown + stopped (no running/draining/failed and not all stopped) => Unknown
        assert_eq!(aggregate_pipeline_phase([Unknown, Stopped]), Unknown);

        // unknown + pending + stopped => Pending wins over Unknown
        assert_eq!(
            aggregate_pipeline_phase([Unknown, Pending, Stopped]),
            Pending
        );
    }

    #[test]
    fn condition_type_from_str_covers_builtins() {
        let cases = [
            ("Healthy", ConditionType::Healthy),
            ("Ready", ConditionType::Ready),
            ("BackpressureHigh", ConditionType::BackpressureHigh),
            ("HeartbeatMissing", ConditionType::HeartbeatMissing),
            ("RolloutInProgress", ConditionType::RolloutInProgress),
            ("ConfigRejected", ConditionType::ConfigRejected),
            ("Panic", ConditionType::Panic),
            ("StartError", ConditionType::StartError),
        ];

        for (input, expected) in cases {
            assert_eq!(ConditionType::from_str(input).unwrap(), expected);
        }
    }

    #[test]
    fn condition_type_from_str_custom_falls_back() {
        let parsed = ConditionType::from_str("VendorSpecific").unwrap();
        assert_eq!(parsed, ConditionType::Custom("VendorSpecific".to_owned()));
    }
}
