// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Engine lifecycle for multi-stage validation scenarios.
//!
//! The engine is started once from the first stage's rendered group. Each
//! subsequent stage is reached with the live-update (reconfigure) API: the
//! `suv` pipeline plus every generator and capture pipeline are reconfigured in
//! place, so the engine is never restarted between stages.
//!
//! Because a live update relaunches a pipeline instance (and drains the old
//! one), every stage runs with a fresh traffic generator and a fresh validation
//! exporter. Stage completion is detected edge-triggered against a per-stage
//! baseline of the cumulative `produced` and `finished` counters, which is
//! robust to the cumulative, per-instance nature of the metrics.

use crate::error::ValidationError;
use crate::metrics_types::{MetricSetSnapshot, MetricsSnapshot};
use crate::scenario::{SUV_PIPELINE_ID, VALIDATION_GROUP_ID};
use crate::stage::RolloutAction;
use otel_arrow_dfe_admin_api::{
    AdminClient, AdminEndpoint, HttpAdminClientSettings, engine::ProbeStatus,
    groups::ShutdownStatus, operations::OperationOptions, pipelines::ReconfigureOutcome,
    pipelines::ReconfigureRequest, telemetry::MetricsOptions,
};
use otel_arrow_dfe_config::engine::OtelDataflowSpec;
use otel_arrow_dfe_config::pipeline::PipelineConfig;
use otel_arrow_dfe_controller::Controller;
use otel_arrow_dfe_otap::OTAP_PIPELINE_FACTORY;
use std::collections::HashMap;
use tokio::time::{Duration, sleep};

const LOADGEN_METRIC_SET: &str = "receiver.traffic_generator";
const LOADGEN_METRIC_NAME_LOGS: &str = "logs.produced";
const LOADGEN_METRIC_NAME_METRICS: &str = "metrics.produced";
const LOADGEN_TRACE_NAME_SPANS: &str = "spans.produced";
const VALIDATION_METRIC_SET: &str = "exporter.validation";
const VALIDATION_METRIC_NAME: &str = "valid";
const VALIDATION_FINISHED_METRIC_NAME: &str = "finished";

/// A single executable validation stage produced by the scenario builder.
///
/// It carries the rendered group YAML (the first stage's YAML starts the
/// engine), the per-pipeline configs used to reconfigure into this stage, and
/// the expectations used to detect and assert stage completion.
#[derive(Debug)]
pub(crate) struct StagePlan {
    /// Human-readable stage label (also used in error messages).
    pub(crate) label: String,
    /// Full `validation_test` group YAML rendered for this stage.
    pub(crate) rendered_group: String,
    /// Per-pipeline configs keyed by pipeline id (`suv` plus each generator and
    /// capture label). Used to build reconfigure requests.
    pub(crate) pipeline_configs: HashMap<String, PipelineConfig>,
    /// Expected produced-signal count per generator label for this stage.
    pub(crate) expected_signals: HashMap<String, u64>,
    /// Capture pipeline labels whose validation exporters must finish+pass.
    pub(crate) capture_labels: Vec<String>,
    /// Optional assertion on the `suv` rollout classification for this stage.
    pub(crate) expected_action: Option<RolloutAction>,
    /// Per-core admission/ready timeout for reconfigure into this stage.
    pub(crate) step_timeout_secs: u64,
    /// Graceful drain timeout for reconfigure into this stage.
    pub(crate) drain_timeout_secs: u64,
    /// Per-stage timeout in seconds. This stage's combined load-generation and
    /// validation work must complete within this budget.
    pub(crate) stage_timeout_secs: u64,
}

/// Run every stage in order against a single engine, transitioning between
/// stages with the live-update API. The engine is started from the first
/// stage and shut down only after the final stage's validation completes.
pub(crate) async fn run_stages_with_timeout(
    stages: Vec<StagePlan>,
    admin_base: String,
    timeout: Duration,
    ready_max_attempts: usize,
    ready_backoff: Duration,
    metrics_poll: Duration,
) -> Result<(), ValidationError> {
    let first = stages
        .first()
        .ok_or_else(|| ValidationError::Config("no stages to run".into()))?;
    let simulator = PipelineSimulator::new(first.rendered_group.as_str())?;
    let _pipeline_handle = std::thread::spawn(move || simulator.run());
    let admin_client = admin_client(&admin_base)?;

    wait_for_ready(&admin_client, ready_max_attempts, ready_backoff).await?;

    tokio::time::timeout(timeout, async {
        for (index, stage) in stages.iter().enumerate() {
            // Transition into this stage with a live update (stages after the
            // first). The reconfigure waits for the target generation to serve
            // and the previous generation to drain, so by the time it returns
            // only this stage's fresh instances report telemetry.
            if index > 0 {
                reconfigure_stage(&admin_client, stage).await?;
            }

            // A single per-stage budget bounds this stage's load generation and
            // validation together.
            let stage_budget = Duration::from_secs(stage.stage_timeout_secs);

            tokio::time::timeout(stage_budget, async {
                wait_for_loadgen(&admin_client, &stage.expected_signals, metrics_poll)
                    .await
                    .map_err(|e| stage_error(&stage.label, e))?;
                wait_for_validation_finished(&admin_client, &stage.capture_labels, metrics_poll)
                    .await
                    .map_err(|e| stage_error(&stage.label, e))?;
                Ok::<(), ValidationError>(())
            })
            .await
            .map_err(|_| {
                ValidationError::Validation(format!(
                    "stage '{}': timed out after {stage_budget:?}",
                    stage.label
                ))
            })??;
        }
        shutdown_pipeline(&admin_client).await
    })
    .await
    .map_err(|_| ValidationError::Validation(format!("scenario timed out after {timeout:?}")))?
}

/// Prefix a stage-scoped error with the stage label for easier diagnosis.
///
/// Only the variants that a running stage's phases can surface (validation
/// outcomes, admin HTTP failures, readiness failures) are re-labeled. Other
/// variants either already carry stage context or arise before execution.
fn stage_error(label: &str, err: ValidationError) -> ValidationError {
    match err {
        ValidationError::Validation(msg) => {
            ValidationError::Validation(format!("stage '{label}': {msg}"))
        }
        ValidationError::Http(msg) => ValidationError::Http(format!("stage '{label}': {msg}")),
        ValidationError::Ready(msg) => ValidationError::Ready(format!("stage '{label}': {msg}")),
        other => other,
    }
}

struct PipelineSimulator {
    engine_config: OtelDataflowSpec,
}

impl PipelineSimulator {
    fn new(yaml: &str) -> Result<Self, ValidationError> {
        let engine_config = OtelDataflowSpec::from_yaml(yaml)
            .map_err(|e| ValidationError::Config(e.to_string()))?;
        Ok(Self { engine_config })
    }

    fn run(&self) {
        let controller = Controller::new(&OTAP_PIPELINE_FACTORY);
        let engine_config = self.engine_config.clone();
        let _ = controller.run_till_shutdown(engine_config);
    }
}

/// Reconfigure the `suv` pipeline plus every generator and capture pipeline
/// into the target stage. Captures are reconfigured first (so downstream
/// receivers are ready), then the `suv` pipeline, then generators.
async fn reconfigure_stage(client: &AdminClient, stage: &StagePlan) -> Result<(), ValidationError> {
    // Order across buckets is deterministic: captures first (so downstream
    // receivers are ready), then suv, then generators. Order within the
    // generators bucket is unspecified (HashMap iteration) and safe because
    // generators are independent of one another.
    let mut ordered: Vec<&String> = stage.pipeline_configs.keys().collect();
    ordered.sort_by_key(|id| {
        if stage.capture_labels.iter().any(|c| c == *id) {
            0
        } else if id.as_str() == SUV_PIPELINE_ID {
            1
        } else {
            2
        }
    });

    for pipeline_id in ordered {
        let config = stage
            .pipeline_configs
            .get(pipeline_id)
            .expect("pipeline id came from the same map");
        let request = ReconfigureRequest {
            pipeline: config.clone(),
            step_timeout_secs: stage.step_timeout_secs,
            drain_timeout_secs: stage.drain_timeout_secs,
        };
        let options = OperationOptions {
            wait: true,
            timeout_secs: stage.step_timeout_secs.max(stage.drain_timeout_secs) + 10,
        };
        let outcome = client
            .pipelines()
            .reconfigure(VALIDATION_GROUP_ID, pipeline_id, &request, &options)
            .await
            .map_err(|e| {
                ValidationError::Reconfigure(format!(
                    "stage '{}': reconfigure of '{pipeline_id}' failed: {e}",
                    stage.label
                ))
            })?;

        match outcome {
            ReconfigureOutcome::Completed(status) => {
                if pipeline_id.as_str() == SUV_PIPELINE_ID {
                    assert_expected_action(stage, &status.action)?;
                }
            }
            ReconfigureOutcome::Accepted(status) => {
                return Err(ValidationError::Reconfigure(format!(
                    "stage '{}': reconfigure of '{pipeline_id}' returned Accepted (rollout {}) \
                     but a terminal result was requested",
                    stage.label, status.rollout_id
                )));
            }
            ReconfigureOutcome::Failed(status) | ReconfigureOutcome::TimedOut(status) => {
                return Err(ValidationError::Reconfigure(format!(
                    "stage '{}': reconfigure of '{pipeline_id}' did not succeed: state={:?} reason={:?}",
                    stage.label, status.state, status.failure_reason
                )));
            }
        }
    }
    Ok(())
}

/// Assert the `suv` rollout classification matches the stage expectation.
fn assert_expected_action(stage: &StagePlan, observed_action: &str) -> Result<(), ValidationError> {
    if let Some(expected) = stage.expected_action {
        let observed = RolloutAction::from_wire(observed_action);
        if observed != Some(expected) {
            return Err(ValidationError::Reconfigure(format!(
                "stage '{}': expected rollout action {} but engine classified it as {observed_action}",
                stage.label,
                expected.as_str()
            )));
        }
    }
    Ok(())
}

async fn wait_for_ready(
    client: &AdminClient,
    max_retry: usize,
    retry_cooldown: Duration,
) -> Result<(), ValidationError> {
    let mut last_error: Option<String> = None;
    for _attempt in 0..max_retry {
        match client.engine().readyz().await {
            Ok(resp) if resp.status == ProbeStatus::Ok => return Ok(()),
            Ok(resp) => {
                let details = serde_json::to_string(&resp)
                    .unwrap_or_else(|_| format!("probe status={:?}", resp.status));
                last_error = Some(format!("pipeline is not ready: {details}"));
            }
            Err(err) => {
                last_error = Some(format!("pipeline is not ready: {err}"));
            }
        }

        sleep(retry_cooldown).await;
    }

    Err(ValidationError::Ready(
        last_error.unwrap_or_else(|| "readyz timeout".to_string()),
    ))
}

async fn fetch_metrics(client: &AdminClient) -> Result<MetricsSnapshot, ValidationError> {
    let response = client
        .telemetry()
        .metrics(&MetricsOptions {
            reset: false,
            keep_all_zeroes: true,
        })
        .await
        .map_err(admin_error)?;

    serde_json::from_value(
        serde_json::to_value(response).map_err(|e| ValidationError::Http(e.to_string()))?,
    )
    .map_err(|e| ValidationError::Http(e.to_string()))
}

/// Poll until each generator label of the current stage has produced at least
/// its expected number of signals.
///
/// A live-update reconfigure waits for the previous generation to drain before
/// returning, so once a stage is active only this stage's fresh generator
/// instances report telemetry and their counters start from zero. Absolute
/// thresholds are therefore correct per stage.
async fn wait_for_loadgen(
    client: &AdminClient,
    expected_generator_signals: &HashMap<String, u64>,
    metrics_poll: Duration,
) -> Result<(), ValidationError> {
    loop {
        let snapshot = fetch_metrics(client).await?;
        if loadgen_reached_limit(&snapshot, expected_generator_signals) {
            return Ok(());
        }
        sleep(metrics_poll).await;
    }
}

/// Poll until every capture's validation exporter for the current stage reports
/// `finished >= 1`, then evaluate `valid`.
async fn wait_for_validation_finished(
    client: &AdminClient,
    capture_labels: &[String],
    metrics_poll: Duration,
) -> Result<(), ValidationError> {
    loop {
        let snapshot = fetch_metrics(client).await?;
        match validation_finished_and_passed(&snapshot, capture_labels) {
            ValidationPollResult::NotFinished => {
                sleep(metrics_poll).await;
            }
            ValidationPollResult::FinishedAndPassed => return Ok(()),
            ValidationPollResult::FinishedWithFailures(failed) => {
                return Err(ValidationError::Validation(format!(
                    "validation exporters did not report success: {failed}",
                )));
            }
        }
    }
}

/// shutdown pipeline after running all stages
async fn shutdown_pipeline(client: &AdminClient) -> Result<(), ValidationError> {
    let response = client
        .groups()
        .shutdown(&OperationOptions::default())
        .await
        .map_err(admin_error)?;

    match response.status {
        ShutdownStatus::Accepted | ShutdownStatus::Completed => Ok(()),
        ShutdownStatus::Failed | ShutdownStatus::Timeout => Err(ValidationError::Http(
            serde_json::to_string(&response).unwrap_or_else(|_| format!("{response:?}")),
        )),
    }
}

fn admin_client(admin_base: &str) -> Result<AdminClient, ValidationError> {
    let endpoint =
        AdminEndpoint::from_url(admin_base).map_err(|e| ValidationError::Http(e.to_string()))?;
    AdminClient::builder()
        .http(HttpAdminClientSettings::new(endpoint))
        .build()
        .map_err(admin_error)
}

fn admin_error(err: otel_arrow_dfe_admin_api::Error) -> ValidationError {
    ValidationError::Http(err.to_string())
}

// get the value from a specific metric given the snapshot
fn metric_value(set: &MetricSetSnapshot, metric_name: &str) -> Option<u64> {
    set.metrics
        .iter()
        .find(|m| m.name == metric_name)
        .map(|m| m.value.to_u64_lossy())
}

// get value from attribute with key node.id
fn attribute_node_id(
    attributes: &HashMap<String, otel_arrow_dfe_telemetry::attributes::AttributeValue>,
) -> Option<String> {
    use otel_arrow_dfe_telemetry::attributes::AttributeValue;
    match attributes.get("node.id") {
        Some(AttributeValue::String(v)) => Some(v.clone()),
        _ => None,
    }
}

/// Total produced signals per generator label (summed across all instances,
/// including any old drained instance sharing the same `node.id`).
fn loadgen_totals(snapshot: &MetricsSnapshot) -> HashMap<String, u64> {
    let mut totals = HashMap::new();
    for set in snapshot
        .metric_sets
        .iter()
        .filter(|set| set.name == LOADGEN_METRIC_SET)
    {
        if let Some(label) = attribute_node_id(&set.attributes) {
            let produced = metric_value(set, LOADGEN_METRIC_NAME_LOGS).unwrap_or(0)
                + metric_value(set, LOADGEN_METRIC_NAME_METRICS).unwrap_or(0)
                + metric_value(set, LOADGEN_TRACE_NAME_SPANS).unwrap_or(0);
            *totals.entry(label).or_insert(0) += produced;
        }
    }
    totals
}

/// Total `finished` per capture label (summed across instances).
fn finished_totals(snapshot: &MetricsSnapshot) -> HashMap<String, u64> {
    let mut totals = HashMap::new();
    for set in snapshot
        .metric_sets
        .iter()
        .filter(|set| set.name == VALIDATION_METRIC_SET)
    {
        if let Some(label) = attribute_node_id(&set.attributes) {
            let finished = metric_value(set, VALIDATION_FINISHED_METRIC_NAME).unwrap_or(0);
            *totals.entry(label).or_insert(0) += finished;
        }
    }
    totals
}

/// True when every expected generator label has produced at least its expected
/// count of signals for the current stage.
fn loadgen_reached_limit(
    snapshot: &MetricsSnapshot,
    expected_per_gen: &HashMap<String, u64>,
) -> bool {
    if expected_per_gen.is_empty() {
        return true;
    }
    let totals = loadgen_totals(snapshot);
    // Require every expected generator to have reported telemetry and reached
    // its target so a not-yet-started generator is not treated as complete.
    expected_per_gen
        .iter()
        .all(|(label, expected)| totals.get(label).copied().unwrap_or(0) >= *expected)
}

/// Result of checking whether all validation exporters have finished for the
/// current stage.
#[derive(Debug)]
enum ValidationPollResult {
    /// At least one exporter has not incremented `finished` past its baseline.
    NotFinished,
    /// All expected exporters finished and all report `valid >= 1`.
    FinishedAndPassed,
    /// All expected exporters finished but one or more report `valid < 1`.
    FinishedWithFailures(String),
}

/// Check a metrics snapshot for the `finished` and `valid` gauges of every
/// expected capture label of the current stage.
///
/// After a live-update reconfigure the previous generation has drained, so
/// each capture label maps to a single fresh validation exporter whose
/// `finished` gauge transitions from 0 to 1 once the stage settles.
fn validation_finished_and_passed(
    snapshot: &MetricsSnapshot,
    capture_labels: &[String],
) -> ValidationPollResult {
    if capture_labels.is_empty() {
        return ValidationPollResult::FinishedAndPassed;
    }

    let finished_now = finished_totals(snapshot);

    // Highest `valid` gauge observed per capture label across its instances.
    let mut valid_by_label: HashMap<&str, u64> = HashMap::new();
    for set in snapshot
        .metric_sets
        .iter()
        .filter(|set| set.name == VALIDATION_METRIC_SET)
    {
        if let Some(label) = attribute_node_id(&set.attributes) {
            let valid = metric_value(set, VALIDATION_METRIC_NAME).unwrap_or(0);
            if let Some(expected) = capture_labels.iter().find(|c| c.as_str() == label) {
                let entry = valid_by_label.entry(expected.as_str()).or_insert(0);
                *entry = (*entry).max(valid);
            }
        }
    }

    let mut failed = Vec::new();
    for label in capture_labels {
        if finished_now.get(label).copied().unwrap_or(0) < 1 {
            return ValidationPollResult::NotFinished;
        }
        if valid_by_label.get(label.as_str()).copied().unwrap_or(0) < 1 {
            failed.push(label.clone());
        }
    }

    if failed.is_empty() {
        ValidationPollResult::FinishedAndPassed
    } else {
        ValidationPollResult::FinishedWithFailures(failed.join(", "))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::metrics_types::MetricValue;
    use crate::metrics_types::{MetricDataPoint, MetricSetSnapshot, MetricsSnapshot};
    use otel_arrow_dfe_telemetry::descriptor::{Instrument, MetricValueType, Temporality};
    use std::collections::HashMap;
    use wiremock::matchers::{method, path, query_param};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    fn set_with_node(set_name: &str, metric: &str, value: u64, node_id: &str) -> MetricSetSnapshot {
        MetricSetSnapshot {
            name: set_name.into(),
            brief: "test".into(),
            attributes: HashMap::from([(
                "node.id".into(),
                otel_arrow_dfe_telemetry::attributes::AttributeValue::String(node_id.into()),
            )]),
            metrics: vec![MetricDataPoint {
                name: metric.into(),
                unit: "".into(),
                brief: "".into(),
                instrument: Instrument::Counter,
                temporality: Some(Temporality::Cumulative),
                value_type: MetricValueType::U64,
                value: MetricValue::U64(value),
            }],
        }
    }

    /// Scenario: two generators are checked against their per-stage targets.
    /// Guarantees: load-gen completion sums produced signals across instances
    /// of the same generator label and compares against the target count.
    #[test]
    fn loadgen_reached_limit_uses_targets() {
        let snap = MetricsSnapshot {
            timestamp: "t".into(),
            metric_sets: vec![
                set_with_node(LOADGEN_METRIC_SET, LOADGEN_METRIC_NAME_LOGS, 110, "genA"),
                set_with_node(LOADGEN_METRIC_SET, LOADGEN_METRIC_NAME_LOGS, 54, "genB"),
            ],
        };
        let expected = HashMap::from([("genA".to_string(), 100), ("genB".to_string(), 50)]);
        assert!(loadgen_reached_limit(&snap, &expected));

        let expected_high = HashMap::from([("genA".to_string(), 200), ("genB".to_string(), 50)]);
        assert!(!loadgen_reached_limit(&snap, &expected_high));
    }

    /// Scenario: no generator metric sets have been reported yet.
    /// Guarantees: load-gen is not considered complete when metrics are absent.
    #[test]
    fn loadgen_empty_snapshot_returns_false() {
        let snap = MetricsSnapshot {
            timestamp: "t".into(),
            metric_sets: vec![],
        };
        let expected = HashMap::from([("genA".to_string(), 5)]);
        assert!(!loadgen_reached_limit(&snap, &expected));
    }

    fn validation_set(valid: u64, finished: u64, node_id: &str) -> Vec<MetricSetSnapshot> {
        vec![MetricSetSnapshot {
            name: VALIDATION_METRIC_SET.into(),
            brief: "test".into(),
            attributes: HashMap::from([(
                "node.id".into(),
                otel_arrow_dfe_telemetry::attributes::AttributeValue::String(node_id.into()),
            )]),
            metrics: vec![
                MetricDataPoint {
                    name: VALIDATION_METRIC_NAME.into(),
                    unit: "".into(),
                    brief: "".into(),
                    instrument: Instrument::Counter,
                    temporality: Some(Temporality::Cumulative),
                    value_type: MetricValueType::U64,
                    value: MetricValue::U64(valid),
                },
                MetricDataPoint {
                    name: VALIDATION_FINISHED_METRIC_NAME.into(),
                    unit: "".into(),
                    brief: "".into(),
                    instrument: Instrument::Counter,
                    temporality: Some(Temporality::Cumulative),
                    value_type: MetricValueType::U64,
                    value: MetricValue::U64(finished),
                },
            ],
        }]
    }

    /// Scenario: a capture has not yet reported `finished` for the stage.
    /// Guarantees: the detector keeps polling until the exporter finishes.
    #[test]
    fn not_finished_keeps_polling() {
        let snap = MetricsSnapshot {
            timestamp: "t".into(),
            metric_sets: validation_set(0, 0, "cap1"),
        };
        assert!(matches!(
            validation_finished_and_passed(&snap, &["cap1".to_string()]),
            ValidationPollResult::NotFinished
        ));
    }

    /// Scenario: a capture reports finished and valid for the stage.
    /// Guarantees: the detector reports success for the stage.
    #[test]
    fn finished_and_passed_returns_ok() {
        let snap = MetricsSnapshot {
            timestamp: "t".into(),
            metric_sets: validation_set(1, 1, "cap1"),
        };
        assert!(matches!(
            validation_finished_and_passed(&snap, &["cap1".to_string()]),
            ValidationPollResult::FinishedAndPassed
        ));
    }

    /// Scenario: two captures finish but one reports invalid.
    /// Guarantees: the detector names the failing capture label.
    #[test]
    fn finished_with_failures_reports_labels() {
        let mut sets = validation_set(1, 1, "cap1");
        sets.extend(validation_set(0, 1, "cap2"));
        let snap = MetricsSnapshot {
            timestamp: "t".into(),
            metric_sets: sets,
        };
        let labels = vec!["cap1".to_string(), "cap2".to_string()];
        match validation_finished_and_passed(&snap, &labels) {
            ValidationPollResult::FinishedWithFailures(failed) => {
                assert_eq!(failed, "cap2");
            }
            other => panic!("expected FinishedWithFailures, got {other:?}"),
        }
    }

    /// Scenario: one of two captures has not finished for this stage.
    /// Guarantees: the detector keeps polling until all captures finish.
    #[test]
    fn mixed_finished_returns_not_finished() {
        let mut sets = validation_set(1, 1, "cap1");
        sets.extend(validation_set(0, 0, "cap2"));
        let snap = MetricsSnapshot {
            timestamp: "t".into(),
            metric_sets: sets,
        };
        let labels = vec!["cap1".to_string(), "cap2".to_string()];
        assert!(matches!(
            validation_finished_and_passed(&snap, &labels),
            ValidationPollResult::NotFinished
        ));
    }

    /// Scenario: no capture labels are expected for a stage.
    /// Guarantees: validation is trivially complete when there is nothing to
    /// assert.
    #[test]
    fn no_capture_labels_returns_passed() {
        let snap = MetricsSnapshot {
            timestamp: "t".into(),
            metric_sets: vec![],
        };
        assert!(matches!(
            validation_finished_and_passed(&snap, &[]),
            ValidationPollResult::FinishedAndPassed
        ));
    }

    /// Scenario: stage-scoped errors of each execution-phase variant are
    /// re-labeled with the stage name.
    /// Guarantees: `Validation`, `Http`, and `Ready` errors surfaced during a
    /// stage are prefixed with the stage label (and keep their variant), while
    /// pre-execution variants such as `Config` are left untouched.
    #[test]
    fn stage_error_labels_execution_phase_variants() {
        match stage_error("s0", ValidationError::Validation("boom".into())) {
            ValidationError::Validation(msg) => assert_eq!(msg, "stage 's0': boom"),
            other => panic!("expected Validation, got {other:?}"),
        }
        match stage_error("s0", ValidationError::Http("boom".into())) {
            ValidationError::Http(msg) => assert_eq!(msg, "stage 's0': boom"),
            other => panic!("expected Http, got {other:?}"),
        }
        match stage_error("s0", ValidationError::Ready("boom".into())) {
            ValidationError::Ready(msg) => assert_eq!(msg, "stage 's0': boom"),
            other => panic!("expected Ready, got {other:?}"),
        }
        // Pre-execution variants are passed through unchanged.
        match stage_error("s0", ValidationError::Config("boom".into())) {
            ValidationError::Config(msg) => assert_eq!(msg, "boom"),
            other => panic!("expected Config, got {other:?}"),
        }
    }

    fn stage_plan_with_action(label: &str, expected: Option<RolloutAction>) -> StagePlan {
        StagePlan {
            label: label.into(),
            rendered_group: String::new(),
            pipeline_configs: HashMap::new(),
            expected_signals: HashMap::new(),
            capture_labels: Vec::new(),
            expected_action: expected,
            step_timeout_secs: 1,
            drain_timeout_secs: 1,
            stage_timeout_secs: 1,
        }
    }

    /// Scenario: a stage expects a specific rollout action and the engine
    /// classifies the live update as a different action.
    /// Guarantees: the mismatch fails with a `Reconfigure` error that names the
    /// stage, the expected action, and the observed action.
    #[test]
    fn assert_expected_action_reports_mismatch() {
        let plan = stage_plan_with_action("s1", Some(RolloutAction::Replace));
        match assert_expected_action(&plan, "resize") {
            Err(ValidationError::Reconfigure(msg)) => {
                assert_eq!(
                    msg,
                    "stage 's1': expected rollout action replace but engine classified it as resize"
                );
            }
            other => panic!("expected Reconfigure error, got {other:?}"),
        }
    }

    /// Scenario: a stage expects a rollout action and the engine classifies the
    /// live update as exactly that action.
    /// Guarantees: a matching classification passes without error.
    #[test]
    fn assert_expected_action_accepts_match() {
        let plan = stage_plan_with_action("s1", Some(RolloutAction::Replace));
        assert!(assert_expected_action(&plan, "replace").is_ok());
    }

    /// Scenario: a stage does not assert any rollout action.
    /// Guarantees: the classification is not checked, so any observed action
    /// (including an unknown wire string) passes without error.
    #[test]
    fn assert_expected_action_skips_when_unset() {
        let plan = stage_plan_with_action("s1", None);
        assert!(assert_expected_action(&plan, "replace").is_ok());
        assert!(assert_expected_action(&plan, "totally-unknown").is_ok());
    }

    /// Scenario: the harness drives readiness, load-gen, and shutdown against a
    /// mocked admin API using the multi-stage helpers.
    /// Guarantees: the readiness, metrics, and shutdown helpers still speak the
    /// existing admin wire contract.
    #[tokio::test]
    async fn admin_client_helpers_follow_existing_validation_flow() {
        let server = MockServer::start().await;
        let metrics = MetricsSnapshot {
            timestamp: "2026-01-01T00:00:00Z".into(),
            metric_sets: vec![set_with_node(
                LOADGEN_METRIC_SET,
                LOADGEN_METRIC_NAME_LOGS,
                7,
                "genA",
            )],
        };

        Mock::given(method("GET"))
            .and(path("/api/v1/readyz"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "probe": "readyz",
                "status": "ok",
                "generatedAt": "2026-01-01T00:00:00Z",
                "failing": []
            })))
            .mount(&server)
            .await;

        Mock::given(method("GET"))
            .and(path("/api/v1/telemetry/metrics"))
            .and(query_param("reset", "false"))
            .and(query_param("keep_all_zeroes", "true"))
            .and(query_param("format", "json"))
            .respond_with(ResponseTemplate::new(200).set_body_json(&metrics))
            .mount(&server)
            .await;

        Mock::given(method("POST"))
            .and(path("/api/v1/groups/shutdown"))
            .and(query_param("wait", "false"))
            .and(query_param("timeout_secs", "60"))
            .respond_with(ResponseTemplate::new(202).set_body_json(serde_json::json!({
                "status": "accepted"
            })))
            .mount(&server)
            .await;

        let client = admin_client(&server.uri()).expect("client should build");
        wait_for_ready(&client, 1, Duration::from_millis(1))
            .await
            .expect("readyz should pass");
        let snapshot = fetch_metrics(&client).await.expect("metrics should decode");
        assert!(loadgen_reached_limit(
            &snapshot,
            &HashMap::from([(String::from("genA"), 7)]),
        ));
        shutdown_pipeline(&client)
            .await
            .expect("shutdown should accept");
    }
}
