// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Programmatic scenario builder that renders a full pipeline group, runs it,
//! waits for readiness, and checks validation metrics.

use crate::container::ContainerConfig;
use crate::error::ValidationError;
use crate::pipeline::{EndpointKind, Pipeline};
use crate::simulate::run_pipelines_with_timeout;
use crate::traffic::MessageType;
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
const DEFAULT_PROPAGATION_DELAY: Duration = Duration::from_secs(20);
const DEFAULT_SCENARIO_RUNTIME: Duration = Duration::from_secs(140);

/// Programmatic scenario builder used by tests.
pub struct Scenario {
    pipeline: Option<Pipeline>,
    generators: HashMap<String, Generator>,
    captures: HashMap<String, Capture>,
    connections: Vec<(String, String)>,
    containers: Vec<ContainerConfig>,
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
            containers: Vec::new(),
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

    /// Add a Docker container that will be started before the pipeline runs
    /// and stopped after it shuts down.
    #[must_use]
    pub fn add_container(mut self, container: ContainerConfig) -> Self {
        self.containers.push(container);
        self
    }

    /// Set the total runtime budget for the scenario.
    #[must_use]
    pub fn expect_within(mut self, duration: Duration) -> Self {
        self.runtime = duration;
        self
    }

    /// Execute the scenario.
    ///
    /// When containers are configured (via [`add_container`](Self::add_container)),
    /// they are started before the pipeline group runs. After the pipeline
    /// shuts down, the containers are stopped.
    pub fn run(mut self) -> Result<(), ValidationError> {
        let ready_max_attempts = self.ready_max_attempts;
        let ready_backoff = self.ready_backoff;
        let metrics_poll = self.metrics_poll;
        let propagation_delay = self.propagation_delay;
        let timeout = self.runtime;

        self.update_configs()?;
        let admin_base = format!("http://{}", self.admin_addr);
        let generator_signals: HashMap<String, u64> = self
            .generators
            .iter()
            .map(|(label, g)| (label.clone(), g.max_signal_count as u64))
            .collect();

        let rendered_group = self.render_template()?;
        let containers = self.containers;

        let tokio_rt = tokio::runtime::Runtime::new()
            .map_err(|e| ValidationError::Io(format!("failed to create tokio runtime: {e}")))?;

        tokio_rt.block_on(async move {
            let mut running_containers = Vec::new();
            for config in containers {
                running_containers.push(config.start().await?);
            }

            let result = run_pipelines_with_timeout(
                rendered_group,
                admin_base,
                generator_signals,
                timeout,
                ready_max_attempts,
                ready_backoff,
                metrics_poll,
                propagation_delay,
            )
            .await;

            for container in running_containers {
                container.stop().await.map_err(|e| {
                    ValidationError::Container(format!("failed to stop container: {e}"))
                })?;
            }

            result
        })
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
        // Collect ports reserved by test containers so we never hand them out.
        let reserved_ports: std::collections::HashSet<u16> = self
            .containers
            .iter()
            .flat_map(|c| c.exposed_tcp_ports.iter().copied())
            .collect();

        // helper to get port and return error if no ports are found.
        // Retries if portpicker returns a port reserved by a container.
        let pick_port = |context: &str| -> Result<u16, ValidationError> {
            for _ in 0..100 {
                let port = pick_unused_port().ok_or_else(|| {
                    ValidationError::Config(format!("failed to get port for {context}"))
                })?;
                if !reserved_ports.contains(&port) {
                    return Ok(port);
                }
            }
            Err(ValidationError::Config(format!(
                "failed to get port for {context}: all picked ports conflict with container ports"
            )))
        };

        // helper to check if generators/captures are missing fields
        fn require_non_empty(s: &str, context: &str) -> Result<(), ValidationError> {
            if s.is_empty() {
                return Err(ValidationError::Config(format!("{context} missing")));
            }
            Ok(())
        }

        if self.generators.is_empty() {
            return Err(ValidationError::Config("no generators configured".into()));
        }
        if self.captures.is_empty() {
            return Err(ValidationError::Config("no captures configured".into()));
        }

        let pipeline = self
            .pipeline
            .as_mut()
            .ok_or_else(|| ValidationError::Config("pipeline not provided".into()))?;

        // Allocate a receiver port per generator and configure the pipeline.
        for generator in self.generators.values_mut() {
            require_non_empty(
                &generator.suv_exporter_node,
                "generator missing suv exporter node name",
            )?;

            let port = pick_port("generator wiring")?;
            generator.suv_port = port;

            let node = generator.suv_exporter_node.clone();
            let endpoint = match generator.suv_exporter_type {
                MessageType::Otlp => EndpointKind::OtlpGrpcReceiver(node),
                MessageType::Otap => EndpointKind::OtapGrpcReceiver(node),
            };
            pipeline.apply_endpoint(endpoint, port)?;
        }

        // Allocate an exporter port per capture and configure the pipeline.
        for capture in self.captures.values_mut() {
            require_non_empty(
                &capture.suv_receiver_node,
                "capture missing suv receiver node name",
            )?;

            let port = pick_port("capture wiring")?;
            capture.suv_port = port;

            let node = capture.suv_receiver_node.clone();
            let endpoint = match capture.suv_receiver_type {
                MessageType::Otlp => EndpointKind::OtlpGrpcExporter(node),
                MessageType::Otap => EndpointKind::OtapGrpcExporter(node),
            };
            pipeline.apply_endpoint(endpoint, port)?;
        }

        // Connect control paths between each generator–capture pair.
        for (gen_label, cap_label) in &self.connections {
            let control_port = pick_port("control wiring")?;

            self.generators
                .get_mut(gen_label)
                .ok_or_else(|| ValidationError::Config(format!("unknown generator {gen_label}")))?
                .control_ports
                .push(control_port);

            self.captures
                .get_mut(cap_label)
                .ok_or_else(|| ValidationError::Config(format!("unknown capture {cap_label}")))?
                .control_ports
                .push(control_port);
        }

        self.admin_addr = format!("127.0.0.1:{}", pick_port("admin")?);

        Ok(())
    }

    /// render the capture pipelines
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
        let mut captures_rendered: Vec<String> = vec![];

        for (label, capture) in captures.iter() {
            let ctx = context! {
                suv_receiver_type => &capture.suv_receiver_type,
                suv_port => capture.suv_port,
                control_ports => capture.control_ports,
                validate => &capture.validations_config(),
                capture_core_start => capture.core_start,
                capture_core_end => capture.core_end,
                capture_label => label,
            };
            captures_rendered.push(
                tmpl.render(ctx)
                    .map_err(|e| ValidationError::Template(e.to_string()))?,
            );
        }
        Ok(captures_rendered.join("\n"))
    }

    /// render the generator pipelines
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
        let mut generators_rendered: Vec<String> = vec![];

        for (label, generator) in generators.iter() {
            let ctx = context! {
                suv_exporter_type => &generator.suv_exporter_type,
                control_ports => generator.control_ports,
                max_signal_count => generator.max_signal_count,
                max_batch_size => generator.max_batch_size,
                signals_per_second => generator.signals_per_second,
                metric_weight => generator.metric_weight,
                trace_weight => generator.trace_weight,
                log_weight => generator.log_weight,
                suv_port => generator.suv_port,
                generator_core_start => generator.core_start,
                generator_core_end => generator.core_end,
                generator_label => label,
                data_source => &generator.data_source
            };
            generators_rendered.push(
                tmpl.render(ctx)
                    .map_err(|e| ValidationError::Template(e.to_string()))?,
            );
        }
        Ok(generators_rendered.join("\n"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pipeline::Pipeline;

    fn sample_yaml() -> &'static str {
        r#"
nodes:
  receiver:
    config:
      protocols:
        grpc:
          listening_addr: "0.0.0.0:4317"
  exporter:
    config:
      grpc_endpoint: "http://default-export"
  otap_recv:
    config:
      listening_addr: "0.0.0.0:4420"
  otap_exp:
    config:
      grpc_endpoint: "http://default-otap-export"
"#
    }

    #[test]
    fn render_template_requires_pipeline() {
        let scenario = Scenario::new();
        let err = scenario
            .render_template()
            .expect_err("missing pipeline should error");
        assert!(matches!(err, ValidationError::Config(_)));
        assert!(err.to_string().contains("pipeline missing"));
    }

    #[test]
    fn render_template_requires_connected_labels() {
        let pipeline = Pipeline::from_yaml(sample_yaml());
        let generator = Generator::logs().otlp_grpc("receiver");
        let capture = Capture::default().otap_grpc("exporter");
        let mut scenario = Scenario::new()
            .pipeline(pipeline)
            .add_generator("gen", generator)
            .add_capture("cap", capture)
            .connect("missing_gen", "cap");

        let err = scenario
            .update_configs()
            .expect_err("unknown generator label should error");

        assert!(matches!(err, ValidationError::Config(_)));
        assert!(err.to_string().contains("unknown generator missing_gen"));
    }

    #[test]
    fn render_template_includes_added_generator_and_capture() {
        let pipeline = Pipeline::from_yaml(sample_yaml());
        let generator = Generator::logs().otlp_grpc("receiver");
        let capture = Capture::default().otap_grpc("exporter");

        let rendered = Scenario::new()
            .pipeline(pipeline)
            .add_generator("gen", generator)
            .add_capture("cap", capture)
            .connect("gen", "cap")
            .render_template()
            .expect("template should render");

        assert!(rendered.contains("gen:"));
        assert!(rendered.contains("cap:"));
    }

    #[test]
    fn add_container_stores_config() {
        let scenario = Scenario::new()
            .add_container(
                ContainerConfig::new("redis", "7.2.4")
                    .expose_tcp(6379)
                    .env("FOO", "bar"),
            )
            .add_container(ContainerConfig::new("kafka", "3.6"));

        assert_eq!(scenario.containers.len(), 2);
        assert_eq!(scenario.containers[0].image, "redis");
        assert_eq!(scenario.containers[0].tag, "7.2.4");
        assert_eq!(scenario.containers[0].exposed_tcp_ports, vec![6379]);
        assert_eq!(scenario.containers[1].image, "kafka");
        assert_eq!(scenario.containers[1].tag, "3.6");
    }

    #[test]
    fn update_configs_no_generators_errors() {
        let pipeline = Pipeline::from_yaml(sample_yaml());
        let mut scenario = Scenario::new()
            .pipeline(pipeline)
            .add_capture("cap", Capture::default().otap_grpc("exporter"));

        let err = scenario
            .update_configs()
            .expect_err("should error without generators");
        assert!(matches!(err, ValidationError::Config(_)));
        assert!(err.to_string().contains("no generators configured"));
    }

    #[test]
    fn update_configs_no_captures_errors() {
        let pipeline = Pipeline::from_yaml(sample_yaml());
        let mut scenario = Scenario::new()
            .pipeline(pipeline)
            .add_generator("gen", Generator::logs().otlp_grpc("receiver"));

        let err = scenario
            .update_configs()
            .expect_err("should error without captures");
        assert!(matches!(err, ValidationError::Config(_)));
        assert!(err.to_string().contains("no captures configured"));
    }

    #[test]
    fn update_configs_no_pipeline_errors() {
        let mut scenario = Scenario::new()
            .add_generator("gen", Generator::logs().otlp_grpc("receiver"))
            .add_capture("cap", Capture::default().otap_grpc("exporter"));

        let err = scenario
            .update_configs()
            .expect_err("should error without pipeline");
        assert!(matches!(err, ValidationError::Config(_)));
        assert!(err.to_string().contains("pipeline not provided"));
    }

    #[test]
    fn unknown_capture_label_errors() {
        let pipeline = Pipeline::from_yaml(sample_yaml());
        let mut scenario = Scenario::new()
            .pipeline(pipeline)
            .add_generator("gen", Generator::logs().otlp_grpc("receiver"))
            .add_capture("cap", Capture::default().otap_grpc("exporter"))
            .connect("gen", "missing_cap");

        let err = scenario
            .update_configs()
            .expect_err("unknown capture label should error");
        assert!(matches!(err, ValidationError::Config(_)));
        assert!(err.to_string().contains("unknown capture missing_cap"));
    }

    #[test]
    fn expect_within_overrides_runtime() {
        let scenario = Scenario::new().expect_within(Duration::from_secs(42));
        assert_eq!(scenario.runtime, Duration::from_secs(42));
    }

    #[test]
    fn update_configs_avoids_container_ports() {
        let pipeline = Pipeline::from_yaml(sample_yaml());
        let mut scenario = Scenario::new()
            .pipeline(pipeline)
            .add_generator("gen", Generator::logs().otlp_grpc("receiver"))
            .add_capture("cap", Capture::default().otap_grpc("exporter"))
            .connect("gen", "cap")
            .add_container(ContainerConfig::new("redis", "7.2.4").expose_tcp(4317));

        scenario.update_configs().expect("should succeed");

        // Verify no allocated port matches the container's exposed port.
        let generator = scenario.generators.get("gen").unwrap();
        assert_ne!(generator.suv_port, 4317);

        let capture = scenario.captures.get("cap").unwrap();
        assert_ne!(capture.suv_port, 4317);
    }

    #[test]
    fn default_matches_new() {
        let from_new = Scenario::new();
        let from_default = Scenario::default();
        assert_eq!(from_new.runtime, from_default.runtime);
        assert_eq!(from_new.ready_max_attempts, from_default.ready_max_attempts);
        assert_eq!(from_new.ready_backoff, from_default.ready_backoff);
        assert!(from_new.containers.is_empty());
        assert!(from_default.containers.is_empty());
    }
}
