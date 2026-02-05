// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Validation test module to validate pipelines for otap/otlp messages
#![cfg_attr(not(test), allow(dead_code))]

use crate::traffic::{Capture, Generator};
use minijinja::{Environment, context};
use otap_df_config::engine::EngineConfig;
use otap_df_controller::Controller;
use otap_df_otap::OTAP_PIPELINE_FACTORY;
use reqwest::Client;
use serde::Deserialize;
use std::fs;
use std::path::PathBuf;
use std::time::Instant;
use tokio::time::{Duration, sleep};

use crate::error::ValidationError;
use crate::metrics_types::MetricsSnapshot;

const VALIDATION_TEMPLATE_PATH: &str = "templates/validation_template.yaml.j2";
const ADMIN_ENDPOINT: &str = "http://127.0.0.1:8085";
const READY_MAX_ATTEMPTS: usize = 10;
const READY_BACKOFF_SECS: u64 = 3;

const METRICS_POLL_SECS: u64 = 2;
const LOADGEN_TIMEOUT_SECS: u64 = 70;
const PROPAGATION_DELAY_SECS: u64 = 10;

/// Default location of the YAML file that lists pipeline validation scenarios.
pub const PIPELINE_CONFIG_YAML: &str = "pipeline_validation_scenarios.yaml";

#[derive(Debug, Deserialize)]
struct ValidationScenarios {
    pub scenarios: Vec<Scenario>,
}

#[derive(Debug, Deserialize)]
struct Scenario {
    name: String,
    scenario_config_path: PathBuf,
    traffic_generation_config: Generator,
    traffic_capture_config: Capture,
}

/// struct to simulate a pipeline running, reads a config and starts a pipeline to send and receive data
pub struct PipelineSimulator {
    engine_config: EngineConfig,
}

impl PipelineSimulator {
    /// creates a new simulator from a pipeline yaml configuration
    pub fn new(yaml: &str) -> Result<Self, ValidationError> {
        let engine_config =
            EngineConfig::from_yaml(yaml).map_err(|e| ValidationError::Config(e.to_string()))?;
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

impl Scenario {
    /// validates a pipeline, returns true if valid, false if not
    pub async fn validate(&self, pipeline: String) -> Result<bool, ValidationError> {
        let base = ADMIN_ENDPOINT;
        let max_signals_sent = self.traffic_generation_config.max_signal_count as u64;
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
            if loadgen_reached_limit(&snapshot, max_signals_sent) {
                break;
            }
            if Instant::now() >= loadgen_deadline {
                return Err(ValidationError::Ready(format!(
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
            .map_err(|e| ValidationError::Http(e.to_string()))?
            .error_for_status()
            .map_err(|e| ValidationError::Http(e.to_string()))?;
        Ok(result)
    }

    /// render_template generates the validation pipeline group with the pipeline that will be validated
    pub fn render_template(&mut self, template_path: &str) -> Result<String, ValidationError> {
        // get suv pipeline defintion
        let pipeline_config = fs::read_to_string(&self.scenario_config_path).map_err(|_| {
            ValidationError::Io(format!(
                "Failed to read in from {}",
                &self.scenario_config_path.display()
            ))
        })?;
        // get pipeline group template to run validation test
        let template = fs::read_to_string(template_path)
            .map_err(|_| ValidationError::Io(format!("Failed to read in from {template_path}")))?;
        let mut env = Environment::new();
        env.add_template("template", template.as_str())
            .map_err(|e| ValidationError::Template(e.to_string()))?;
        let tmpl = env
            .get_template("template")
            .map_err(|e| ValidationError::Template(e.to_string()))?;
        // pass the context variables
        let traffic_gen = &self.traffic_generation_config;
        let traffic_cap = &self.traffic_capture_config;
        let ctx = context! {
            suv_receiver_type => &traffic_cap.suv_receiver_type,
            suv_exporter_type => &traffic_gen.suv_exporter_type,
            control_receiver_type => &traffic_cap.control_receiver_type,
            control_exporter_type => &traffic_gen.control_exporter_type,
            expect_failure => traffic_cap.transformative,
            max_signal_count => traffic_gen.max_signal_count,
            max_batch_size => traffic_gen.max_batch_size,
            signals_per_second => traffic_gen.signals_per_second,
            suv_addr => &traffic_cap.suv_listening_addr,
            suv_endpoint => &traffic_gen.suv_endpoint,
            control_addr => &traffic_cap.control_listening_addr,
            control_endpoint => &traffic_gen.control_endpoint,
            pipeline_config => pipeline_config,
        };
        let rendered = tmpl
            .render(ctx)
            .map_err(|e| ValidationError::Template(e.to_string()))?;
        Ok(rendered)
    }
}

/// wait_for_ready probes the ready endpoint until it reaches success or used up all retry attempts
async fn wait_for_ready(
    client: &Client,
    base: &str,
    max_retry: usize,
    retry_cooldown: u64,
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

        sleep(Duration::from_secs(retry_cooldown)).await;
    }

    Err(ValidationError::Ready(
        last_error.unwrap_or_else(|| "readyz timeout".to_string()),
    ))
}

/// fetch_metric fetches metrics from the metric endpoint and returns them as a serialized struct
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

/// loadgen_reached checks the metric for the fake_data_generator metrics
/// and returns true if the max signal count has been reached
fn loadgen_reached_limit(snapshot: &MetricsSnapshot, max_signals: u64) -> bool {
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
        return v >= max_signals;
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

#[cfg(test)]
use std::fs::File;

const REPORT_PATH: &str = "target/pipeline_validation_report.txt";

// ignore as we run this seperately in a different job
#[ignore] 
#[tokio::test]
async fn run_validation_scenarios() {
    let file = File::open(PIPELINE_CONFIG_YAML).expect("failed to open config file");
    let config: ValidationScenarios =
        serde_yaml::from_reader(file).expect("failed to serialize config from file");

    println!("========== Running Pipeline Validation ===========");

    let mut report = String::from("========== Pipeline Validation Results ===========\n");
    let mut failures: usize = 0;

    // loop through each test config
    for mut config in config.scenarios {
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

    // persist report for CI to surface
    let report_path = PathBuf::from(REPORT_PATH);
    if let Some(parent) = report_path.parent() {
        fs::create_dir_all(parent).expect("failed to create report directory");
    }
    fs::write(&report_path, &report).expect("failed to write validation report");

    // trigger failed test if we encountered any failed results
    assert_eq!(failures, 0);
}
