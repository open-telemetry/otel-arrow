// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Programmatic scenario builder for validation tests.

use crate::error::ValidationError;
use crate::pipeline::Pipeline;
use crate::simulate::run_pipelines_with_timeout;
use crate::traffic::{Capture, Generator, TlsConfig};
use minijinja::{Environment, context};
use portpicker::pick_unused_port;
use std::fs;
use std::path::PathBuf;
use std::time::Duration;

/// Backwards-compatible alias for [`TlsConfig`].
pub type TlsScenarioConfig = TlsConfig;

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
    input: Option<Generator>,
    observation: Option<Capture>,
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
            input: None,
            observation: None,
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

    /// Attach an input generator.
    #[must_use]
    pub fn input(mut self, generator: Generator) -> Self {
        self.input = Some(generator);
        self
    }

    /// Attach an observation sink.
    #[must_use]
    pub fn observe(mut self, capture: Capture) -> Self {
        self.observation = Some(capture);
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
        let max_signal_count = self
            .input
            .as_ref()
            .ok_or_else(|| ValidationError::Config("input missing after config update".into()))?
            .max_signal_count as u64;
        let admin_base = format!("http://{}", self.admin_addr);

        let rendered_group = self.render_template()?;

        let tokio_rt = tokio::runtime::Runtime::new()
            .map_err(|e| ValidationError::Io(format!("failed to create tokio runtime: {e}")))?;

        tokio_rt.block_on(async move {
            run_pipelines_with_timeout(
                rendered_group,
                admin_base,
                max_signal_count,
                timeout,
                ready_max_attempts,
                ready_backoff,
                metrics_poll,
                propagation_delay,
            )
            .await
        })
    }

    fn render_template(&self) -> Result<String, ValidationError> {
        let pipeline_yaml = self
            .pipeline
            .as_ref()
            .ok_or_else(|| ValidationError::Config("pipeline missing".into()))?
            .to_yaml_string()?;
        let traffic_generation_config = self
            .input
            .as_ref()
            .ok_or_else(|| ValidationError::Config("input missing after config update".into()))?;
        let traffic_capture_config = self.observation.as_ref().ok_or_else(|| {
            ValidationError::Config("observation missing after config update".into())
        })?;

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
        let tls = traffic_generation_config.tls.as_ref();
        let tls_enabled = tls.is_some();
        let tls_ca_cert = tls.map(TlsConfig::ca_cert_str).transpose()?.unwrap_or("");
        let tls_client_cert = tls
            .map(TlsConfig::client_cert_str)
            .transpose()?
            .unwrap_or("");
        let tls_client_key = tls
            .map(TlsConfig::client_key_str)
            .transpose()?
            .unwrap_or("");
        let mtls_enabled = tls.is_some_and(TlsConfig::is_mtls);
        let tls_server_name = tls.map_or("localhost", |t| t.server_name.as_str());

        let ctx = context! {
            suv_receiver_type => &traffic_capture_config.suv_receiver_type,
            suv_exporter_type => &traffic_generation_config.suv_exporter_type,
            max_signal_count => traffic_generation_config.max_signal_count,
            max_batch_size => traffic_generation_config.max_batch_size,
            signals_per_second => traffic_generation_config.signals_per_second,
            metric_weight => traffic_generation_config.metric_weight,
            trace_weight => traffic_generation_config.trace_weight,
            log_weight => traffic_generation_config.log_weight,
            suv_addr => &traffic_capture_config.suv_listening_addr,
            suv_endpoint => &traffic_generation_config.suv_endpoint,
            control_addr => &traffic_capture_config.control_listening_addr,
            control_endpoint => &traffic_generation_config.control_endpoint,
            pipeline_config => pipeline_yaml,
            admin_bind_address => &self.admin_addr,
            tls_enabled => tls_enabled,
            tls_ca_cert => tls_ca_cert,
            tls_client_cert => tls_client_cert,
            tls_client_key => tls_client_key,
            mtls_enabled => mtls_enabled,
            tls_server_name => tls_server_name,
        };
        tmpl.render(ctx)
            .map_err(|e| ValidationError::Template(e.to_string()))
    }

    fn update_configs(&mut self) -> Result<(), ValidationError> {
        let pipeline = self
            .pipeline
            .as_mut()
            .ok_or_else(|| ValidationError::Config("pipeline not provided".into()))?;
        let generator = self
            .input
            .take()
            .ok_or_else(|| ValidationError::Config("no input configured".into()))?;
        let capture = self
            .observation
            .take()
            .ok_or_else(|| ValidationError::Config("no observation configured".into()))?;
        if pipeline.input_wire.is_none() {
            return Err(ValidationError::Config(
                "no receiver wired in pipeline".into(),
            ));
        }
        if pipeline.output_wire.is_none() {
            return Err(ValidationError::Config(
                "no exporter wired in pipeline".into(),
            ));
        }
        let input_port = pick_unused_port()
            .ok_or_else(|| ValidationError::Config("failed to get new port for config".into()))?;
        let output_port = pick_unused_port()
            .ok_or_else(|| ValidationError::Config("failed to get new port for config".into()))?;
        let control_port = pick_unused_port()
            .ok_or_else(|| ValidationError::Config("failed to get new port for config".into()))?;
        let admin_port = pick_unused_port()
            .ok_or_else(|| ValidationError::Config("failed to get new port for config".into()))?;

        let input_addr = format!("127.0.0.1:{input_port}");
        let output_addr = format!("127.0.0.1:{output_port}");
        let control_addr = format!("127.0.0.1:{control_port}");
        let admin_addr = format!("127.0.0.1:{admin_port}");
        let suv_scheme = if generator.tls.is_some() {
            "https"
        } else {
            "http"
        };
        pipeline.update_pipeline(&input_addr, &format!("http://{output_addr}"))?;

        let traffic_generation_config = Generator {
            suv_endpoint: format!("{suv_scheme}://{input_addr}"),
            control_endpoint: format!("http://{control_addr}"),
            ..generator
        };
        let traffic_capture_config = Capture {
            suv_listening_addr: output_addr,
            control_listening_addr: control_addr,
            ..capture
        };
        self.input = Some(traffic_generation_config);
        self.observation = Some(traffic_capture_config);
        self.admin_addr = admin_addr;
        Ok(())
    }
}
