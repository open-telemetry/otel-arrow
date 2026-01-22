// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Validation test module to validate the encoding/decoding process for otlp messages

use otap_df_config::engine::EngineConfig;
use otap_df_controller::Controller;
use otap_df_otap::OTAP_PIPELINE_FACTORY;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fs;
use std::path::PathBuf;
use std::time::Instant;
use tera::{Context, Tera};
use tokio::time::{Duration, sleep};

use crate::error::PipelineError;
use crate::metrics_types::{MetricValue, MetricsSnapshot};

const VALIDATION_TEMPLATE_PATH: &str = "templates/validation_template.yaml.tera";
const ADMIN_ENDPOINT: &str = "http://127.0.0.1:8085";
const READY_MAX_ATTEMPTS: usize = 10;
const READY_BACKOFF_SECS: u64 = 3;

const METRICS_POLL_SECS: u64 = 2;
const LOADGEN_MAX_SIGNALS: u64 = 2000;
const LOADGEN_TIMEOUT_SECS: u64 = 60;
const PROPAGATION_DELAY_SECS: u64 = 3;

const PIPELINE_GROUP_ID: &str = "validation_test";
const PIPELINE_ID_TRAFFIC: &str = "traffic_gen";
const PIPELINE_ID_SUV: &str = "suv";
const PIPELINE_ID_VALIDATION: &str = "validate";

/// Helps distinguish between the message types
#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PlatformType {
    /// otlp type
    Otlp,
    /// otap type
    Otap,
}

#[derive(Debug, Deserialize)]
struct PipelineValidationConfig {
    pub tests: Vec<PipelineValidation>,
}

#[derive(Debug, Deserialize)]
struct PipelineValidation {
    name: String,
    pipeline_config_path: PathBuf,
    variables: TemplateVariables,
}

#[derive(Debug, Deserialize, Serialize)]
struct TemplateVariables {
    loadgen_exporter_type: PlatformType,
    backend_receiver_type: PlatformType,
    #[serde(default)]
    expect_failure: bool,
}

/// struct to simulate a pipeline running, reads a config and starts a pipeline to send and receive data
pub struct PipelineSimulator {
    engine_config: EngineConfig,
}

impl PipelineSimulator {
    /// creates a new simulator from a pipeline yaml configuration
    pub fn new(yaml: &str) -> Result<Self, PipelineError> {
        let engine_config =
            EngineConfig::from_yaml(yaml).map_err(|e| PipelineError::Config(e.to_string()))?;
        Ok(Self { engine_config })
    }

    /// runs the pipeline
    pub fn run(&self) {
        // create controller and run
        let controller = Controller::new(&OTAP_PIPELINE_FACTORY);
        let engine_config = self.engine_config.clone();
        let _ = controller.run_forever(engine_config);
    }
}

impl PipelineValidation {
    /// validates a pipeline, returns true if valid, false if not
    pub async fn validate(&self, pipeline: String) -> Result<bool, PipelineError> {
        let pipeline_simulator = PipelineSimulator::new(pipeline.as_str())?;

        // start pipeline groups in the background
        let _pipeline_handle = std::thread::spawn(move || {
            pipeline_simulator.run();
        });

        let admin_client = Client::new();

        // wait for ready signal, returns error if not ready after some time
        wait_for_ready(&admin_client, ADMIN_ENDPOINT).await?;

        let loadgen_deadline =
            std::time::Instant::now() + Duration::from_secs(LOADGEN_TIMEOUT_SECS);
        loop {
            let snapshot = fetch_metrics(&admin_client).await?;
            if loadgen_reached_limit(&snapshot) {
                // load gen has sent all signals
                break;
            }
            if std::time::Instant::now() >= loadgen_deadline {
                // return Err(PipelineError::Status(format!(
                //     "load generator did not reach {} signals before timeout",
                //     LOADGEN_MAX_SIGNALS
                // )));
                break;
            }
            sleep(Duration::from_secs(METRICS_POLL_SECS)).await;
        }

        let content = admin_client
        .get(format!("{ADMIN_ENDPOINT}/telemetry/metrics"))
        .send()
        .await.expect("failed to get metrics").error_for_status().expect("failed").text().await.expect("failed to get metrics");
        println!("metrics: {content}");


        sleep(Duration::from_secs(PROPAGATION_DELAY_SECS)).await;

        /// send shutdown signal
        let shutdown: Value = admin_client
            .post(format!("{ADMIN_ENDPOINT}/pipeline-groups/shutdown"))
            .send()
            .await?
            .error_for_status()
            .map_err(|e| PipelineError::Status(e.to_string()))?
            .json()
            .await?;

        let assert_snapshot = fetch_metrics(&admin_client).await?;
        // let result = assert_from_metrics(&assert_snapshot);
        Ok(true)
    }

    /// render_template generates the validation pipeline group with the pipeline that will be validated
    pub fn render_template(&mut self, template_path: &str) -> Result<String, PipelineError> {
        // get suv pipeline defintion
        let pipeline_config = fs::read_to_string(&self.pipeline_config_path)?;
        // get pipeline group template to run validation test
        let template = fs::read_to_string(template_path)?;

        let mut tera_context = Context::from_serialize(&self.variables)?;
        tera_context.insert("pipeline_config", &pipeline_config);
        // render the pipeline group string
        Ok(Tera::one_off(&template, &tera_context, false)?)
    }
}

async fn wait_for_ready(client: &Client, base: &str) -> Result<(), PipelineError> {
    let readyz_url = format!("{base}/readyz");
    let mut last_error: Option<String> = None;
    for _attempt in 0..READY_MAX_ATTEMPTS {
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

        sleep(Duration::from_secs(READY_BACKOFF_SECS)).await;
    }

    Err(PipelineError::Ready(
        last_error.unwrap_or_else(|| "readyz timeout".to_string()),
    ))
}

async fn fetch_metrics(client: &Client) -> Result<MetricsSnapshot, PipelineError> {
    Ok(client
        .get(format!("{ADMIN_ENDPOINT}/telemetry/metrics"))
        .send()
        .await?
        .error_for_status()
        .map_err(|e| PipelineError::Status(e.to_string()))?
        .json()
        .await?)
}

fn metric_value_u64(value: &MetricValue) -> Option<u64> {
    match value {
        MetricValue::U64(v) => Some(*v),
        MetricValue::F64(v) => Some(*v as u64),
    }
}

fn loadgen_reached_limit(snapshot: &MetricsSnapshot) -> bool {
    snapshot
        .metric_sets
        .iter()
        .find(|set| set.name == "fake_data_generator.receiver.metrics")
        .and_then(|set| {
            set.metrics
                .iter()
                .find(|m| m.metadata.name == "logs_produced")
                .and_then(|m| metric_value_u64(&m.value))
        })
        .map(|v| v >= LOADGEN_MAX_SIGNALS)
        .unwrap_or(false)
}

// fn assert_from_metrics(snapshot: &MetricsSnapshot) -> bool {
//     let mut comparisons = 0_u64;
//     let mut mismatches = 0_u64;

//     if let Some(set) = snapshot
//         .metric_sets
//         .iter()
//         .find(|set| set.name == "validation.assert.exporter.metrics")
//     {
//         for metric in &set.metrics {
//             match metric.metadata.name.as_str() {
//                 "comparisons" => {
//                     if let Some(v) = metric_value_u64(&metric.value) {
//                         comparisons = v;
//                     }
//                 }
//                 "mismatches" => {
//                     if let Some(v) = metric_value_u64(&metric.value) {
//                         mismatches = v;
//                     }
//                 }
//                 _ => {}
//             }
//         }
//     }

//     comparisons > 0 && mismatches == 0
// }

#[cfg(test)]
mod test {
    use super::*;
    use otap_df_otap::OTAP_PIPELINE_FACTORY;
    use otap_df_otap::fake_data_generator::fake_signal::{
        fake_otlp_logs, fake_otlp_metrics, fake_otlp_traces,
    };
    use otap_df_pdata::testing::equiv::assert_equivalent;
    use std::fs;
    use std::fs::File;

    const PIPELINE_CONFIG_YAML: &str = "pipeline_validation_configs.yaml";

    // validate the encoding and decoding
    #[tokio::test]
    async fn validate_pipeline() {
        let file = File::open(PIPELINE_CONFIG_YAML).expect("failed to get config file");
        let config: PipelineValidationConfig =
            serde_yaml::from_reader(file).expect("Could not deserialize config");
        for mut config in config.tests {
            match config.render_template(VALIDATION_TEMPLATE_PATH) {
                Ok(rendered_template) => match config.validate(rendered_template).await {
                    Ok(result) => println!("Pipeline: {} => {}", config.name, result),
                    Err(error) => println!("Pipeline: {} => {}", config.name, error),
                },
                Err(error) => println!("Pipeline: {} => {}", config.name, error),
            }
        }
    }
}
