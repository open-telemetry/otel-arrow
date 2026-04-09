// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use crate::error::ValidationError;
use crate::metrics_types::{MetricSetSnapshot, MetricsSnapshot};
use otap_df_admin_api::{
    AdminClient, AdminEndpoint, HttpAdminClientSettings, engine::ProbeStatus,
    operations::OperationOptions, pipeline_groups::ShutdownStatus, telemetry::MetricsOptions,
};
use otap_df_config::engine::OtelDataflowSpec;
use otap_df_controller::Controller;
use otap_df_otap::OTAP_PIPELINE_FACTORY;
use std::collections::HashMap;
use tokio::time::{Duration, sleep};

const LOADGEN_METRIC_SET: &str = "fake_data_generator.receiver.metrics";
const LOADGEN_METRIC_NAME_LOGS: &str = "logs.produced";
const LOADGEN_METRIC_NAME_METRICS: &str = "metrics.produced";
const LOADGEN_TRACE_NAME_SPANS: &str = "spans.produced";
const VALIDATION_METRIC_SET: &str = "validation.exporter.metrics";
const VALIDATION_METRIC_NAME: &str = "valid";
const VALIDATION_FINISHED_METRIC_NAME: &str = "finished";

pub(crate) async fn run_pipelines_with_timeout(
    rendered_group: String,
    admin_base: String,
    expected_generator_signals: HashMap<String, u64>,
    timeout: Duration,
    ready_max_attempts: usize,
    ready_backoff: Duration,
    metrics_poll: Duration,
) -> Result<(), ValidationError> {
    let pipeline_simulator = PipelineSimulator::new(rendered_group.as_str())?;
    let _pipeline_handle = std::thread::spawn(move || pipeline_simulator.run());
    let admin_client = admin_client(&admin_base)?;

    wait_for_ready(&admin_client, ready_max_attempts, ready_backoff).await?;
    tokio::time::timeout(timeout, async {
        wait_for_loadgen(&admin_client, &expected_generator_signals, metrics_poll).await?;
        let result = wait_for_validation_finished(&admin_client, metrics_poll).await;
        shutdown_pipeline(&admin_client).await?;
        result
    })
    .await
    .map_err(|_| ValidationError::Validation(format!("scenario timed out after {timeout:?}")))?
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

/// loop until traffic generation is done
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

/// Poll metrics until every validation exporter reports `finished == 1`,
/// then check the `valid` gauge from the same snapshot. This replaces the
/// previous fixed propagation-delay sleep followed by a single metrics check.
async fn wait_for_validation_finished(
    client: &AdminClient,
    metrics_poll: Duration,
) -> Result<(), ValidationError> {
    loop {
        let snapshot = fetch_metrics(client).await?;
        match validation_finished_and_passed(&snapshot) {
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

/// shutdown pipeline after running
async fn shutdown_pipeline(client: &AdminClient) -> Result<(), ValidationError> {
    let response = client
        .pipeline_groups()
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

fn admin_error(err: otap_df_admin_api::Error) -> ValidationError {
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
// basically gets the name of the node from metrics via metric attributes
fn attribute_node_id(
    attributes: &HashMap<String, otap_df_telemetry::attributes::AttributeValue>,
) -> Option<String> {
    use otap_df_telemetry::attributes::AttributeValue;
    match attributes.get("node.id") {
        Some(AttributeValue::String(v)) => Some(v.clone()),
        _ => None,
    }
}

fn loadgen_reached_limit(
    snapshot: &MetricsSnapshot,
    expected_per_gen: &HashMap<String, u64>,
) -> bool {
    if expected_per_gen.is_empty() {
        return true;
    }

    let mut iter = snapshot
        .metric_sets
        .iter()
        .filter(|set| set.name == LOADGEN_METRIC_SET)
        .filter_map(|set| attribute_node_id(&set.attributes).map(|label| (set, label)))
        .peekable();

    // No loadgen metric sets found yet — generators have not reported their
    // first telemetry tick. Keep polling.
    if iter.peek().is_none() {
        return false;
    }

    iter.all(|(set, label)| {
        let loadgen_signals_produced = metric_value(set, LOADGEN_METRIC_NAME_LOGS).unwrap_or(0)
            + metric_value(set, LOADGEN_METRIC_NAME_METRICS).unwrap_or(0)
            + metric_value(set, LOADGEN_TRACE_NAME_SPANS).unwrap_or(0);
        loadgen_signals_produced >= *expected_per_gen.get(&label).unwrap_or(&0u64)
    })
}

/// Result of checking whether all validation exporters have finished.
#[derive(Debug)]
enum ValidationPollResult {
    /// At least one exporter has not signaled `finished` yet.
    NotFinished,
    /// All exporters finished and all report `valid >= 1`.
    FinishedAndPassed,
    /// All exporters finished but one or more report `valid < 1`.
    FinishedWithFailures(String),
}

/// Check a single metrics snapshot for the `finished` and `valid` gauges of
/// every validation exporter. Returns a single result covering all exporters
/// so that the caller can decide to keep polling or report success/failure.
fn validation_finished_and_passed(snapshot: &MetricsSnapshot) -> ValidationPollResult {
    let mut failed_validation_exporters = vec![];

    let mut iter = snapshot
        .metric_sets
        .iter()
        .filter(|set: &&MetricSetSnapshot| set.name == VALIDATION_METRIC_SET)
        .peekable();

    if iter.peek().is_none() {
        return ValidationPollResult::NotFinished;
    }

    for set in iter {
        // If any exporter has not yet signaled finished, keep waiting.
        if metric_value(set, VALIDATION_FINISHED_METRIC_NAME).is_none_or(|v| v < 1) {
            return ValidationPollResult::NotFinished;
        }
        // Exporter is finished — check its validation result in the same pass.
        if metric_value(set, VALIDATION_METRIC_NAME).is_some_and(|v| v < 1) {
            if let Some(label) = attribute_node_id(&set.attributes) {
                failed_validation_exporters.push(label);
            }
        }
    }

    if failed_validation_exporters.is_empty() {
        ValidationPollResult::FinishedAndPassed
    } else {
        ValidationPollResult::FinishedWithFailures(failed_validation_exporters.join(", "))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::metrics_types::{MetricDataPoint, MetricSetSnapshot, MetricsSnapshot};
    use otap_df_telemetry::descriptor::{Instrument, MetricValueType, Temporality};
    use otap_df_telemetry::metrics::MetricValue;
    use std::collections::HashMap;
    use wiremock::matchers::{method, path, query_param};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    fn set_with_node(set_name: &str, metric: &str, value: u64, node_id: &str) -> MetricSetSnapshot {
        MetricSetSnapshot {
            name: set_name.into(),
            brief: "test".into(),
            attributes: HashMap::from([(
                "node.id".into(),
                otap_df_telemetry::attributes::AttributeValue::String(node_id.into()),
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

    #[test]
    fn loadgen_reached_limit_uses_labels() {
        let snap = MetricsSnapshot {
            timestamp: "t".into(),
            metric_sets: vec![
                set_with_node(LOADGEN_METRIC_SET, LOADGEN_METRIC_NAME_LOGS, 10, "genA"),
                set_with_node(LOADGEN_METRIC_SET, LOADGEN_METRIC_NAME_LOGS, 4, "genB"),
            ],
        };
        let mut expected = HashMap::new();
        _ = expected.insert("genA".into(), 5);
        _ = expected.insert("genB".into(), 4);
        assert!(loadgen_reached_limit(&snap, &expected));

        _ = expected.insert("genB".into(), 5);
        assert!(!loadgen_reached_limit(&snap, &expected));
    }

    #[test]
    fn loadgen_empty_snapshot_returns_false() {
        let snap = MetricsSnapshot {
            timestamp: "t".into(),
            metric_sets: vec![],
        };
        let mut expected = HashMap::new();
        _ = expected.insert("genA".into(), 5);
        assert!(!loadgen_reached_limit(&snap, &expected));
    }

    fn validation_set(valid: u64, finished: u64, node_id: &str) -> Vec<MetricSetSnapshot> {
        vec![MetricSetSnapshot {
            name: VALIDATION_METRIC_SET.into(),
            brief: "test".into(),
            attributes: HashMap::from([(
                "node.id".into(),
                otap_df_telemetry::attributes::AttributeValue::String(node_id.into()),
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

    #[test]
    fn not_finished_keeps_polling() {
        let snap = MetricsSnapshot {
            timestamp: "t".into(),
            metric_sets: validation_set(0, 0, "cap1"),
        };
        assert!(matches!(
            validation_finished_and_passed(&snap),
            ValidationPollResult::NotFinished
        ));
    }

    #[test]
    fn finished_and_passed_returns_ok() {
        let snap = MetricsSnapshot {
            timestamp: "t".into(),
            metric_sets: validation_set(1, 1, "cap1"),
        };
        assert!(matches!(
            validation_finished_and_passed(&snap),
            ValidationPollResult::FinishedAndPassed
        ));
    }

    #[test]
    fn finished_with_failures_reports_labels() {
        let mut sets = validation_set(1, 1, "cap1");
        sets.extend(validation_set(0, 1, "cap2"));
        let snap = MetricsSnapshot {
            timestamp: "t".into(),
            metric_sets: sets,
        };
        match validation_finished_and_passed(&snap) {
            ValidationPollResult::FinishedWithFailures(failed) => {
                assert_eq!(failed, "cap2");
            }
            other => panic!("expected FinishedWithFailures, got {other:?}"),
        }
    }

    #[test]
    fn mixed_finished_returns_not_finished() {
        let mut sets = validation_set(1, 1, "cap1");
        sets.extend(validation_set(0, 0, "cap2"));
        let snap = MetricsSnapshot {
            timestamp: "t".into(),
            metric_sets: sets,
        };
        assert!(matches!(
            validation_finished_and_passed(&snap),
            ValidationPollResult::NotFinished
        ));
    }

    #[test]
    fn empty_snapshot_returns_not_finished() {
        let snap = MetricsSnapshot {
            timestamp: "t".into(),
            metric_sets: vec![],
        };
        assert!(matches!(
            validation_finished_and_passed(&snap),
            ValidationPollResult::NotFinished
        ));
    }

    #[test]
    fn no_matching_metric_sets_returns_not_finished() {
        let snap = MetricsSnapshot {
            timestamp: "t".into(),
            metric_sets: vec![set_with_node(
                LOADGEN_METRIC_SET,
                LOADGEN_METRIC_NAME_LOGS,
                100,
                "genA",
            )],
        };
        assert!(matches!(
            validation_finished_and_passed(&snap),
            ValidationPollResult::NotFinished
        ));
    }

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
            .and(path("/api/v1/pipeline-groups/shutdown"))
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
            &HashMap::from([(String::from("genA"), 7)])
        ));
        shutdown_pipeline(&client)
            .await
            .expect("shutdown should accept");
    }
}
