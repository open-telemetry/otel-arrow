// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Programmatic scenario builder that renders a full pipeline group, runs it,
//! waits for readiness, and checks validation metrics.

use crate::error::ValidationError;
use crate::pipeline::{EndpointKind, Pipeline};
use crate::simulate::run_pipelines_with_timeout;
use crate::traffic::{Capture, Generator};
use minijinja::{Environment, context};
use portpicker::pick_unused_port;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::time::Duration;

const VALIDATION_TEMPLATE_PATH: &str = "templates/validation_template.yaml.j2";
const DEFAULT_ADMIN_ADDR: &str = "127.0.0.1:8085";
const DEFAULT_READY_MAX_ATTEMPTS: usize = 10;
const DEFAULT_READY_BACKOFF: Duration = Duration::from_secs(3);
const DEFAULT_METRICS_POLL: Duration = Duration::from_secs(2);
const DEFAULT_PROPAGATION_DELAY: Duration = Duration::from_secs(10);
const DEFAULT_SCENARIO_RUNTIME: Duration = Duration::from_secs(140);

/// Programmatic scenario builder used by tests.
pub struct Scenario {
    pipeline: Option<Pipeline>,
    generators: HashMap<String, Generator>,
    captures: HashMap<String, Capture>,
    connections: Vec<(String, String)>,
    template_path: PathBuf,
    admin_addr: String,
    ready_max_attempts: usize,
    ready_backoff: Duration,
    metrics_poll: Duration,
    propagation_delay: Duration,
    runtime: Duration,
}

impl Default for Scenario {
    fn default() -> Self {
        Self::new()
    }
}

impl Scenario {
    /// Start a new scenario builder with sensible defaults.
    #[must_use]
    pub fn new() -> Self {
        Self {
            pipeline: None,
            generators: HashMap::new(),
            captures: HashMap::new(),
            connections: Vec::new(),
            template_path: PathBuf::from(VALIDATION_TEMPLATE_PATH),
            admin_addr: DEFAULT_ADMIN_ADDR.to_string(),
            ready_max_attempts: DEFAULT_READY_MAX_ATTEMPTS,
            ready_backoff: DEFAULT_READY_BACKOFF,
            metrics_poll: DEFAULT_METRICS_POLL,
            propagation_delay: DEFAULT_PROPAGATION_DELAY,
            runtime: DEFAULT_SCENARIO_RUNTIME,
        }
    }

    /// Provide the pipeline under validation.
    #[must_use]
    pub fn pipeline(mut self, pipeline: Pipeline) -> Self {
        self.pipeline = Some(pipeline);
        self
    }

    /// Add a traffic generator labeled for wiring.
    #[must_use]
    pub fn add_generator(mut self, label: impl Into<String>, generator: Generator) -> Self {
        let key = label.into();
        let _ = self.generators.insert(key, generator);
        self
    }

    /// Add a capture labeled for wiring.
    #[must_use]
    pub fn add_capture(mut self, label: impl Into<String>, capture: Capture) -> Self {
        let key = label.into();
        let _ = self.captures.insert(key, capture);
        self
    }

    /// Connect a generator to a capture for control path wiring.
    #[must_use]
    pub fn connect(
        mut self,
        generator_label: impl Into<String>,
        capture_label: impl Into<String>,
    ) -> Self {
        self.connections
            .push((generator_label.into(), capture_label.into()));
        self
    }

    /// Set the total runtime budget for the scenario.
    #[must_use]
    pub fn expect_within(mut self, duration: Duration) -> Self {
        self.runtime = duration;
        self
    }

    /// Execute the scenario.
    pub fn run(mut self) -> Result<(), ValidationError> {
        let ready_max_attempts = self.ready_max_attempts;
        let ready_backoff = self.ready_backoff;
        let metrics_poll = self.metrics_poll;
        let propagation_delay = self.propagation_delay;
        let timeout = self.runtime;

        self.update_configs()?;
        let admin_base = format!("http://{}", self.admin_addr);
        let generator_signals: Vec<u64> = self
            .generators
            .values()
            .map(|g| g.max_signal_count as u64)
            .collect();
        let capture_count = self.captures.len();

        let rendered_group = self.render_template()?;

        println!("{rendered_group}");
        // let tokio_rt = tokio::runtime::Runtime::new()
        //     .map_err(|e| ValidationError::Io(format!("failed to create tokio runtime: {e}")))?;

        // tokio_rt.block_on(async move {
        //     run_pipelines_with_timeout(
        //         rendered_group,
        //         admin_base,
        //         generator_signals,
        //         capture_count,
        //         timeout,
        //         ready_max_attempts,
        //         ready_backoff,
        //         metrics_poll,
        //         propagation_delay,
        //     )
        //     .await
        // })
        Ok(())
    }

    /// convert the template to a finalized yaml string to run
    fn render_template(&self) -> Result<String, ValidationError> {
        let pipeline_yaml = self
            .pipeline
            .as_ref()
            .ok_or_else(|| ValidationError::Config("pipeline missing".into()))?
            .to_yaml_string()?;
        let (suv_core_start, suv_core_end) = self
            .pipeline
            .as_ref()
            .map(|p| (p.core_start, p.core_end))
            .unwrap_or((0, 0));
        let generators = &self.generators;
        let captures = &self.captures;
        let capture_pipeline = self.render_captures(captures)?;
        let generator_pipeline = self.render_generators(generators)?;
        let template = fs::read_to_string(&self.template_path).map_err(|_| {
            ValidationError::Io(format!(
                "Failed to read in from {}",
                self.template_path.display()
            ))
        })?;
        let mut env = Environment::new();
        env.add_template("template", template.as_str())
            .map_err(|e| ValidationError::Template(e.to_string()))?;
        let tmpl = env
            .get_template("template")
            .map_err(|e| ValidationError::Template(e.to_string()))?;
        let ctx = context! {
            suv_pipeline => pipeline_yaml,
            admin_bind_address => &self.admin_addr,
            capture_pipeline => capture_pipeline,
            generator_pipeline => generator_pipeline,
            suv_core_start => suv_core_start,
            suv_core_end => suv_core_end,
        };
        tmpl.render(ctx)
            .map_err(|e| ValidationError::Template(e.to_string()))
    }

    /// update the config to wire the connections between the pipelines
    fn update_configs(&mut self) -> Result<(), ValidationError> {
        let pipeline = self
            .pipeline
            .as_mut()
            .ok_or_else(|| ValidationError::Config("pipeline not provided".into()))?;

        if self.generators.is_empty() {
            return Err(ValidationError::Config("no generators configured".into()));
        }
        if self.captures.is_empty() {
            return Err(ValidationError::Config("no captures configured".into()));
        }

        let mut generators = self.generators.clone();
        let mut captures = self.captures.clone();

        // allocate suv input per generator
        for generator in generators.values_mut() {
            if generator.suv_exporter_node.is_empty() {
                return Err(ValidationError::Config(
                    "generator missing suv exporter node name".into(),
                ));
            }
            let input_port = pick_unused_port().ok_or_else(|| {
                ValidationError::Config("failed to get port for generator".into())
            })?;
            generator.suv_port = input_port;
            let port: u16 = input_port;
            match generator.suv_exporter_type {
                crate::traffic::MessageType::Otlp => pipeline.apply_endpoint(
                    EndpointKind::OtlpGrpcReceiver(generator.suv_exporter_node.clone()),
                    port,
                )?,
                crate::traffic::MessageType::Otap => pipeline.apply_endpoint(
                    EndpointKind::OtapGrpcReceiver(generator.suv_exporter_node.clone()),
                    port,
                )?,
            }
        }

        // allocate suv output per capture
        for capture in captures.values_mut() {
            if capture.suv_receiver_node.is_empty() {
                return Err(ValidationError::Config(
                    "capture missing suv receiver node name".into(),
                ));
            }
            let output_port = pick_unused_port()
                .ok_or_else(|| ValidationError::Config("failed to get port for capture".into()))?;
            capture.suv_port = output_port;
            pipeline.apply_endpoint(
                match capture.suv_receiver_type {
                    crate::traffic::MessageType::Otlp => {
                        EndpointKind::OtlpGrpcExporter(capture.suv_receiver_node.clone())
                    }
                    crate::traffic::MessageType::Otap => {
                        EndpointKind::OtapGrpcExporter(capture.suv_receiver_node.clone())
                    }
                },
                output_port,
            )?;
        }

        // connect control paths
        for (gen_label, cap_label) in &self.connections {
            let generator_cfg = generators
                .get_mut(gen_label)
                .ok_or_else(|| ValidationError::Config(format!("unknown generator {gen_label}")))?;
            let cap = captures
                .get_mut(cap_label)
                .ok_or_else(|| ValidationError::Config(format!("unknown capture {cap_label}")))?;
            let control_port = pick_unused_port()
                .ok_or_else(|| ValidationError::Config("failed to get control port".into()))?;
            generator_cfg.control_ports.push(control_port);
            cap.control_ports.push(control_port);
        }

        let admin_port = pick_unused_port()
            .ok_or_else(|| ValidationError::Config("failed to get new port for admin".into()))?;
        self.admin_addr = format!("127.0.0.1:{admin_port}");
        self.generators = generators;
        self.captures = captures;
        Ok(())
    }

    fn render_captures(
        &self,
        captures: &HashMap<String, Capture>,
    ) -> Result<String, ValidationError> {
        let mut env = Environment::new();
        let template = fs::read_to_string("templates/capture_template.yaml.j2")
            .map_err(|e| ValidationError::Io(format!("failed to read capture template: {e}")))?;
        env.add_template("capture", &template)
            .map_err(|e| ValidationError::Template(e.to_string()))?;
        let tmpl = env
            .get_template("capture")
            .map_err(|e| ValidationError::Template(e.to_string()))?;
        let mut out = String::new();
        let mut labels: Vec<_> = captures.keys().cloned().collect();
        labels.sort();
        for (idx, label) in labels.iter().enumerate() {
            let cap = captures.get(label).expect("label exists");
            let ctx = context! {
                suv_receiver_type => &cap.suv_receiver_type,
                suv_port => cap.suv_port,
                control_ports => cap.control_ports,
                validate => &cap.validations_config(),
                capture_index => idx + 1,
                capture_core_start => cap.core_start,
                capture_core_end => cap.core_end,
            };
            out.push_str(
                &tmpl
                    .render(ctx)
                    .map_err(|e| ValidationError::Template(e.to_string()))?,
            );
            out.push('\n');
        }
        Ok(out)
    }

    fn render_generators(
        &self,
        generators: &HashMap<String, Generator>,
    ) -> Result<String, ValidationError> {
        let mut env = Environment::new();
        let template = fs::read_to_string("templates/generator_template.yaml.j2")
            .map_err(|e| ValidationError::Io(format!("failed to read generator template: {e}")))?;
        env.add_template("generator", &template)
            .map_err(|e| ValidationError::Template(e.to_string()))?;
        let tmpl = env
            .get_template("generator")
            .map_err(|e| ValidationError::Template(e.to_string()))?;
        let mut out = String::new();
        let mut labels: Vec<_> = generators.keys().cloned().collect();
        labels.sort();
        for (idx, label) in labels.iter().enumerate() {
            let generator_cfg = generators.get(label).expect("label exists");
            let ctx = context! {
                suv_exporter_type => &generator_cfg.suv_exporter_type,
                control_ports => generator_cfg.control_ports,
                max_signal_count => generator_cfg.max_signal_count,
                max_batch_size => generator_cfg.max_batch_size,
                signals_per_second => generator_cfg.signals_per_second,
                metric_weight => generator_cfg.metric_weight,
                trace_weight => generator_cfg.trace_weight,
                log_weight => generator_cfg.log_weight,
                suv_port => generator_cfg.suv_port,
                generator_index => idx + 1,
                generator_core_start => generator_cfg.core_start,
                generator_core_end => generator_cfg.core_end,
            };
            out.push_str(
                &tmpl
                    .render(ctx)
                    .map_err(|e| ValidationError::Template(e.to_string()))?,
            );
            out.push('\n');
        }
        Ok(out)
    }
}
