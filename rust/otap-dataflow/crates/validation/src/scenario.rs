// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Programmatic scenario builder for validation tests.

use crate::error::ValidationError;
use crate::pipeline::Pipeline;
use crate::simulate::run_pipelines_with_timeout;
use crate::traffic::{Capture, Generator};
use minijinja::{Environment, context};
use portpicker::pick_unused_port;
use std::fs;
use std::path::PathBuf;
use std::time::Duration;

/// TLS configuration for validation scenarios.
///
/// When provided, the traffic-generator exporter connects to the SUV receiver
/// over TLS (or mTLS) instead of plain-text gRPC.
///
/// Construct via [`TlsScenarioConfig::tls_only`] or [`TlsScenarioConfig::mtls`].
///
/// **Note:** Requires the `experimental-tls` feature to be enabled on `otap-df-otap`,
/// otherwise the rendered pipeline config will fail to deserialize.
pub struct TlsScenarioConfig {
    ca_cert_path: PathBuf,
    client_cert_path: Option<PathBuf>,
    client_key_path: Option<PathBuf>,
    server_name: String,
}

impl TlsScenarioConfig {
    /// Create a TLS-only config (no client cert).
    #[must_use]
    pub fn tls_only(ca_cert_path: impl Into<PathBuf>) -> Self {
        Self {
            ca_cert_path: ca_cert_path.into(),
            client_cert_path: None,
            client_key_path: None,
            server_name: "localhost".to_string(),
        }
    }

    /// Create an mTLS config with client certificate and key.
    #[must_use]
    pub fn mtls(
        ca_cert_path: impl Into<PathBuf>,
        client_cert_path: impl Into<PathBuf>,
        client_key_path: impl Into<PathBuf>,
    ) -> Self {
        Self {
            ca_cert_path: ca_cert_path.into(),
            client_cert_path: Some(client_cert_path.into()),
            client_key_path: Some(client_key_path.into()),
            server_name: "localhost".to_string(),
        }
    }

    /// Override the server name used for TLS SNI and certificate verification.
    ///
    /// Defaults to `"localhost"`. Set this when the server certificate uses a
    /// different SAN/CN than `localhost`.
    #[must_use]
    pub fn with_server_name(mut self, name: impl Into<String>) -> Self {
        self.server_name = name.into();
        self
    }

    fn ca_cert_str(&self) -> Result<&str, ValidationError> {
        self.ca_cert_path
            .to_str()
            .ok_or_else(|| ValidationError::Config("ca_cert_path is not valid UTF-8".into()))
    }

    fn client_cert_str(&self) -> Result<&str, ValidationError> {
        match self.client_cert_path.as_deref() {
            None => Ok(""),
            Some(path) => path.to_str().ok_or_else(|| {
                ValidationError::Config("client_cert_path is not valid UTF-8".into())
            }),
        }
    }

    fn client_key_str(&self) -> Result<&str, ValidationError> {
        match self.client_key_path.as_deref() {
            None => Ok(""),
            Some(path) => path.to_str().ok_or_else(|| {
                ValidationError::Config("client_key_path is not valid UTF-8".into())
            }),
        }
    }

    fn is_mtls(&self) -> bool {
        self.client_cert_path.is_some() && self.client_key_path.is_some()
    }
}

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
    tls: Option<TlsScenarioConfig>,
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
            tls: None,
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

    /// Enable TLS (or mTLS) between the traffic generator and the SUV receiver.
    #[must_use]
    pub fn with_tls(mut self, config: TlsScenarioConfig) -> Self {
        self.tls = Some(config);
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
        let tls_enabled = self.tls.is_some();
        let tls_ca_cert = self
            .tls
            .as_ref()
            .map(TlsScenarioConfig::ca_cert_str)
            .transpose()?
            .unwrap_or("");
        let tls_client_cert = self
            .tls
            .as_ref()
            .map(TlsScenarioConfig::client_cert_str)
            .transpose()?
            .unwrap_or("");
        let tls_client_key = self
            .tls
            .as_ref()
            .map(TlsScenarioConfig::client_key_str)
            .transpose()?
            .unwrap_or("");
        let mtls_enabled = self.tls.as_ref().is_some_and(TlsScenarioConfig::is_mtls);
        let tls_server_name = self
            .tls
            .as_ref()
            .map_or("localhost", |t| t.server_name.as_str());

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
        let suv_scheme = if self.tls.is_some() { "https" } else { "http" };
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
