// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

#![allow(dead_code)]

//! Validation test module to validate the encoding/decoding process for otlp messages

use minijinja::{Environment, context};
use otap_df_config::engine::EngineConfig;
use otap_df_controller::Controller;
use otap_df_otap::OTAP_PIPELINE_FACTORY;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::fs;
use std::fs::File;
use std::path::PathBuf;
use std::time::Instant;
use tokio::time::{Duration, sleep};

use crate::error::PipelineError;
use crate::metrics_types::MetricsSnapshot;

const VALIDATION_TEMPLATE_PATH: &str = "templates/validation_template.yaml.j2";
const ADMIN_ENDPOINT: &str = "http://127.0.0.1:8085";
const READY_MAX_ATTEMPTS: usize = 10;
const READY_BACKOFF_SECS: u64 = 3;

const METRICS_POLL_SECS: u64 = 2;
const LOADGEN_MAX_SIGNALS: u64 = 2000;
const LOADGEN_TIMEOUT_SECS: u64 = 70;
const PROPAGATION_DELAY_SECS: u64 = 10;

const PIPELINE_GROUP_ID: &str = "validation_test";
const PIPELINE_ID_TRAFFIC: &str = "traffic_gen";
const PIPELINE_ID_SUV: &str = "suv";
const PIPELINE_ID_VALIDATION: &str = "validate";
pub const PIPELINE_CONFIG_YAML: &str = "pipeline_validation_configs.yaml";

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
    transformative: bool,
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
        let _ = controller.run_till_shutdown(engine_config);
    }
}

impl PipelineValidation {
    /// validates a pipeline, returns true if valid, false if not
    pub async fn validate(&self, pipeline: String) -> Result<bool, PipelineError> {
        let base = ADMIN_ENDPOINT;
        let max_signals_sent = LOADGEN_MAX_SIGNALS;
        let metric_poll_cooldown = METRICS_POLL_SECS;
        let pipeline_delay_sec = PROPAGATION_DELAY_SECS;
        let pipeline_simulator = PipelineSimulator::new(pipeline.as_str())?;

        // start pipeline groups in the background
        let _pipeline_handle = std::thread::spawn(move || {
            pipeline_simulator.run();
        });

        let admin_client = Client::new();

        // wait for ready signal if no ready signal is reached then we error out
        wait_for_ready(&admin_client, base, READY_MAX_ATTEMPTS, READY_BACKOFF_SECS).await?;

        // wait for loadgen to send all msgs
        let loadgen_deadline = Instant::now() + Duration::from_secs(LOADGEN_TIMEOUT_SECS);
        loop {
            let snapshot = fetch_metrics(&admin_client, base).await?;
            if loadgen_reached_limit(&snapshot) {
                break;
            }
            if Instant::now() >= loadgen_deadline {
                return Err(PipelineError::Ready(format!(
                    "load generator did not reach {max_signals_sent} signals before timeout"
                )));
            }
            sleep(Duration::from_secs(metric_poll_cooldown)).await;
        }

        // wait sometime to allow msgs to propogate through the pipelines
        sleep(Duration::from_secs(pipeline_delay_sec)).await;

        // get metrics to check the validation exporter and retrieve the result
        let assert_snapshot = fetch_metrics(&admin_client, base).await?;
        let result = validation_from_metrics(&assert_snapshot);

        // shutdown the pipeline
        let _ = admin_client
            .post(format!("{base}/pipeline-groups/shutdown?wait=true"))
            .send()
            .await
            .map_err(|e| PipelineError::Http(e.to_string()))?
            .error_for_status()
            .map_err(|e| PipelineError::Http(e.to_string()))?;
        Ok(result)
    }

    /// render_template generates the validation pipeline group with the pipeline that will be validated
    pub fn render_template(&mut self, template_path: &str) -> Result<String, PipelineError> {
        // get suv pipeline defintion
        let pipeline_config = fs::read_to_string(&self.pipeline_config_path).map_err(|_| {
            PipelineError::Io(format!(
                "Failed to read in from {}",
                &self.pipeline_config_path.display()
            ))
        })?;
        // get pipeline group template to run validation test
        let template = fs::read_to_string(template_path)
            .map_err(|_| PipelineError::Io(format!("Failed to read in from {template_path}")))?;
        let mut env = Environment::new();
        env.add_template("template", template.as_str())
            .map_err(|e| PipelineError::Template(e.to_string()))?;
        let tmpl = env
            .get_template("template")
            .map_err(|e| PipelineError::Template(e.to_string()))?;
        // pass the context variables
        let ctx = context! {
            loadgen_exporter_type => &self.variables.loadgen_exporter_type,
            backend_receiver_type => &self.variables.backend_receiver_type,
            transformative => self.variables.transformative,
            pipeline_config => pipeline_config,
        };
        let rendered = tmpl
            .render(ctx)
            .map_err(|e| PipelineError::Template(e.to_string()))?;
        Ok(rendered)
    }
}

/// wait_for_ready probes the ready endpoint until it reaches success or used up all retry attempts
async fn wait_for_ready(
    client: &Client,
    base: &str,
    max_retry: usize,
    retry_cooldown: u64,
) -> Result<(), PipelineError> {
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

        sleep(Duration::from_secs(retry_cooldown)).await;
    }

    Err(PipelineError::Ready(
        last_error.unwrap_or_else(|| "readyz timeout".to_string()),
    ))
}

/// fetch_metric fetches metrics from the metric endpoint and returns them as a serialized struct
async fn fetch_metrics(client: &Client, base: &str) -> Result<MetricsSnapshot, PipelineError> {
    client
        .get(format!("{base}/telemetry/metrics"))
        .query(&[("reset", false), ("keep_all_zeroes", false)])
        .send()
        .await
        .map_err(|_| PipelineError::Http(format!("No Response from {base}/telemetry/metrics")))?
        .error_for_status()
        .map_err(|e| PipelineError::Http(e.to_string()))?
        .json()
        .await
        .map_err(|e| PipelineError::Http(e.to_string()))
}

/// loadgen_reached checks the metric for the fake_data_generator metrics
/// and returns true if the max signal count has been reached
fn loadgen_reached_limit(snapshot: &MetricsSnapshot) -> bool {
    // Prefer the fake data generator metrics if present.
    if let Some(v) = snapshot
        .metric_sets
        .iter()
        .find(|set| set.name == "fake_data_generator.receiver.metrics")
        .and_then(|set| {
            set.metrics
                .iter()
                .find(|m| m.name == "logs.produced")
                .map(|m| m.value.to_u64_lossy())
        })
    {
        return v >= LOADGEN_MAX_SIGNALS;
    }
    false
}

/// validation_from_metrics checks the metrics for the validation metrics
/// returns true if validation passed
fn validation_from_metrics(snapshot: &MetricsSnapshot) -> bool {
    if let Some(v) = snapshot
        .metric_sets
        .iter()
        .find(|set| set.name == "validation.exporter.metrics")
        .and_then(|set| {
            set.metrics
                .iter()
                .find(|m| m.name == "valid")
                .map(|m| m.value.to_u64_lossy())
        })
    {
        return v >= 1;
    }
    false
}

/// Execute pipeline validations from the given config file path (or the default).
pub async fn run_validation_tests(config_path: Option<&str>) -> Result<(), PipelineError> {
    let path = config_path.unwrap_or(PIPELINE_CONFIG_YAML);
    let file =
        File::open(path).map_err(|e| PipelineError::Io(format!("failed to open {path}: {e}")))?;
    let config: PipelineValidationConfig = serde_yaml::from_reader(file)
        .map_err(|e| PipelineError::Config(format!("Could not deserialize {path}: {e}")))?;

    println!("========== Running Pipeline Validation ===========");

    let mut report = String::from("========== Pipeline Validation Results ===========\n");
    let mut failures = 0usize;

    // loop through each test config
    for mut config in config.tests {
        // render the pipeline group yaml
        match config.render_template(VALIDATION_TEMPLATE_PATH) {
            // run the validation process on the pipeline group and get the result
            // build a report string
            Ok(rendered_template) => match config.validate(rendered_template).await {
                Ok(result) => {
                    report.push_str(&format!("Pipeline: {} => {}\n", config.name, result));
                    // keep track of failed results
                    if !result {
                        failures += 1;
                    }
                }
                Err(error) => {
                    report.push_str(&format!("Pipeline: {} => ERROR {error}\n", config.name));
                    // keep track of failed results
                    failures += 1;
                }
            },
            Err(error) => {
                report.push_str(&format!("Pipeline: {} => ERROR {error}\n", config.name));
                // keep track of failed results
                failures += 1;
            }
        }
    }

    // print out the report string
    println!("{report}");

    // trigger failed test if we encountered any failed results
    if failures == 0 {
        Ok(())
    } else {
        Err(PipelineError::Validation(format!(
            "{failures} pipeline validation(s) failed"
        )))
    }
}
