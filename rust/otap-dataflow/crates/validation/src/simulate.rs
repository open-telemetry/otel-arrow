// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use crate::error::ValidationError;
use crate::metrics_types::MetricsSnapshot;
use otap_df_config::engine::EngineConfig;
use otap_df_controller::Controller;
use otap_df_otap::OTAP_PIPELINE_FACTORY;
use reqwest::Client;
use tokio::time::{Duration, sleep};

const LOADGEN_METRIC_SET: &str = "fake_data_generator.receiver.metrics";
const LOADGEN_METRIC_NAME: &str = "logs.produced";
const VALIDATION_METRIC_SET: &str = "validation.exporter.metrics";
const VALIDATION_METRIC_NAME: &str = "valid";

pub(crate) async fn run_pipelines_with_timeout(
    rendered_group: String,
    admin_base: String,
    max_signals_sent: u64,
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
        wait_for_loadgen(&admin_client, &admin_base, max_signals_sent, metrics_poll).await?;
        sleep(propagation_delay).await;
        let result = ensure_validation_passed(&admin_client, &admin_base).await;
        shutdown_pipeline(&admin_client, &admin_base).await?;
        result
    })
    .await
    .map_err(|_| ValidationError::Validation(format!("scenario timed out after {timeout:?}")))?
}

struct PipelineSimulator {
    engine_config: EngineConfig,
}

impl PipelineSimulator {
    fn new(yaml: &str) -> Result<Self, ValidationError> {
        let engine_config =
            EngineConfig::from_yaml(yaml).map_err(|e| ValidationError::Config(e.to_string()))?;
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
        .query(&[("reset", false), ("keep_all_zeroes", false)])
        .send()
        .await
        .map_err(|_| ValidationError::Http(format!("No Response from {base}/telemetry/metrics")))?
        .error_for_status()
        .map_err(|e| ValidationError::Http(e.to_string()))?
        .json()
        .await
        .map_err(|e| ValidationError::Http(e.to_string()))
}

async fn wait_for_loadgen(
    client: &Client,
    base: &str,
    max_signals_sent: u64,
    metrics_poll: Duration,
) -> Result<(), ValidationError> {
    loop {
        let snapshot = fetch_metrics(client, base).await?;
        if loadgen_reached_limit(&snapshot, max_signals_sent) {
            return Ok(());
        }
        sleep(metrics_poll).await;
    }
}

async fn ensure_validation_passed(client: &Client, base: &str) -> Result<(), ValidationError> {
    let snapshot = fetch_metrics(client, base).await?;
    if validation_from_metrics(&snapshot) {
        Ok(())
    } else {
        Err(ValidationError::Validation(
            "validation metrics did not report success".into(),
        ))
    }
}

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

fn metric_value(snapshot: &MetricsSnapshot, set_name: &str, metric_name: &str) -> Option<u64> {
    snapshot
        .metric_sets
        .iter()
        .find(|set| set.name == set_name)
        .and_then(|set| {
            set.metrics
                .iter()
                .find(|m| m.name == metric_name)
                .map(|m| m.value.to_u64_lossy())
        })
}

fn loadgen_reached_limit(snapshot: &MetricsSnapshot, max_signals: u64) -> bool {
    metric_value(snapshot, LOADGEN_METRIC_SET, LOADGEN_METRIC_NAME)
        .is_some_and(|v| v >= max_signals)
}

fn validation_from_metrics(snapshot: &MetricsSnapshot) -> bool {
    metric_value(snapshot, VALIDATION_METRIC_SET, VALIDATION_METRIC_NAME).is_some_and(|v| v >= 1)
}
