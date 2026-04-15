// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Programmatic scenario builder that renders a full pipeline group, runs it,
//! waits for readiness, and checks validation metrics.

use crate::container::ContainerConfig;
use crate::error::ValidationError;
use crate::pipeline::{EndpointKind, Pipeline};
use crate::simulate::run_pipelines_with_timeout;
use crate::template::render_jinja;
use crate::traffic::MessageType;
use crate::traffic::{Capture, Generator, TlsConfig};
use minijinja::context;
use portpicker::pick_unused_port;
use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::PathBuf;
use std::time::Duration;

const VALIDATION_TEMPLATE_PATH: &str = "templates/validation_template.yaml.j2";
const DEFAULT_ADMIN_ADDR: &str = "127.0.0.1:8085";
const DEFAULT_READY_MAX_ATTEMPTS: usize = 10;
const DEFAULT_READY_BACKOFF: Duration = Duration::from_secs(3);
const DEFAULT_METRICS_POLL: Duration = Duration::from_secs(2);
const DEFAULT_SCENARIO_RUNTIME: Duration = Duration::from_secs(60);

/// Look up a container by label, validate that `internal_port` is set, and
/// return the host port mapped to that internal port. If no mapping exists
/// yet, a new host port is allocated via `pick_port` and recorded in the
/// container's `mapped_ports`.
fn allocate_container_port(
    containers: &mut HashMap<String, ContainerConfig>,
    container_label: &str,
    internal_port: Option<u16>,
    pick_port: &impl Fn(&str) -> Result<u16, ValidationError>,
    context: &str,
) -> Result<u16, ValidationError> {
    let internal = internal_port.ok_or_else(|| {
        ValidationError::Config(format!(
            "container connection to '{container_label}' missing internal_port"
        ))
    })?;
    let container = containers
        .get_mut(container_label)
        .ok_or_else(|| ValidationError::Config(format!("unknown container: {container_label}")))?;
    if let Some(&host) = container.mapped_ports.get(&internal) {
        Ok(host)
    } else {
        let port = pick_port(context)?;
        let _ = container.mapped_ports.insert(internal, port);
        Ok(port)
    }
}

/// Programmatic scenario builder used by tests.
pub struct Scenario {
    pipeline: Option<Pipeline>,
    generators: HashMap<String, Generator>,
    captures: HashMap<String, Capture>,
    containers: HashMap<String, ContainerConfig>,
    template_path: PathBuf,
    admin_addr: String,
    ready_max_attempts: usize,
    ready_backoff: Duration,
    metrics_poll: Duration,
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
            containers: HashMap::new(),
            template_path: PathBuf::from(VALIDATION_TEMPLATE_PATH),
            admin_addr: DEFAULT_ADMIN_ADDR.to_string(),
            ready_max_attempts: DEFAULT_READY_MAX_ATTEMPTS,
            ready_backoff: DEFAULT_READY_BACKOFF,
            metrics_poll: DEFAULT_METRICS_POLL,
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

    /// Add a Docker container labeled for wiring. The label is used to
    /// reference this container from [`ContainerConnection`](crate::traffic::ContainerConnection)
    /// on generators and captures.
    ///
    /// Containers are started before the pipeline runs and stopped after
    /// it shuts down.
    #[must_use]
    pub fn add_container(mut self, label: impl Into<String>, container: ContainerConfig) -> Self {
        let key = label.into();
        let _ = self.containers.insert(key, container);
        self
    }

    /// Set the total runtime budget (in seconds) for the scenario.
    #[must_use]
    pub fn expect_within(mut self, timeout_secs: u64) -> Self {
        self.runtime = Duration::from_secs(timeout_secs);
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
            for (label, config) in containers {
                running_containers.push(config.start().await.map_err(|e| {
                    ValidationError::Container(format!("container '{label}': {e}"))
                })?);
            }

            let result = run_pipelines_with_timeout(
                rendered_group,
                admin_base,
                generator_signals,
                timeout,
                ready_max_attempts,
                ready_backoff,
                metrics_poll,
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

    /// Render all templates into the final pipeline group YAML.
    fn render_template(&self) -> Result<String, ValidationError> {
        let pipeline = self
            .pipeline
            .as_ref()
            .ok_or_else(|| ValidationError::Config("pipeline missing".into()))?;
        let pipeline_yaml = pipeline.to_yaml_string()?;
        let (suv_core_start, suv_core_end) = (pipeline.core_start, pipeline.core_end);
        let capture_pipeline = self.render_captures()?;
        let generator_pipeline = self.render_generators()?;
        let template = fs::read_to_string(&self.template_path).map_err(|e| {
            ValidationError::Io(format!(
                "failed to read {}: {e}",
                self.template_path.display()
            ))
        })?;

        render_jinja(
            &template,
            context! {
                suv_pipeline => pipeline_yaml,
                admin_bind_address => &self.admin_addr,
                capture_pipeline => capture_pipeline,
                generator_pipeline => generator_pipeline,
                suv_core_start => suv_core_start,
                suv_core_end => suv_core_end,
            },
        )
    }

    /// update the config to wire the connections between the pipelines
    fn update_configs(&mut self) -> Result<(), ValidationError> {
        // Track ports already handed out so that back-to-back
        // `pick_unused_port()` calls (which only probe availability)
        // cannot return the same port twice (TOCTOU race).
        let allocated = RefCell::new(HashSet::<u16>::new());
        let pick_port = |context: &str| -> Result<u16, ValidationError> {
            let mut set = allocated.borrow_mut();
            for _ in 0..64 {
                if let Some(port) = pick_unused_port() {
                    if set.insert(port) {
                        return Ok(port);
                    }
                }
            }
            Err(ValidationError::Config(format!(
                "failed to get unique port for {context}"
            )))
        };

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
        let containers = &mut self.containers;

        // Allocate a receiver port per generator and configure the pipeline.
        // Generators with a container connection get their port allocated and
        // mapped to the container instead of wiring the SUV pipeline.
        for generator in self.generators.values_mut() {
            if let Some(ref mut conn) = generator.container_connection {
                conn.allocated_port = Some(allocate_container_port(
                    containers,
                    &conn.container_label,
                    conn.internal_port,
                    &pick_port,
                    "generator container connection",
                )?);
            } else {
                if generator.suv_exporter_node.is_empty() {
                    return Err(ValidationError::Config(
                        "generator missing suv exporter node name".into(),
                    ));
                }

                let port = pick_port("generator wiring")?;
                generator.suv_port = port;

                let node = generator.suv_exporter_node.clone();
                let endpoint = match generator.suv_exporter_type {
                    MessageType::Otlp => EndpointKind::OtlpGrpcReceiver(node),
                    MessageType::Otap => EndpointKind::OtapGrpcReceiver(node),
                };
                pipeline.apply_endpoint(endpoint, port)?;
            }
        }

        // Allocate an exporter port per capture, configure the pipeline,
        // and wire control paths to the corresponding generators.
        // Captures with a container connection get their port allocated and
        // mapped to the container instead of wiring the SUV pipeline.
        let generators = &mut self.generators;
        for capture in self.captures.values_mut() {
            if let Some(ref mut conn) = capture.container_connection {
                conn.allocated_port = Some(allocate_container_port(
                    containers,
                    &conn.container_label,
                    conn.internal_port,
                    &pick_port,
                    "capture container connection",
                )?);
            } else {
                if capture.suv_receiver_node.is_empty() {
                    return Err(ValidationError::Config(
                        "capture missing suv receiver node name".into(),
                    ));
                }

                let port = pick_port("capture wiring")?;
                capture.suv_port = port;

                let node = capture.suv_receiver_node.clone();
                let endpoint = match capture.suv_receiver_type {
                    MessageType::Otlp => EndpointKind::OtlpGrpcExporter(node),
                    MessageType::Otap => EndpointKind::OtapGrpcExporter(node),
                };
                pipeline.apply_endpoint(endpoint, port)?;
            }

            for gen_label in &capture.control_streams {
                let control_port = pick_port("control wiring")?;
                capture.control_ports.push(control_port);
                generators
                    .get_mut(gen_label.as_str())
                    .ok_or_else(|| {
                        ValidationError::Config(format!("unknown generator: {gen_label}"))
                    })?
                    .control_ports
                    .push(control_port);
            }
        }

        // Wire pipeline nodes that connect directly to containers.
        // Take the connections out of pipeline to avoid simultaneous
        // immutable (iterator) and mutable (set_node_config_value) borrows.
        let pipeline_conns = std::mem::take(&mut pipeline.container_connections);
        for conn in &pipeline_conns {
            let host_port = allocate_container_port(
                containers,
                &conn.container_label,
                conn.internal_port,
                &pick_port,
                "pipeline container connection",
            )?;
            let address = conn.render_address(host_port)?;
            pipeline.set_node_config_value(&conn.node_name, &conn.config_key_path, &address)?;
        }

        // Resolve templated environment variables now that all connection
        // ports are allocated. For any templated env var whose internal_port
        // has no mapping yet, allocate a new host port and add the mapping.
        for (label, container) in containers.iter_mut() {
            if let Some(ref tevs) = container.templated_env_vars {
                for tev in tevs {
                    if let std::collections::hash_map::Entry::Vacant(e) =
                        container.mapped_ports.entry(tev.internal_port)
                    {
                        let port = pick_port(&format!(
                            "templated env var '{}' on container '{label}'",
                            tev.key
                        ))?;
                        let _ = e.insert(port);
                    }
                }
            }
            container.resolve_templated_env_vars()?;
        }

        self.admin_addr = format!("127.0.0.1:{}", pick_port("admin")?);

        Ok(())
    }

    /// Render the capture pipelines.
    fn render_captures(&self) -> Result<String, ValidationError> {
        let template = fs::read_to_string("templates/capture_template.yaml.j2")
            .map_err(|e| ValidationError::Io(format!("failed to read capture template: {e}")))?;
        let mut captures_rendered: Vec<String> = vec![];

        for (label, capture) in self.captures.iter() {
            // Render custom receiver template if this capture has a
            // container connection; otherwise pass an empty string so
            // the main template falls through to the built-in receiver.
            let custom_suv_receiver = match capture.container_connection {
                Some(ref conn) => conn.render()?,
                None => String::new(),
            };

            captures_rendered.push(render_jinja(
                &template,
                context! {
                    suv_receiver_type => &capture.suv_receiver_type,
                    suv_port => capture.suv_port,
                    control_ports => capture.control_ports,
                    validate => &capture.validations_config(),
                    capture_core_start => capture.core_start,
                    capture_core_end => capture.core_end,
                    capture_label => label,
                    custom_suv_receiver => &custom_suv_receiver,
                    idle_timeout_secs => capture.idle_timeout,
                },
            )?);
        }
        Ok(captures_rendered.join("\n"))
    }

    /// Render the generator pipelines.
    fn render_generators(&self) -> Result<String, ValidationError> {
        let template = fs::read_to_string("templates/generator_template.yaml.j2")
            .map_err(|e| ValidationError::Io(format!("failed to read generator template: {e}")))?;
        let mut generators_rendered: Vec<String> = vec![];

        for (label, generator) in self.generators.iter() {
            // Render custom exporter template if this generator has a
            // container connection; otherwise pass an empty string so
            // the main template falls through to the built-in exporter.
            let custom_suv_exporter = match generator.container_connection {
                Some(ref conn) => conn.render()?,
                None => String::new(),
            };

            let tls_enabled = generator.tls.is_some();
            let tls_ca_cert = generator
                .tls
                .as_ref()
                .map(TlsConfig::ca_cert_str)
                .transpose()?
                .unwrap_or("");
            let tls_client_cert = generator
                .tls
                .as_ref()
                .map(TlsConfig::client_cert_str)
                .transpose()?
                .unwrap_or("");
            let tls_client_key = generator
                .tls
                .as_ref()
                .map(TlsConfig::client_key_str)
                .transpose()?
                .unwrap_or("");
            let mtls_enabled = generator.tls.as_ref().is_some_and(TlsConfig::is_mtls);
            let tls_server_name = generator
                .tls
                .as_ref()
                .map_or("localhost", |t| t.server_name.as_str());

            generators_rendered.push(render_jinja(
                &template,
                context! {
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
                    data_source => &generator.data_source,
                    tls_enabled => tls_enabled,
                    tls_ca_cert => tls_ca_cert,
                    tls_client_cert => tls_client_cert,
                    tls_client_key => tls_client_key,
                    mtls_enabled => mtls_enabled,
                    tls_server_name => tls_server_name,
                    custom_suv_exporter => &custom_suv_exporter,
                },
            )?);
        }
        Ok(generators_rendered.join("\n"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pipeline::{Pipeline, PipelineContainerConnection};
    use crate::traffic::ContainerConnection;

    fn sample_yaml() -> &'static str {
        r#"
nodes:
  receiver:
    config:
      protocols:
        grpc:
          listening_addr: "127.0.0.1:4317"
  exporter:
    config:
      grpc_endpoint: "http://default-export"
  otap_recv:
    config:
      listening_addr: "127.0.0.1:4420"
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
        let pipeline = Pipeline::from_yaml(sample_yaml()).unwrap();
        let generator = Generator::logs().otlp_grpc("receiver");
        let capture = Capture::default()
            .otap_grpc("exporter")
            .control_streams(["missing_gen"]);
        let mut scenario = Scenario::new()
            .pipeline(pipeline)
            .add_generator("gen", generator)
            .add_capture("cap", capture);

        let err = scenario
            .update_configs()
            .expect_err("unknown generator label should error");

        assert!(matches!(err, ValidationError::Config(_)));
        assert!(err.to_string().contains("unknown generator: missing_gen"));
    }

    #[test]
    fn render_template_includes_added_generator_and_capture() {
        let pipeline = Pipeline::from_yaml(sample_yaml()).unwrap();
        let generator = Generator::logs().otlp_grpc("receiver");
        let capture = Capture::default()
            .otap_grpc("exporter")
            .control_streams(["gen"]);

        let rendered = Scenario::new()
            .pipeline(pipeline)
            .add_generator("gen", generator)
            .add_capture("cap", capture)
            .render_template()
            .expect("template should render");

        assert!(rendered.contains("gen:"));
        assert!(rendered.contains("cap:"));
    }

    #[test]
    fn add_container_stores_config() {
        let scenario = Scenario::new()
            .add_container(
                "redis",
                ContainerConfig::new("redis", "7.2.4").env("FOO", "bar"),
            )
            .add_container("kafka", ContainerConfig::new("kafka", "3.6"));

        assert_eq!(scenario.containers.len(), 2);
        let redis = scenario.containers.get("redis").expect("redis missing");
        assert_eq!(redis.image, "redis");
        assert_eq!(redis.tag, "7.2.4");
        assert_eq!(redis.env_vars, vec![("FOO".into(), "bar".into())]);
        let kafka = scenario.containers.get("kafka").expect("kafka missing");
        assert_eq!(kafka.image, "kafka");
        assert_eq!(kafka.tag, "3.6");
    }

    #[test]
    fn update_configs_no_generators_errors() {
        let pipeline = Pipeline::from_yaml(sample_yaml()).unwrap();
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
        let pipeline = Pipeline::from_yaml(sample_yaml()).unwrap();
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
    fn expect_within_overrides_runtime() {
        let scenario = Scenario::new().expect_within(42);
        assert_eq!(scenario.runtime, Duration::from_secs(42));
    }

    #[test]
    fn update_configs_wires_pipeline_container_connection() {
        let pipeline = Pipeline::from_yaml(
            r#"
nodes:
  receiver:
    config:
      protocols:
        grpc:
          listening_addr: "127.0.0.1:4317"
  kafka_sink:
    config:
      broker: "placeholder:9092"
      topic: "otlp-logs"
  exporter:
    config:
      grpc_endpoint: "http://default-export"
"#,
        )
        .unwrap()
        .connect_container(
            PipelineContainerConnection::new("kafka")
                .internal_port(9092)
                .node("kafka_sink")
                .config_key("broker")
                .address_template("127.0.0.1:{{ port }}"),
        );

        let mut scenario = Scenario::new()
            .pipeline(pipeline)
            .add_container(
                "kafka",
                ContainerConfig::new("confluentinc/cp-kafka", "7.5.0"),
            )
            .add_generator("gen", Generator::logs().otlp_grpc("receiver"))
            .add_capture(
                "cap",
                Capture::default()
                    .otlp_grpc("exporter")
                    .control_streams(["gen"]),
            );

        scenario
            .update_configs()
            .expect("update_configs should succeed");

        // The kafka container should have a mapped port for internal port 9092.
        let kafka = scenario.containers.get("kafka").unwrap();
        assert_eq!(kafka.mapped_ports.len(), 1);
        let host_port = kafka.mapped_ports[&9092];
        assert_ne!(host_port, 0);

        // The pipeline YAML should have been rewritten with the allocated port.
        let yaml_str = scenario
            .pipeline
            .as_ref()
            .unwrap()
            .to_yaml_string()
            .unwrap();
        let doc: serde_yaml::Value = serde_yaml::from_str(&yaml_str).unwrap();
        let broker = &doc["nodes"]["kafka_sink"]["config"]["broker"];
        assert_eq!(
            broker,
            &serde_yaml::Value::from(format!("127.0.0.1:{host_port}"))
        );
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

    #[test]
    fn update_configs_missing_internal_port_errors() {
        let pipeline = Pipeline::from_yaml(sample_yaml()).unwrap();
        // ContainerConnection without .internal_port() — should fail validation.
        let generator = Generator::logs()
            .to_container(ContainerConnection::new("redis").node_template("type: fake"));
        let mut scenario = Scenario::new()
            .pipeline(pipeline)
            .add_container("redis", ContainerConfig::new("redis", "7.2.4"))
            .add_generator("gen", generator)
            .add_capture(
                "cap",
                Capture::default()
                    .otlp_grpc("exporter")
                    .control_streams(["gen"]),
            );

        let err = scenario
            .update_configs()
            .expect_err("missing internal_port should error");
        assert!(matches!(err, ValidationError::Config(_)));
        assert!(err.to_string().contains("missing internal_port"));
    }

    #[test]
    fn update_configs_resolves_templated_env_vars() {
        let pipeline = Pipeline::from_yaml(
            r#"
nodes:
  receiver:
    config:
      protocols:
        grpc:
          listening_addr: "127.0.0.1:4317"
  kafka_sink:
    config:
      broker: "placeholder:9092"
      topic: "otlp-logs"
  exporter:
    config:
      grpc_endpoint: "http://default-export"
"#,
        )
        .unwrap()
        .connect_container(
            PipelineContainerConnection::new("kafka")
                .internal_port(9092)
                .node("kafka_sink")
                .config_key("broker")
                .address_template("127.0.0.1:{{ port }}"),
        );

        let mut scenario = Scenario::new()
            .pipeline(pipeline)
            .add_container(
                "kafka",
                ContainerConfig::new("confluentinc/cp-kafka", "7.5.0").env_host_port(
                    "KAFKA_ADVERTISED_LISTENERS",
                    "PLAINTEXT://127.0.0.1:{{ host_port }}",
                    9092,
                ),
            )
            .add_generator("gen", Generator::logs().otlp_grpc("receiver"))
            .add_capture(
                "cap",
                Capture::default()
                    .otlp_grpc("exporter")
                    .control_streams(["gen"]),
            );

        scenario
            .update_configs()
            .expect("update_configs should succeed");

        let kafka = scenario.containers.get("kafka").unwrap();

        // The pipeline connection and the templated env var share the same
        // internal port 9092, so they must use the same host port.
        let host_port = kafka.mapped_ports[&9092];
        assert_ne!(host_port, 0);

        // The templated env var should have been resolved and moved to env_vars.
        assert!(kafka.templated_env_vars.is_none());
        assert!(kafka.env_vars.contains(&(
            "KAFKA_ADVERTISED_LISTENERS".into(),
            format!("PLAINTEXT://127.0.0.1:{host_port}")
        )));
    }

    #[test]
    fn update_configs_auto_allocates_for_templated_env_var() {
        let pipeline = Pipeline::from_yaml(sample_yaml()).unwrap();

        let mut scenario = Scenario::new()
            .pipeline(pipeline)
            .add_container(
                "db",
                ContainerConfig::new("postgres", "16").env_host_port(
                    "PG_HOST_PORT",
                    "{{ host_port }}",
                    5432,
                ),
            )
            .add_generator("gen", Generator::logs().otlp_grpc("receiver"))
            .add_capture(
                "cap",
                Capture::default()
                    .otap_grpc("exporter")
                    .control_streams(["gen"]),
            );

        scenario
            .update_configs()
            .expect("update_configs should succeed");

        let db = scenario.containers.get("db").unwrap();

        // The internal port 5432 was not referenced by any connection, so
        // the framework should have auto-allocated a host port.
        assert!(db.mapped_ports.contains_key(&5432));
        let host_port = db.mapped_ports[&5432];
        assert_ne!(host_port, 0);

        // The templated env var should be resolved with the auto-allocated port.
        assert!(db.templated_env_vars.is_none());
        assert!(
            db.env_vars
                .contains(&("PG_HOST_PORT".into(), format!("{host_port}")))
        );
    }
}
