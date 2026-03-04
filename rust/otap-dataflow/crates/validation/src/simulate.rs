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

pub(crate) async fn run_pipelines_with_timeout(
    rendered_group: String,
    admin_base: String,
    expected_generator_signals: HashMap<String, u64>,
    timeout: Duration,
    ready_max_attempts: usize,
    ready_backoff: Duration,
    metrics_poll: Duration,
    propagation_delay: Duration,
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
        sleep(propagation_delay).await;
        let result = ensure_validation_passed(&admin_client, &admin_base).await;
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

/// get metrics and check the validation metric
/// errors if validation failed
async fn ensure_validation_passed(client: &Client, base: &str) -> Result<(), ValidationError> {
    let snapshot = fetch_metrics(client, base).await?;
    match validation_from_metrics(&snapshot) {
        Ok(()) => Ok(()),
        Err(failed) => Err(ValidationError::Validation(format!(
            "validation exporters did not report success: {failed}",
        ))),
    }
}

/// shutdown pipeline after running
async fn shutdown_pipeline(client: &Client, base: &str) -> Result<(), ValidationError> {
    let _ = client
        .post(format!("{base}/pipeline-groups/shutdown?wait=true"))
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

    for set in snapshot
        .metric_sets
        .iter()
        .filter(|set| set.name == LOADGEN_METRIC_SET)
    {
        if let Some(label) = attribute_node_id(&set.attributes) {
            let loadgen_signals_produced = metric_value(set, LOADGEN_METRIC_NAME_LOGS).unwrap_or(0)
                + metric_value(set, LOADGEN_METRIC_NAME_METRICS).unwrap_or(0)
                + metric_value(set, LOADGEN_TRACE_NAME_SPANS).unwrap_or(0);
            if &loadgen_signals_produced < expected_per_gen.get(&label).unwrap_or(&0u64) {
                return false;
            }
        }
    }
    true
}

fn validation_from_metrics(snapshot: &MetricsSnapshot) -> Result<(), String> {
    let mut failed_validation_exporters = vec![];

    for set in snapshot
        .metric_sets
        .iter()
        .filter(|set: &&MetricSetSnapshot| set.name == VALIDATION_METRIC_SET)
    {
        // if validation failed detected
        if metric_value(set, VALIDATION_METRIC_NAME).is_some_and(|v| v < 1) {
            // track the node id
            if let Some(label) = attribute_node_id(&set.attributes) {
                failed_validation_exporters.push(label);
            }
        }
    }

    if failed_validation_exporters.is_empty() {
        Ok(())
    } else {
        Err(failed_validation_exporters.join(", "))
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
    fn validation_from_metrics_reports_failed_labels() {
        let snap = MetricsSnapshot {
            timestamp: "t".into(),
            metric_sets: vec![
                set_with_node(VALIDATION_METRIC_SET, VALIDATION_METRIC_NAME, 1, "cap1"),
                set_with_node(VALIDATION_METRIC_SET, VALIDATION_METRIC_NAME, 0, "cap2"),
            ],
        };
        let res = validation_from_metrics(&snap);
        assert_eq!(res, Err("cap2".to_string()));
    }
}
