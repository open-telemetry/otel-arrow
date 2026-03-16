// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use crate::error::ValidationError;
use crate::metrics_types::{MetricSetSnapshot, MetricsSnapshot};
use otap_df_config::engine::OtelDataflowSpec;
use otap_df_controller::Controller;
use otap_df_otap::OTAP_PIPELINE_FACTORY;
use reqwest::Client;
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
    let admin_client = Client::new();

    wait_for_ready(
        &admin_client,
        &admin_base,
        ready_max_attempts,
        ready_backoff,
    )
    .await?;
    tokio::time::timeout(timeout, async {
        wait_for_loadgen(
            &admin_client,
            &admin_base,
            &expected_generator_signals,
            metrics_poll,
        )
        .await?;
        let result = wait_for_validation_finished(&admin_client, &admin_base, metrics_poll).await;
        shutdown_pipeline(&admin_client, &admin_base).await?;
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
    client: &Client,
    base: &str,
    max_retry: usize,
    retry_cooldown: Duration,
) -> Result<(), ValidationError> {
    let readyz_url = format!("{base}/readyz");
    let mut last_error: Option<String> = None;
    for _attempt in 0..max_retry {
        match client.get(&readyz_url).send().await {
            Ok(resp) if resp.status().is_success() => return Ok(()),
            Ok(resp) => match resp.text().await {
                Ok(body) => last_error = Some(format!("pipeline is not ready: {body}")),
                Err(err) => last_error = Some(format!("pipeline is not ready: {err}")),
            },
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

async fn fetch_metrics(client: &Client, base: &str) -> Result<MetricsSnapshot, ValidationError> {
    client
        .get(format!("{base}/telemetry/metrics"))
        .query(&[
            ("reset", "false"),
            ("keep_all_zeroes", "true"),
            ("format", "json"),
        ])
        .send()
        .await
        .map_err(|_| ValidationError::Http(format!("No Response from {base}/telemetry/metrics")))?
        .error_for_status()
        .map_err(|e| ValidationError::Http(e.to_string()))?
        .json()
        .await
        .map_err(|e| ValidationError::Http(e.to_string()))
}

/// loop until traffic generation is done
async fn wait_for_loadgen(
    client: &Client,
    base: &str,
    expected_generator_signals: &HashMap<String, u64>,
    metrics_poll: Duration,
) -> Result<(), ValidationError> {
    loop {
        let snapshot = fetch_metrics(client, base).await?;
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
    client: &Client,
    base: &str,
    metrics_poll: Duration,
) -> Result<(), ValidationError> {
    loop {
        let snapshot = fetch_metrics(client, base).await?;
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
async fn shutdown_pipeline(client: &Client, base: &str) -> Result<(), ValidationError> {
    let _ = client
        .post(format!("{base}/pipeline-groups/shutdown"))
        .send()
        .await
        .map_err(|e| ValidationError::Http(e.to_string()))?
        .error_for_status()
        .map_err(|e| ValidationError::Http(e.to_string()))?;
    Ok(())
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

    let mut loadgen_metric_set_seen = false;

    for set in snapshot
        .metric_sets
        .iter()
        .filter(|set| set.name == LOADGEN_METRIC_SET)
    {
        if let Some(label) = attribute_node_id(&set.attributes) {
            loadgen_metric_set_seen = true;
            let loadgen_signals_produced = metric_value(set, LOADGEN_METRIC_NAME_LOGS).unwrap_or(0)
                + metric_value(set, LOADGEN_METRIC_NAME_METRICS).unwrap_or(0)
                + metric_value(set, LOADGEN_TRACE_NAME_SPANS).unwrap_or(0);
            if &loadgen_signals_produced < expected_per_gen.get(&label).unwrap_or(&0u64) {
                return false;
            }
        }
    }

    // No loadgen metric sets found yet — generators have not reported their
    // first telemetry tick. Keep polling.
    loadgen_metric_set_seen
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
    let mut validation_metrics_seen = false;

    for set in snapshot
        .metric_sets
        .iter()
        .filter(|set: &&MetricSetSnapshot| set.name == VALIDATION_METRIC_SET)
    {
        validation_metrics_seen = true;
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

    // No validation exporter metric sets found yet — the exporter has not
    // reported its first telemetry tick. Keep polling.
    if !validation_metrics_seen {
        return ValidationPollResult::NotFinished;
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
    fn validation_not_finished_keeps_polling() {
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
    fn validation_finished_and_passed_returns_ok() {
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
    fn validation_finished_with_failures_reports_labels() {
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
    fn validation_mixed_finished_returns_not_finished() {
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
    fn validation_empty_snapshot_returns_not_finished() {
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
    fn validation_no_matching_metric_sets_returns_not_finished() {
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
}
