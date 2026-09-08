// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Programmatic scenario builder that renders one or more pipeline stages,
//! runs them, waits for readiness, and checks validation metrics.
//!
//! A scenario is a list of ordered stages. The first stage starts the engine;
//! every subsequent stage is reached with the engine's live-update
//! (reconfigure) API instead of restarting the engine. Each stage bundles a
//! system-under-validation pipeline, its traffic generators, and its captures
//! (assertions).
//!
//! The legacy flat builder (`pipeline`/`add_generator`/`add_capture`) is kept
//! as sugar over an implicit first stage, so single-stage scenarios behave
//! exactly as before.

use crate::container::ContainerConfig;
use crate::error::ValidationError;
use crate::pipeline::{EndpointKind, Pipeline};
use crate::simulate::{StagePlan, run_stages_with_timeout};
use crate::stage::Stage;
use crate::template::render_jinja;
use crate::traffic::MessageType;
use crate::traffic::{Capture, Generator, TlsConfig};
use minijinja::context;
use otel_arrow_dfe_config::engine::OtelDataflowSpec;
use otel_arrow_dfe_config::pipeline::PipelineConfig;
use otel_arrow_dfe_test_net::try_pick_unused_loopback_tcp_port;
use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::time::Duration;

const VALIDATION_TEMPLATE: &str = include_str!("../templates/validation_template.yaml.j2");
const CAPTURE_TEMPLATE: &str = include_str!("../templates/capture_template.yaml.j2");
const GENERATOR_TEMPLATE: &str = include_str!("../templates/generator_template.yaml.j2");
const DEFAULT_ADMIN_ADDR: &str = "127.0.0.1:8085";
const DEFAULT_READY_MAX_ATTEMPTS: usize = 10;
const DEFAULT_READY_BACKOFF: Duration = Duration::from_secs(3);
const DEFAULT_METRICS_POLL: Duration = Duration::from_secs(2);
const DEFAULT_SCENARIO_RUNTIME: Duration = Duration::from_secs(60);
const DEFAULT_RECONFIGURE_TIMEOUT_SECS: u64 = 60;
/// Default per-stage budget in seconds. A stage's combined load generation and
/// validation work must complete within this budget; it is the effective cap on
/// how long a stage can run before it is considered stalled.
const DEFAULT_STAGE_TIMEOUT_SECS: u64 = 60;
const MAX_PORT_ALLOCATION_ATTEMPTS: usize = 64;

/// Pipeline group id used for every rendered stage. The SUV pipeline is
/// `suv`; each generator/capture pipeline is named by its label.
pub(crate) const VALIDATION_GROUP_ID: &str = "validation_test";
/// Pipeline id of the system-under-validation pipeline in the rendered group.
pub(crate) const SUV_PIPELINE_ID: &str = "suv";

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
    stages: Vec<(String, Stage)>,
    containers: HashMap<String, ContainerConfig>,
    admin_addr: String,
    ready_max_attempts: usize,
    ready_backoff: Duration,
    metrics_poll: Duration,
    runtime: Duration,
    step_timeout_secs: u64,
    drain_timeout_secs: u64,
    stage_timeout_secs: u64,
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
            stages: Vec::new(),
            containers: HashMap::new(),
            admin_addr: DEFAULT_ADMIN_ADDR.to_string(),
            ready_max_attempts: DEFAULT_READY_MAX_ATTEMPTS,
            ready_backoff: DEFAULT_READY_BACKOFF,
            metrics_poll: DEFAULT_METRICS_POLL,
            runtime: DEFAULT_SCENARIO_RUNTIME,
            step_timeout_secs: DEFAULT_RECONFIGURE_TIMEOUT_SECS,
            drain_timeout_secs: DEFAULT_RECONFIGURE_TIMEOUT_SECS,
            stage_timeout_secs: DEFAULT_STAGE_TIMEOUT_SECS,
        }
    }

    /// Returns a mutable reference to the implicit first stage, creating it if
    /// the scenario has no stages yet. Used by the legacy flat builder.
    fn implicit_stage(&mut self) -> &mut Stage {
        if self.stages.is_empty() {
            self.stages.push((String::from("stage0"), Stage::new()));
        }
        &mut self.stages[0].1
    }

    /// Provide the pipeline under validation for the implicit first stage.
    ///
    /// Legacy single-stage API. For multi-stage scenarios use
    /// [`add_stage`](Self::add_stage).
    #[must_use]
    pub fn pipeline(mut self, pipeline: Pipeline) -> Self {
        self.implicit_stage().pipeline = Some(pipeline);
        self
    }

    /// Add a traffic generator to the implicit first stage.
    ///
    /// Legacy single-stage API. For multi-stage scenarios use
    /// [`add_stage`](Self::add_stage).
    #[must_use]
    pub fn add_generator(mut self, label: impl Into<String>, generator: Generator) -> Self {
        let _ = self
            .implicit_stage()
            .generators
            .insert(label.into(), generator);
        self
    }

    /// Add a capture to the implicit first stage.
    ///
    /// Legacy single-stage API. For multi-stage scenarios use
    /// [`add_stage`](Self::add_stage).
    #[must_use]
    pub fn add_capture(mut self, label: impl Into<String>, capture: Capture) -> Self {
        let _ = self.implicit_stage().captures.insert(label.into(), capture);
        self
    }

    /// Add a stage to the scenario.
    ///
    /// Stages run in the order they are added. The first stage starts the
    /// engine; each subsequent stage is reached with a live-update
    /// (reconfigure) of the `suv` pipeline plus each generator and capture
    /// pipeline. The engine is not restarted between stages.
    #[must_use]
    pub fn add_stage(mut self, label: impl Into<String>, stage: Stage) -> Self {
        self.stages.push((label.into(), stage));
        self
    }

    /// Add a Docker container labeled for wiring. The label is used to
    /// reference this container from [`ContainerConnection`](crate::traffic::ContainerConnection)
    /// on generators and captures.
    ///
    /// Containers are started before the pipeline runs and stopped after
    /// it shuts down. Container connections are only supported in
    /// single-stage scenarios.
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

    /// Configure the per-core step and drain timeouts (in seconds) used for
    /// live-update transitions between stages.
    #[must_use]
    pub fn reconfigure_timeouts(mut self, step_secs: u64, drain_secs: u64) -> Self {
        self.step_timeout_secs = step_secs;
        self.drain_timeout_secs = drain_secs;
        self
    }

    /// Configure the per-stage timeout (in seconds) applied to each stage's
    /// combined load-generation and validation work.
    ///
    /// This is the effective cap on how long a single stage may run before the
    /// scenario fails with an error naming the stage.
    #[must_use]
    pub fn stage_timeout(mut self, timeout_secs: u64) -> Self {
        self.stage_timeout_secs = timeout_secs;
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

        let stage_plans = self.build_stage_plans()?;
        let admin_base = format!("http://{}", self.admin_addr);
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

            let result = run_stages_with_timeout(
                stage_plans,
                admin_base,
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

    /// Wire every stage, render each into a full group YAML, extract per-stage
    /// pipeline configs, and produce the ordered [`StagePlan`] list executed by
    /// [`run_stages_with_timeout`].
    fn build_stage_plans(&mut self) -> Result<Vec<StagePlan>, ValidationError> {
        if self.stages.is_empty() {
            return Err(ValidationError::Config("no stages configured".into()));
        }

        // Global, stable port plan shared across stages. Ports are keyed by the
        // logical wiring endpoint so a pipeline reused across stages keeps the
        // same listening/target address and reconnects cleanly after a
        // reconfigure.
        let allocated = RefCell::new(HashSet::<u16>::new());
        let pick_port = |context: &str| -> Result<u16, ValidationError> {
            let mut set = allocated.borrow_mut();
            let mut io_error_count = 0;
            let mut last_io_error = None;
            for _ in 0..MAX_PORT_ALLOCATION_ATTEMPTS {
                match try_pick_unused_loopback_tcp_port() {
                    Ok(port) => {
                        if set.insert(port) {
                            return Ok(port);
                        }
                    }
                    Err(error) => {
                        io_error_count += 1;
                        last_io_error = Some(error);
                    }
                }
            }
            let detail = last_io_error.map_or_else(
                || "all returned ports were duplicates".to_string(),
                |error| {
                    format!(
                        "{io_error_count} socket allocation attempts failed; \
                         last error: {error}"
                    )
                },
            );
            Err(ValidationError::Config(format!(
                "failed to get unique port for {context} after \
                 {MAX_PORT_ALLOCATION_ATTEMPTS} attempts: {detail}"
            )))
        };

        // Stable ports keyed by generator label, capture label, and control
        // edge (capture_label -> generator_label).
        let mut suv_receiver_ports: HashMap<String, u16> = HashMap::new();
        let mut suv_exporter_ports: HashMap<String, u16> = HashMap::new();
        let mut control_ports: HashMap<(String, String), u16> = HashMap::new();

        // Container connections are only supported in single-stage scenarios.
        let multi_stage = self.stages.len() > 1;

        // Timeouts are copied out before the mutable borrows below so the
        // per-stage plans carry the scenario's configured values.
        let step_timeout_secs = self.step_timeout_secs;
        let drain_timeout_secs = self.drain_timeout_secs;
        let stage_timeout_secs = self.stage_timeout_secs;

        // Admin port is allocated once for the whole engine lifetime.
        self.admin_addr = format!("127.0.0.1:{}", pick_port("admin")?);

        let mut plans = Vec::with_capacity(self.stages.len());
        for (stage_label, stage) in &mut self.stages {
            let plan = Self::build_one_stage_plan(
                stage_label,
                stage,
                &self.admin_addr,
                &mut self.containers,
                &mut suv_receiver_ports,
                &mut suv_exporter_ports,
                &mut control_ports,
                &pick_port,
                multi_stage,
                step_timeout_secs,
                drain_timeout_secs,
                stage_timeout_secs,
            )?;
            plans.push(plan);
        }

        // Resolve templated environment variables now that all connection
        // ports are allocated (single-stage container path only).
        for (label, container) in self.containers.iter_mut() {
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

        Ok(plans)
    }

    /// Wire and render a single stage, returning its executable plan.
    #[allow(clippy::too_many_arguments)]
    fn build_one_stage_plan(
        stage_label: &str,
        stage: &mut Stage,
        admin_addr: &str,
        containers: &mut HashMap<String, ContainerConfig>,
        suv_receiver_ports: &mut HashMap<String, u16>,
        suv_exporter_ports: &mut HashMap<String, u16>,
        control_ports: &mut HashMap<(String, String), u16>,
        pick_port: &impl Fn(&str) -> Result<u16, ValidationError>,
        multi_stage: bool,
        step_timeout_secs: u64,
        drain_timeout_secs: u64,
        stage_timeout_secs: u64,
    ) -> Result<StagePlan, ValidationError> {
        if stage.generators.is_empty() {
            return Err(ValidationError::Config(format!(
                "stage '{stage_label}': no generators configured"
            )));
        }
        if stage.captures.is_empty() {
            return Err(ValidationError::Config(format!(
                "stage '{stage_label}': no captures configured"
            )));
        }

        let pipeline = stage.pipeline.as_mut().ok_or_else(|| {
            ValidationError::Config(format!("stage '{stage_label}': pipeline not provided"))
        })?;

        // Wire generators to the SUV pipeline using stable per-label ports.
        for (gen_label, generator) in stage.generators.iter_mut() {
            if let Some(ref mut conn) = generator.container_connection {
                if multi_stage {
                    return Err(ValidationError::Config(
                        "container connections are not supported in multi-stage scenarios".into(),
                    ));
                }
                conn.allocated_port = Some(allocate_container_port(
                    containers,
                    &conn.container_label,
                    conn.internal_port,
                    pick_port,
                    "generator container connection",
                )?);
            } else {
                if generator.suv_exporter_node.is_empty() {
                    return Err(ValidationError::Config(format!(
                        "stage '{stage_label}': generator '{gen_label}' missing suv exporter node name"
                    )));
                }
                let port = match suv_receiver_ports.get(gen_label) {
                    Some(&p) => p,
                    None => {
                        let p = pick_port("generator wiring")?;
                        let _ = suv_receiver_ports.insert(gen_label.clone(), p);
                        p
                    }
                };
                generator.suv_port = port;
                generator.control_ports.clear();

                let node = generator.suv_exporter_node.clone();
                let endpoint = match generator.suv_exporter_type {
                    MessageType::Otlp => EndpointKind::OtlpGrpcReceiver(node),
                    MessageType::Otap => EndpointKind::OtapGrpcReceiver(node),
                };
                pipeline.apply_endpoint(endpoint, port)?;
            }
        }

        // Wire captures to the SUV pipeline and control paths to generators.
        for (cap_label, capture) in stage.captures.iter_mut() {
            if let Some(ref mut conn) = capture.container_connection {
                if multi_stage {
                    return Err(ValidationError::Config(
                        "container connections are not supported in multi-stage scenarios".into(),
                    ));
                }
                conn.allocated_port = Some(allocate_container_port(
                    containers,
                    &conn.container_label,
                    conn.internal_port,
                    pick_port,
                    "capture container connection",
                )?);
            } else {
                if capture.suv_receiver_node.is_empty() {
                    return Err(ValidationError::Config(format!(
                        "stage '{stage_label}': capture '{cap_label}' missing suv receiver node name"
                    )));
                }
                let port = match suv_exporter_ports.get(cap_label) {
                    Some(&p) => p,
                    None => {
                        let p = pick_port("capture wiring")?;
                        let _ = suv_exporter_ports.insert(cap_label.clone(), p);
                        p
                    }
                };
                capture.suv_port = port;

                let node = capture.suv_receiver_node.clone();
                let endpoint = match capture.suv_receiver_type {
                    MessageType::Otlp => EndpointKind::OtlpGrpcExporter(node),
                    MessageType::Otap => EndpointKind::OtapGrpcExporter(node),
                };
                pipeline.apply_endpoint(endpoint, port)?;
            }

            capture.control_ports.clear();
            for gen_label in capture.control_streams.clone() {
                if !stage.generators.contains_key(gen_label.as_str()) {
                    return Err(ValidationError::Config(format!(
                        "stage '{stage_label}': unknown generator: {gen_label}"
                    )));
                }
                let key = (cap_label.clone(), gen_label.clone());
                let control_port = match control_ports.get(&key) {
                    Some(&p) => p,
                    None => {
                        let p = pick_port("control wiring")?;
                        let _ = control_ports.insert(key, p);
                        p
                    }
                };
                capture.control_ports.push(control_port);
                if let Some(generator) = stage.generators.get_mut(gen_label.as_str()) {
                    generator.control_ports.push(control_port);
                }
            }
        }

        // Wire pipeline nodes that connect directly to containers.
        let pipeline_conns = std::mem::take(&mut pipeline.container_connections);
        if multi_stage && !pipeline_conns.is_empty() {
            return Err(ValidationError::Config(
                "container connections are not supported in multi-stage scenarios".into(),
            ));
        }
        for conn in &pipeline_conns {
            let host_port = allocate_container_port(
                containers,
                &conn.container_label,
                conn.internal_port,
                pick_port,
                "pipeline container connection",
            )?;
            let address = conn.render_address(host_port)?;
            pipeline.set_node_config_value(&conn.node_name, &conn.config_key_path, &address)?;
        }

        // Render the full stage group YAML and extract each pipeline config.
        let rendered = Self::render_stage(stage_label, stage, admin_addr)?;
        let pipeline_configs = Self::extract_pipeline_configs(stage_label, &rendered)?;

        let expected_signals: HashMap<String, u64> = stage
            .generators
            .iter()
            .map(|(label, g)| (label.clone(), g.max_signal_count as u64))
            .collect();
        let capture_labels: Vec<String> = stage.captures.keys().cloned().collect();

        Ok(StagePlan {
            label: stage_label.to_string(),
            rendered_group: rendered,
            pipeline_configs,
            expected_signals,
            capture_labels,
            expected_action: stage.expected_action,
            step_timeout_secs,
            drain_timeout_secs,
            stage_timeout_secs,
        })
    }

    /// Parse a rendered stage group and return each pipeline's config keyed by
    /// pipeline id (`suv` plus every generator/capture label).
    fn extract_pipeline_configs(
        stage_label: &str,
        rendered: &str,
    ) -> Result<HashMap<String, PipelineConfig>, ValidationError> {
        let spec = OtelDataflowSpec::from_yaml(rendered)
            .map_err(|e| ValidationError::Config(format!("stage '{stage_label}': {e}")))?;
        let group = spec.groups.get(VALIDATION_GROUP_ID).ok_or_else(|| {
            ValidationError::Config(format!(
                "stage '{stage_label}': rendered group '{VALIDATION_GROUP_ID}' missing"
            ))
        })?;
        Ok(group
            .pipelines
            .iter()
            .map(|(id, cfg)| (id.as_ref().to_string(), cfg.clone()))
            .collect())
    }

    /// Render one stage into the final pipeline group YAML.
    fn render_stage(
        stage_label: &str,
        stage: &Stage,
        admin_addr: &str,
    ) -> Result<String, ValidationError> {
        let pipeline = stage.pipeline.as_ref().ok_or_else(|| {
            ValidationError::Config(format!("stage '{stage_label}': pipeline missing"))
        })?;
        let pipeline_yaml = pipeline.to_yaml_string()?;
        let (suv_core_start, suv_core_end) = (pipeline.core_start, pipeline.core_end);
        let suv_transport_headers_policy = pipeline.transport_headers_policy_yaml()?;
        let capture_pipeline = Self::render_captures(&stage.captures, stage_label)?;
        let generator_pipeline = Self::render_generators(&stage.generators, stage_label)?;
        render_jinja(
            VALIDATION_TEMPLATE,
            context! {
                suv_pipeline => pipeline_yaml,
                admin_bind_address => admin_addr,
                capture_pipeline => capture_pipeline,
                generator_pipeline => generator_pipeline,
                suv_core_start => suv_core_start,
                suv_core_end => suv_core_end,
                suv_transport_headers_policy => suv_transport_headers_policy,
            },
        )
    }

    /// Render the capture pipelines for a stage.
    ///
    /// `stage_nonce` is stamped into the validation exporter config (a field it
    /// ignores) so that reconfiguring into a new stage always differs from the
    /// prior stage's capture config, forcing a live-update replace and thus a
    /// fresh validation exporter per stage.
    fn render_captures(
        captures: &HashMap<String, Capture>,
        stage_nonce: &str,
    ) -> Result<String, ValidationError> {
        let mut captures_rendered: Vec<String> = vec![];

        for (label, capture) in captures.iter() {
            let custom_suv_receiver = match capture.container_connection {
                Some(ref conn) => conn.render()?,
                None => String::new(),
            };

            captures_rendered.push(render_jinja(
                CAPTURE_TEMPLATE,
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
                    capture_header_keys => &capture.capture_header_keys,
                    stage_nonce => stage_nonce,
                },
            )?);
        }
        Ok(captures_rendered.join("\n"))
    }

    /// Render the generator pipelines for a stage.
    ///
    /// `stage_nonce` is stamped into the traffic generator's
    /// `resource_attributes` so that reconfiguring into a new stage always
    /// differs from the prior stage's generator config, forcing a live-update
    /// replace and thus a fresh traffic run per stage. The attribute is added
    /// identically to both the SUV and control streams, so signal equivalence
    /// is unaffected.
    fn render_generators(
        generators: &HashMap<String, Generator>,
        stage_nonce: &str,
    ) -> Result<String, ValidationError> {
        let mut generators_rendered: Vec<String> = vec![];

        for (label, generator) in generators.iter() {
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

            let transport_headers = if generator.transport_headers.is_empty() {
                None
            } else {
                Some(&generator.transport_headers)
            };

            generators_rendered.push(render_jinja(
                GENERATOR_TEMPLATE,
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
                    transport_headers => transport_headers,
                    stage_nonce => stage_nonce,
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
    use crate::stage::{RolloutAction, Stage};
    use crate::traffic::ContainerConnection;

    fn sample_yaml() -> &'static str {
        r#"
nodes:
  receiver:
    type: "receiver:otlp"
    config:
      protocols:
        grpc:
          listening_addr: "127.0.0.1:4317"
  exporter:
    type: "exporter:otlp_grpc"
    config:
      grpc_endpoint: "http://default-export"
connections:
  - from: receiver
    to: exporter
"#
    }

    /// Scenario: a scenario with no stages is executed.
    /// Guarantees: planning rejects an empty scenario with a config error
    /// rather than silently starting an engine with nothing to validate.
    #[test]
    fn build_stage_plans_requires_stage() {
        let mut scenario = Scenario::new();
        let err = scenario
            .build_stage_plans()
            .expect_err("missing stage should error");
        assert!(matches!(err, ValidationError::Config(_)));
        assert!(err.to_string().contains("no stages configured"));
    }

    /// Scenario: the legacy flat builder is used without any explicit stage.
    /// Guarantees: `pipeline`/`add_generator`/`add_capture` populate exactly
    /// one implicit stage so single-stage scenarios remain fully supported.
    #[test]
    fn flat_builder_creates_single_implicit_stage() {
        let scenario = Scenario::new()
            .pipeline(Pipeline::from_yaml(sample_yaml()).unwrap())
            .add_generator("gen", Generator::logs().otlp_grpc("receiver"))
            .add_capture(
                "cap",
                Capture::default()
                    .otlp_grpc("exporter")
                    .control_streams(["gen"]),
            );
        assert_eq!(scenario.stages.len(), 1);
        let (label, stage) = &scenario.stages[0];
        assert_eq!(label, "stage0");
        assert!(stage.pipeline.is_some());
        assert!(stage.generators.contains_key("gen"));
        assert!(stage.captures.contains_key("cap"));
    }

    /// Scenario: a stage references a control stream generator that does not
    /// exist in that stage.
    /// Guarantees: planning fails with an explicit unknown-generator error.
    #[test]
    fn stage_requires_connected_labels() {
        let mut scenario = Scenario::new().add_stage(
            "s0",
            Stage::new()
                .pipeline(Pipeline::from_yaml(sample_yaml()).unwrap())
                .add_generator("gen", Generator::logs().otlp_grpc("receiver"))
                .add_capture(
                    "cap",
                    Capture::default()
                        .otap_grpc("exporter")
                        .control_streams(["missing_gen"]),
                ),
        );
        let err = scenario
            .build_stage_plans()
            .expect_err("unknown generator label should error");
        assert!(matches!(err, ValidationError::Config(_)));
        assert!(err.to_string().contains("unknown generator: missing_gen"));
    }

    /// Scenario: a two-stage scenario is planned end to end.
    /// Guarantees: each stage renders to a parseable group exposing the `suv`
    /// pipeline plus its generator and capture pipelines, and shared labels
    /// reuse the same stable ports across stages.
    #[test]
    fn two_stage_plan_reuses_ports_and_extracts_configs() {
        let mut scenario = Scenario::new()
            .add_stage(
                "baseline",
                Stage::new()
                    .pipeline(Pipeline::from_yaml(sample_yaml()).unwrap())
                    .add_generator("gen", Generator::logs().otlp_grpc("receiver"))
                    .add_capture(
                        "cap",
                        Capture::default()
                            .otlp_grpc("exporter")
                            .control_streams(["gen"]),
                    ),
            )
            .add_stage(
                "next",
                Stage::new()
                    .pipeline(Pipeline::from_yaml(sample_yaml()).unwrap())
                    .expect_rollout(RolloutAction::Replace)
                    .add_generator(
                        "gen",
                        Generator::logs().fixed_count(10).otlp_grpc("receiver"),
                    )
                    .add_capture(
                        "cap",
                        Capture::default()
                            .otlp_grpc("exporter")
                            .control_streams(["gen"]),
                    ),
            );

        let plans = scenario.build_stage_plans().expect("plan should build");
        assert_eq!(plans.len(), 2);
        for plan in &plans {
            assert!(plan.pipeline_configs.contains_key(SUV_PIPELINE_ID));
            assert!(plan.pipeline_configs.contains_key("gen"));
            assert!(plan.pipeline_configs.contains_key("cap"));
            assert!(plan.capture_labels.contains(&"cap".to_string()));
            assert_eq!(
                plan.expected_signals.get("gen").copied(),
                Some(if plan.label == "next" { 10 } else { 2000 })
            );
        }
        assert_eq!(plans[0].expected_action, None);
        assert_eq!(plans[1].expected_action, Some(RolloutAction::Replace));
    }

    /// Scenario: a two-stage scenario reuses the same generator and capture
    /// labels across both stages.
    /// Guarantees: a shared label is wired to the same stable SUV receiver and
    /// exporter ports in every stage (and not left at the template defaults),
    /// so the live-update transition does not move ports between stages.
    #[test]
    fn shared_labels_reuse_same_ports_across_stages() {
        let mut scenario = Scenario::new()
            .add_stage(
                "s0",
                Stage::new()
                    .pipeline(Pipeline::from_yaml(sample_yaml()).unwrap())
                    .add_generator("gen", Generator::logs().otlp_grpc("receiver"))
                    .add_capture(
                        "cap",
                        Capture::default()
                            .otlp_grpc("exporter")
                            .control_streams(["gen"]),
                    ),
            )
            .add_stage(
                "s1",
                Stage::new()
                    .pipeline(Pipeline::from_yaml(sample_yaml()).unwrap())
                    .add_generator("gen", Generator::logs().otlp_grpc("receiver"))
                    .add_capture(
                        "cap",
                        Capture::default()
                            .otlp_grpc("exporter")
                            .control_streams(["gen"]),
                    ),
            );

        let _ = scenario.build_stage_plans().expect("plan should build");

        // Extract the SUV receiver listening address from each stage's stored
        // pipeline. A shared generator label must resolve to the same host port
        // in both stages.
        let suv_listen_addr = |stage_idx: usize| -> String {
            let yaml = scenario.stages[stage_idx]
                .1
                .pipeline
                .as_ref()
                .unwrap()
                .to_yaml_string()
                .unwrap();
            let doc: serde_yaml::Value = serde_yaml::from_str(&yaml).unwrap();
            doc["nodes"]["receiver"]["config"]["protocols"]["grpc"]["listening_addr"]
                .as_str()
                .unwrap()
                .to_string()
        };

        let addr0 = suv_listen_addr(0);
        let addr1 = suv_listen_addr(1);
        assert_eq!(addr0, addr1, "shared label must keep the same SUV port");
        // The stable port replaced the template default, so wiring actually ran.
        assert_ne!(addr0, "127.0.0.1:4317");
    }

    /// Scenario: a rendered stage group cannot be parsed as a dataflow spec.
    /// Guarantees: config extraction surfaces a stage-scoped config error
    /// instead of panicking on malformed YAML.
    #[test]
    fn extract_pipeline_configs_rejects_unparseable_yaml() {
        let err = Scenario::extract_pipeline_configs("s0", "this: is: not: valid: yaml:")
            .expect_err("unparseable YAML should error");
        assert!(matches!(err, ValidationError::Config(_)));
        assert!(err.to_string().contains("stage 's0'"));
    }

    /// Scenario: a rendered stage group parses but omits the expected
    /// `validation_test` group.
    /// Guarantees: config extraction fails with a stage-scoped "group missing"
    /// error naming the group id it looked for.
    #[test]
    fn extract_pipeline_configs_requires_validation_group() {
        // Parseable dataflow spec whose only group is not VALIDATION_GROUP_ID.
        let rendered = r#"
version: otel_dataflow/v1
groups:
  some_other_group:
    pipelines: {}
"#;
        let err = Scenario::extract_pipeline_configs("s0", rendered)
            .expect_err("missing validation group should error");
        assert!(matches!(err, ValidationError::Config(_)));
        assert!(err.to_string().contains("stage 's0'"));
        assert!(err.to_string().contains(VALIDATION_GROUP_ID));
        assert!(err.to_string().contains("missing"));
    }

    /// Scenario: the same generator and capture configs are rendered under two
    /// different stage nonces.
    /// Guarantees: the stage nonce is stamped into the rendered generator and
    /// capture pipelines, so two stages produce differing configs that force a
    /// live-update replace (and thus fresh instances) on transition.
    #[test]
    fn stage_nonce_changes_rendered_configs() {
        let generators =
            HashMap::from([("gen".to_string(), Generator::logs().otlp_grpc("receiver"))]);
        let captures = HashMap::from([(
            "cap".to_string(),
            Capture::default()
                .otlp_grpc("exporter")
                .control_streams(["gen"]),
        )]);

        let gen_a = Scenario::render_generators(&generators, "stage-a").unwrap();
        let gen_b = Scenario::render_generators(&generators, "stage-b").unwrap();
        assert_ne!(
            gen_a, gen_b,
            "generator render must differ between stage nonces"
        );
        assert!(gen_a.contains("stage-a"));
        assert!(gen_b.contains("stage-b"));

        let cap_a = Scenario::render_captures(&captures, "stage-a").unwrap();
        let cap_b = Scenario::render_captures(&captures, "stage-b").unwrap();
        assert_ne!(
            cap_a, cap_b,
            "capture render must differ between stage nonces"
        );
        assert!(cap_a.contains("stage-a"));
        assert!(cap_b.contains("stage-b"));
    }

    /// Scenario: a stage is missing its generators.
    /// Guarantees: planning fails with a per-stage no-generators error.
    #[test]
    fn stage_without_generators_errors() {
        let mut scenario = Scenario::new().add_stage(
            "s0",
            Stage::new()
                .pipeline(Pipeline::from_yaml(sample_yaml()).unwrap())
                .add_capture("cap", Capture::default().otap_grpc("exporter")),
        );
        let err = scenario
            .build_stage_plans()
            .expect_err("should error without generators");
        assert!(matches!(err, ValidationError::Config(_)));
        assert!(err.to_string().contains("no generators configured"));
    }

    /// Scenario: a stage is missing its captures.
    /// Guarantees: planning fails with a per-stage no-captures error.
    #[test]
    fn stage_without_captures_errors() {
        let mut scenario = Scenario::new().add_stage(
            "s0",
            Stage::new()
                .pipeline(Pipeline::from_yaml(sample_yaml()).unwrap())
                .add_generator("gen", Generator::logs().otlp_grpc("receiver")),
        );
        let err = scenario
            .build_stage_plans()
            .expect_err("should error without captures");
        assert!(matches!(err, ValidationError::Config(_)));
        assert!(err.to_string().contains("no captures configured"));
    }

    /// Scenario: a stage is missing its pipeline.
    /// Guarantees: planning fails with a per-stage pipeline-not-provided error.
    #[test]
    fn stage_without_pipeline_errors() {
        let mut scenario = Scenario::new().add_stage(
            "s0",
            Stage::new()
                .add_generator("gen", Generator::logs().otlp_grpc("receiver"))
                .add_capture("cap", Capture::default().otap_grpc("exporter")),
        );
        let err = scenario
            .build_stage_plans()
            .expect_err("should error without pipeline");
        assert!(matches!(err, ValidationError::Config(_)));
        assert!(err.to_string().contains("pipeline not provided"));
    }

    /// Scenario: a multi-stage scenario declares a container connection.
    /// Guarantees: planning rejects container connections in multi-stage mode
    /// because container wiring is only supported for single-stage runs.
    #[test]
    fn multi_stage_rejects_container_connection() {
        let mut scenario = Scenario::new()
            .add_container("redis", ContainerConfig::new("redis", "7.2.4"))
            .add_stage(
                "s0",
                Stage::new()
                    .pipeline(Pipeline::from_yaml(sample_yaml()).unwrap())
                    .add_generator(
                        "gen",
                        Generator::logs().to_container(
                            ContainerConnection::new("redis")
                                .internal_port(6379)
                                .node_template("type: fake"),
                        ),
                    )
                    .add_capture(
                        "cap",
                        Capture::default()
                            .otlp_grpc("exporter")
                            .control_streams(["gen"]),
                    ),
            )
            .add_stage(
                "s1",
                Stage::new()
                    .pipeline(Pipeline::from_yaml(sample_yaml()).unwrap())
                    .add_generator("gen", Generator::logs().otlp_grpc("receiver"))
                    .add_capture(
                        "cap",
                        Capture::default()
                            .otlp_grpc("exporter")
                            .control_streams(["gen"]),
                    ),
            );
        let err = scenario
            .build_stage_plans()
            .expect_err("container connection in multi-stage should error");
        assert!(matches!(err, ValidationError::Config(_)));
        assert!(
            err.to_string()
                .contains("container connections are not supported in multi-stage")
        );
    }

    /// Scenario: a multi-stage scenario declares a capture container connection.
    /// Guarantees: planning rejects capture container connections in
    /// multi-stage mode, matching the generator and pipeline rejections.
    #[test]
    fn multi_stage_rejects_capture_container_connection() {
        let mut scenario = Scenario::new()
            .add_container("redis", ContainerConfig::new("redis", "7.2.4"))
            .add_stage(
                "s0",
                Stage::new()
                    .pipeline(Pipeline::from_yaml(sample_yaml()).unwrap())
                    .add_generator("gen", Generator::logs().otlp_grpc("receiver"))
                    .add_capture(
                        "cap",
                        Capture::default().from_container(
                            ContainerConnection::new("redis")
                                .internal_port(6379)
                                .node_template("type: fake"),
                        ),
                    ),
            )
            .add_stage(
                "s1",
                Stage::new()
                    .pipeline(Pipeline::from_yaml(sample_yaml()).unwrap())
                    .add_generator("gen", Generator::logs().otlp_grpc("receiver"))
                    .add_capture(
                        "cap",
                        Capture::default()
                            .otlp_grpc("exporter")
                            .control_streams(["gen"]),
                    ),
            );
        let err = scenario
            .build_stage_plans()
            .expect_err("capture container connection in multi-stage should error");
        assert!(matches!(err, ValidationError::Config(_)));
        assert!(
            err.to_string()
                .contains("container connections are not supported in multi-stage")
        );
    }

    /// Scenario: a multi-stage scenario declares a pipeline container
    /// connection.
    /// Guarantees: planning rejects pipeline container connections in
    /// multi-stage mode, matching the generator and capture rejections.
    #[test]
    fn multi_stage_rejects_pipeline_container_connection() {
        let pipeline = Pipeline::from_yaml(kafka_style_yaml())
            .unwrap()
            .connect_container(
                PipelineContainerConnection::new("kafka")
                    .internal_port(9092)
                    .node("exporter")
                    .config_key("grpc_endpoint")
                    .address_template("http://127.0.0.1:{{ port }}"),
            );

        let mut scenario = Scenario::new()
            .add_container(
                "kafka",
                ContainerConfig::new("confluentinc/cp-kafka", "7.5.0"),
            )
            .add_stage(
                "s0",
                Stage::new()
                    .pipeline(pipeline)
                    .add_generator("gen", Generator::logs().otlp_grpc("receiver"))
                    .add_capture(
                        "cap",
                        Capture::default()
                            .otlp_grpc("exporter")
                            .control_streams(["gen"]),
                    ),
            )
            .add_stage(
                "s1",
                Stage::new()
                    .pipeline(Pipeline::from_yaml(kafka_style_yaml()).unwrap())
                    .add_generator("gen", Generator::logs().otlp_grpc("receiver"))
                    .add_capture(
                        "cap",
                        Capture::default()
                            .otlp_grpc("exporter")
                            .control_streams(["gen"]),
                    ),
            );
        let err = scenario
            .build_stage_plans()
            .expect_err("pipeline container connection in multi-stage should error");
        assert!(matches!(err, ValidationError::Config(_)));
        assert!(
            err.to_string()
                .contains("container connections are not supported in multi-stage")
        );
    }

    /// Scenario: a single-stage container connection omits its internal port.
    /// Guarantees: planning fails with a config error that names the container
    /// and the missing `internal_port`, instead of allocating a bogus mapping.
    #[test]
    fn container_connection_missing_internal_port_errors() {
        let mut scenario = Scenario::new()
            .add_container("redis", ContainerConfig::new("redis", "7.2.4"))
            .pipeline(Pipeline::from_yaml(sample_yaml()).unwrap())
            .add_generator(
                "gen",
                Generator::logs()
                    .to_container(ContainerConnection::new("redis").node_template("type: fake")),
            )
            .add_capture(
                "cap",
                Capture::default()
                    .otlp_grpc("exporter")
                    .control_streams(["gen"]),
            );
        let err = scenario
            .build_stage_plans()
            .expect_err("missing internal_port should error");
        assert!(matches!(err, ValidationError::Config(_)));
        assert!(err.to_string().contains("missing internal_port"));
        assert!(err.to_string().contains("redis"));
    }

    /// Scenario: a single-stage container connection references a container
    /// that was never registered with `add_container`.
    /// Guarantees: planning fails with an "unknown container" config error
    /// rather than panicking on a missing map entry.
    #[test]
    fn container_connection_unknown_container_errors() {
        let mut scenario = Scenario::new()
            .pipeline(Pipeline::from_yaml(sample_yaml()).unwrap())
            .add_generator(
                "gen",
                Generator::logs().to_container(
                    ContainerConnection::new("ghost")
                        .internal_port(6379)
                        .node_template("type: fake"),
                ),
            )
            .add_capture(
                "cap",
                Capture::default()
                    .otlp_grpc("exporter")
                    .control_streams(["gen"]),
            );
        let err = scenario
            .build_stage_plans()
            .expect_err("unknown container should error");
        assert!(matches!(err, ValidationError::Config(_)));
        assert!(err.to_string().contains("unknown container: ghost"));
    }

    /// Scenario: a generator has neither a container connection nor an SUV
    /// exporter node configured.
    /// Guarantees: planning fails with a per-stage, per-label error naming the
    /// generator that is missing its SUV exporter node.
    #[test]
    fn generator_missing_suv_exporter_node_errors() {
        // `Generator::logs()` without `.otlp_grpc(...)` leaves the SUV exporter
        // node name empty.
        let mut scenario = Scenario::new()
            .pipeline(Pipeline::from_yaml(sample_yaml()).unwrap())
            .add_generator("gen", Generator::logs())
            .add_capture(
                "cap",
                Capture::default()
                    .otlp_grpc("exporter")
                    .control_streams(["gen"]),
            );
        let err = scenario
            .build_stage_plans()
            .expect_err("generator without suv exporter node should error");
        assert!(matches!(err, ValidationError::Config(_)));
        let msg = err.to_string();
        assert!(msg.contains("generator 'gen'"));
        assert!(msg.contains("missing suv exporter node"));
    }

    /// Scenario: a capture has neither a container connection nor an SUV
    /// receiver node configured.
    /// Guarantees: planning fails with a per-stage, per-label error naming the
    /// capture that is missing its SUV receiver node.
    #[test]
    fn capture_missing_suv_receiver_node_errors() {
        // `Capture::default()` without `.otlp_grpc(...)` leaves the SUV receiver
        // node name empty.
        let mut scenario = Scenario::new()
            .pipeline(Pipeline::from_yaml(sample_yaml()).unwrap())
            .add_generator("gen", Generator::logs().otlp_grpc("receiver"))
            .add_capture("cap", Capture::default().control_streams(["gen"]));
        let err = scenario
            .build_stage_plans()
            .expect_err("capture without suv receiver node should error");
        assert!(matches!(err, ValidationError::Config(_)));
        let msg = err.to_string();
        assert!(msg.contains("capture 'cap'"));
        assert!(msg.contains("missing suv receiver node"));
    }

    /// Scenario: `expect_within` overrides the default runtime budget.
    /// Guarantees: the configured runtime is stored on the scenario.
    #[test]
    fn expect_within_overrides_runtime() {
        let scenario = Scenario::new().expect_within(42);
        assert_eq!(scenario.runtime, Duration::from_secs(42));
    }

    /// Scenario: a scenario configures non-default reconfigure and stage
    /// timeouts, then builds its stage plans.
    /// Guarantees: `reconfigure_timeouts` and `stage_timeout` are propagated
    /// into every StagePlan instead of being silently replaced by the defaults,
    /// so the setters actually take effect during execution.
    #[test]
    fn configured_timeouts_flow_into_stage_plans() {
        let mut scenario = Scenario::new()
            .reconfigure_timeouts(90, 120)
            .stage_timeout(150)
            .add_stage(
                "s0",
                Stage::new()
                    .pipeline(Pipeline::from_yaml(sample_yaml()).unwrap())
                    .add_generator("gen", Generator::logs().otlp_grpc("receiver"))
                    .add_capture(
                        "cap",
                        Capture::default()
                            .otlp_grpc("exporter")
                            .control_streams(["gen"]),
                    ),
            );

        let plans = scenario.build_stage_plans().expect("plan should build");
        assert_eq!(plans.len(), 1);
        assert_eq!(plans[0].step_timeout_secs, 90);
        assert_eq!(plans[0].drain_timeout_secs, 120);
        assert_eq!(plans[0].stage_timeout_secs, 150);
    }

    /// Scenario: a scenario leaves all timeouts at their defaults.
    /// Guarantees: stage plans carry the default reconfigure and stage timeouts
    /// when the setters are not called.
    #[test]
    fn default_timeouts_flow_into_stage_plans() {
        let mut scenario = Scenario::new().add_stage(
            "s0",
            Stage::new()
                .pipeline(Pipeline::from_yaml(sample_yaml()).unwrap())
                .add_generator("gen", Generator::logs().otlp_grpc("receiver"))
                .add_capture(
                    "cap",
                    Capture::default()
                        .otlp_grpc("exporter")
                        .control_streams(["gen"]),
                ),
        );

        let plans = scenario.build_stage_plans().expect("plan should build");
        assert_eq!(plans[0].step_timeout_secs, DEFAULT_RECONFIGURE_TIMEOUT_SECS);
        assert_eq!(
            plans[0].drain_timeout_secs,
            DEFAULT_RECONFIGURE_TIMEOUT_SECS
        );
        assert_eq!(plans[0].stage_timeout_secs, DEFAULT_STAGE_TIMEOUT_SECS);
    }

    fn kafka_style_yaml() -> &'static str {
        r#"
nodes:
  receiver:
    type: "receiver:otlp"
    config:
      protocols:
        grpc:
          listening_addr: "127.0.0.1:4317"
  exporter:
    type: "exporter:otlp_grpc"
    config:
      grpc_endpoint: "http://placeholder:9092"
connections:
  - from: receiver
    to: exporter
"#
    }

    /// Scenario: a single-stage scenario wires a pipeline container connection.
    /// Guarantees: the container gets a mapped host port and the pipeline YAML
    /// is rewritten with the allocated port, preserving legacy behavior.
    #[test]
    fn single_stage_wires_pipeline_container_connection() {
        let pipeline = Pipeline::from_yaml(kafka_style_yaml())
            .unwrap()
            .connect_container(
                PipelineContainerConnection::new("kafka")
                    .internal_port(9092)
                    .node("exporter")
                    .config_key("grpc_endpoint")
                    .address_template("http://127.0.0.1:{{ port }}"),
            );

        let mut scenario = Scenario::new()
            .add_container(
                "kafka",
                ContainerConfig::new("confluentinc/cp-kafka", "7.5.0"),
            )
            .pipeline(pipeline)
            .add_generator("gen", Generator::logs().otlp_grpc("receiver"))
            .add_capture(
                "cap",
                Capture::default()
                    .otlp_grpc("exporter")
                    .control_streams(["gen"]),
            );

        let _ = scenario
            .build_stage_plans()
            .expect("build_stage_plans should succeed");

        let kafka = scenario.containers.get("kafka").unwrap();
        assert_eq!(kafka.mapped_ports.len(), 1);
        let host_port = kafka.mapped_ports[&9092];
        assert_ne!(host_port, 0);

        let yaml_str = scenario.stages[0]
            .1
            .pipeline
            .as_ref()
            .unwrap()
            .to_yaml_string()
            .unwrap();
        let doc: serde_yaml::Value = serde_yaml::from_str(&yaml_str).unwrap();
        let endpoint = &doc["nodes"]["exporter"]["config"]["grpc_endpoint"];
        assert_eq!(
            endpoint,
            &serde_yaml::Value::from(format!("http://127.0.0.1:{host_port}"))
        );
    }

    /// Scenario: a single-stage container declares a templated env var that
    /// shares an internal port with a pipeline connection.
    /// Guarantees: the shared internal port maps to a single host port and the
    /// templated env var is resolved with that host port.
    #[test]
    fn single_stage_resolves_templated_env_vars() {
        let pipeline = Pipeline::from_yaml(kafka_style_yaml())
            .unwrap()
            .connect_container(
                PipelineContainerConnection::new("kafka")
                    .internal_port(9092)
                    .node("exporter")
                    .config_key("grpc_endpoint")
                    .address_template("http://127.0.0.1:{{ port }}"),
            );

        let mut scenario = Scenario::new()
            .add_container(
                "kafka",
                ContainerConfig::new("confluentinc/cp-kafka", "7.5.0").env_host_port(
                    "KAFKA_ADVERTISED_LISTENERS",
                    "PLAINTEXT://127.0.0.1:{{ host_port }}",
                    9092,
                ),
            )
            .pipeline(pipeline)
            .add_generator("gen", Generator::logs().otlp_grpc("receiver"))
            .add_capture(
                "cap",
                Capture::default()
                    .otlp_grpc("exporter")
                    .control_streams(["gen"]),
            );

        let _ = scenario
            .build_stage_plans()
            .expect("build_stage_plans should succeed");

        let kafka = scenario.containers.get("kafka").unwrap();
        let host_port = kafka.mapped_ports[&9092];
        assert_ne!(host_port, 0);
        assert!(kafka.templated_env_vars.is_none());
        assert!(kafka.env_vars.contains(&(
            "KAFKA_ADVERTISED_LISTENERS".into(),
            format!("PLAINTEXT://127.0.0.1:{host_port}")
        )));
    }

    /// Scenario: a single-stage container declares a templated env var whose
    /// internal port is not referenced by any connection.
    /// Guarantees: the framework auto-allocates a host port for that internal
    /// port and resolves the templated env var with it.
    #[test]
    fn single_stage_auto_allocates_for_templated_env_var() {
        let mut scenario = Scenario::new()
            .add_container(
                "db",
                ContainerConfig::new("postgres", "16").env_host_port(
                    "PG_HOST_PORT",
                    "{{ host_port }}",
                    5432,
                ),
            )
            .pipeline(Pipeline::from_yaml(sample_yaml()).unwrap())
            .add_generator("gen", Generator::logs().otlp_grpc("receiver"))
            .add_capture(
                "cap",
                Capture::default()
                    .otap_grpc("exporter")
                    .control_streams(["gen"]),
            );

        let _ = scenario
            .build_stage_plans()
            .expect("build_stage_plans should succeed");

        let db = scenario.containers.get("db").unwrap();
        assert!(db.mapped_ports.contains_key(&5432));
        let host_port = db.mapped_ports[&5432];
        assert_ne!(host_port, 0);
        assert!(db.templated_env_vars.is_none());
        assert!(
            db.env_vars
                .contains(&("PG_HOST_PORT".into(), format!("{host_port}")))
        );
    }

    /// Scenario: `Scenario::default` and `Scenario::new` are compared.
    /// Guarantees: both constructors produce identical baseline settings.
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
