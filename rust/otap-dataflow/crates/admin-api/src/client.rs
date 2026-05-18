// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Public admin client facade and resource clients.

use crate::endpoint::{AdminAuth, AdminEndpoint};
use crate::http_backend::HttpBackend;
use crate::{Error, engine, groups, operations, pipelines, telemetry};
use async_trait::async_trait;
use std::sync::Arc;
use std::time::Duration;

use crate::config::tls::TlsClientConfig;

const DEFAULT_CONNECT_TIMEOUT: Duration = Duration::from_secs(3);
const DEFAULT_TCP_KEEPALIVE: Duration = Duration::from_secs(45);
const DEFAULT_TCP_NODELAY: bool = true;

/// HTTP client settings for the admin SDK.
#[derive(Debug, Clone)]
pub struct HttpAdminClientSettings {
    /// Target endpoint.
    pub endpoint: AdminEndpoint,
    /// Authentication configuration.
    pub auth: AdminAuth,
    /// Timeout for establishing TCP connections.
    pub connect_timeout: Duration,
    /// Whether to enable `TCP_NODELAY`.
    pub tcp_nodelay: bool,
    /// TCP keepalive timeout for outbound connections.
    pub tcp_keepalive: Option<Duration>,
    /// Interval between TCP keepalive probes once keepalive is active.
    pub tcp_keepalive_interval: Option<Duration>,
    /// Timeout for requests. If omitted, no request timeout is applied.
    pub timeout: Option<Duration>,
    /// Optional TLS/mTLS configuration for HTTPS endpoints.
    pub tls: Option<TlsClientConfig>,
}

impl HttpAdminClientSettings {
    /// Creates HTTP client settings with the SDK defaults for connection behavior.
    ///
    /// Use the builder-style `with_*` methods to override auth, timeout,
    /// keepalive, or TLS behavior.
    #[must_use]
    pub fn new(endpoint: AdminEndpoint) -> Self {
        Self {
            endpoint,
            auth: AdminAuth::None,
            connect_timeout: DEFAULT_CONNECT_TIMEOUT,
            tcp_nodelay: DEFAULT_TCP_NODELAY,
            tcp_keepalive: Some(DEFAULT_TCP_KEEPALIVE),
            tcp_keepalive_interval: None,
            timeout: None,
            tls: None,
        }
    }

    /// Sets the authentication mode for requests sent by this client.
    #[must_use]
    pub fn with_auth(mut self, auth: AdminAuth) -> Self {
        self.auth = auth;
        self
    }

    /// Sets the TCP connect timeout for establishing new connections.
    #[must_use]
    pub fn with_connect_timeout(mut self, connect_timeout: Duration) -> Self {
        self.connect_timeout = connect_timeout;
        self
    }

    /// Sets a per-request timeout for admin calls.
    ///
    /// This is separate from [`operations::OperationOptions::timeout_secs`],
    /// which controls how long the server should wait on long-running
    /// operations such as reconfigure or shutdown.
    #[must_use]
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }

    /// Disables the client-side per-request timeout.
    #[must_use]
    pub fn without_timeout(mut self) -> Self {
        self.timeout = None;
        self
    }

    /// Sets whether outbound TCP sockets should use `TCP_NODELAY`.
    #[must_use]
    pub fn with_tcp_nodelay(mut self, tcp_nodelay: bool) -> Self {
        self.tcp_nodelay = tcp_nodelay;
        self
    }

    /// Sets the TCP keepalive timeout for outbound connections.
    #[must_use]
    pub fn with_tcp_keepalive(mut self, tcp_keepalive: Option<Duration>) -> Self {
        self.tcp_keepalive = tcp_keepalive;
        self
    }

    /// Sets the interval between TCP keepalive probes when keepalive is enabled.
    #[must_use]
    pub fn with_tcp_keepalive_interval(mut self, tcp_keepalive_interval: Option<Duration>) -> Self {
        self.tcp_keepalive_interval = tcp_keepalive_interval;
        self
    }

    /// Sets the TLS or mTLS configuration for HTTPS endpoints.
    ///
    /// This is ignored for plaintext HTTP endpoints and required only when the
    /// target endpoint needs custom CA trust, client certificates, or other TLS
    /// overrides.
    #[must_use]
    pub fn with_tls(mut self, tls: TlsClientConfig) -> Self {
        self.tls = Some(tls);
        self
    }
}

enum BackendConfig {
    Http(HttpAdminClientSettings),
}

/// Builder for `AdminClient`.
#[derive(Default)]
pub struct AdminClientBuilder {
    backend: Option<BackendConfig>,
}

impl AdminClientBuilder {
    /// Creates a new admin client builder with no backend configured yet.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Configures the client to use the HTTP admin transport.
    #[must_use]
    pub fn http(mut self, settings: HttpAdminClientSettings) -> Self {
        self.backend = Some(BackendConfig::Http(settings));
        self
    }

    /// Builds the configured admin client.
    ///
    /// Returns an error when no backend has been configured or when the HTTP
    /// transport settings are invalid.
    pub fn build(self) -> Result<AdminClient, Error> {
        let backend = match self.backend {
            Some(BackendConfig::Http(settings)) => {
                Arc::new(HttpBackend::from_settings(settings)?) as Arc<dyn AdminBackend>
            }
            None => {
                return Err(Error::ClientConfig {
                    details: "no admin backend configured".to_string(),
                });
            }
        };

        Ok(AdminClient { backend })
    }
}

/// Public async admin client root.
#[derive(Clone)]
pub struct AdminClient {
    backend: Arc<dyn AdminBackend>,
}

impl AdminClient {
    /// Creates a builder for constructing an [`AdminClient`].
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use otap_df_admin_api::{AdminClient, AdminEndpoint, HttpAdminClientSettings};
    /// # fn example() -> Result<(), otap_df_admin_api::Error> {
    /// let client = AdminClient::builder()
    ///     .http(HttpAdminClientSettings::new(AdminEndpoint::http(
    ///         "engine-a.internal.example",
    ///         8080,
    ///     )))
    ///     .build()?;
    ///
    /// # let _ = client;
    /// # Ok(())
    /// # }
    /// ```
    #[must_use]
    pub fn builder() -> AdminClientBuilder {
        AdminClientBuilder::new()
    }

    /// Returns the engine-scoped resource client for engine-wide status and probes.
    #[must_use]
    pub fn engine(&self) -> EngineClient<'_> {
        EngineClient {
            backend: self.backend.as_ref(),
        }
    }

    /// Returns the group-scoped resource client for fleet-style status and shutdown operations.
    #[must_use]
    pub fn groups(&self) -> GroupsClient<'_> {
        GroupsClient {
            backend: self.backend.as_ref(),
        }
    }

    /// Returns the pipeline-scoped resource client for per-pipeline status and live control.
    #[must_use]
    pub fn pipelines(&self) -> PipelinesClient<'_> {
        PipelinesClient {
            backend: self.backend.as_ref(),
        }
    }

    /// Returns the telemetry-scoped resource client for logs and structured metrics.
    #[must_use]
    pub fn telemetry(&self) -> TelemetryClient<'_> {
        TelemetryClient {
            backend: self.backend.as_ref(),
        }
    }
}

/// Engine-scoped admin client.
#[derive(Clone, Copy)]
pub struct EngineClient<'a> {
    backend: &'a dyn AdminBackend,
}

impl EngineClient<'_> {
    /// Returns the current engine-wide status snapshot.
    ///
    /// Use this when you need a cross-pipeline view of the running engine.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use otap_df_admin_api::{AdminClient, AdminEndpoint, HttpAdminClientSettings};
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// # let client = AdminClient::builder()
    /// #     .http(HttpAdminClientSettings::new(AdminEndpoint::http(
    /// #         "engine-a.internal.example",
    /// #         8080,
    /// #     )))
    /// #     .build()?;
    /// let status = client.engine().status().await?;
    /// println!("pipelines={}", status.pipelines.len());
    /// # Ok(())
    /// # }
    /// ```
    pub async fn status(&self) -> Result<engine::Status, Error> {
        self.backend.engine_status().await
    }

    /// Returns the engine liveness probe result.
    ///
    /// This is the SDK equivalent of checking whether the engine process is
    /// live enough to keep serving admin traffic.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use otap_df_admin_api::{AdminClient, AdminEndpoint, HttpAdminClientSettings};
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// # let client = AdminClient::builder()
    /// #     .http(HttpAdminClientSettings::new(AdminEndpoint::http(
    /// #         "engine-a.internal.example",
    /// #         8080,
    /// #     )))
    /// #     .build()?;
    /// let probe = client.engine().livez().await?;
    /// println!("livez={:?}", probe.status);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn livez(&self) -> Result<engine::ProbeResponse, Error> {
        self.backend.engine_livez().await
    }

    /// Returns the engine readiness probe result.
    ///
    /// Use this when orchestration or callers need to know whether the engine
    /// currently considers itself ready.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use otap_df_admin_api::{AdminClient, AdminEndpoint, HttpAdminClientSettings};
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// # let client = AdminClient::builder()
    /// #     .http(HttpAdminClientSettings::new(AdminEndpoint::http(
    /// #         "engine-a.internal.example",
    /// #         8080,
    /// #     )))
    /// #     .build()?;
    /// let probe = client.engine().readyz().await?;
    /// println!("readyz={:?}", probe.status);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn readyz(&self) -> Result<engine::ProbeResponse, Error> {
        self.backend.engine_readyz().await
    }
}

/// Group-scoped admin client.
#[derive(Clone, Copy)]
pub struct GroupsClient<'a> {
    backend: &'a dyn AdminBackend,
}

impl GroupsClient<'_> {
    /// Returns a group-wide status snapshot across logical pipelines.
    ///
    /// Use this as a fleet-style overview when you do not need full
    /// engine-wide detail from [`EngineClient::status`].
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use otap_df_admin_api::{AdminClient, AdminEndpoint, HttpAdminClientSettings};
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// # let client = AdminClient::builder()
    /// #     .http(HttpAdminClientSettings::new(AdminEndpoint::http(
    /// #         "engine-a.internal.example",
    /// #         8080,
    /// #     )))
    /// #     .build()?;
    /// let status = client.groups().status().await?;
    /// println!("pipelines={}", status.pipelines.len());
    /// # Ok(())
    /// # }
    /// ```
    pub async fn status(&self) -> Result<groups::Status, Error> {
        self.backend.groups_status().await
    }

    /// Requests coordinated shutdown for all running logical pipelines.
    ///
    /// Use `options.wait` to choose whether the call should return immediately
    /// with the server's current shutdown response or wait up to
    /// `options.timeout_secs` for a terminal shutdown result.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use otap_df_admin_api::{
    /// #     groups, operations, AdminClient, AdminEndpoint, HttpAdminClientSettings,
    /// # };
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// # let client = AdminClient::builder()
    /// #     .http(HttpAdminClientSettings::new(AdminEndpoint::http(
    /// #         "engine-a.internal.example",
    /// #         8080,
    /// #     )))
    /// #     .build()?;
    /// let response = client
    ///     .groups()
    ///     .shutdown(&operations::OperationOptions {
    ///         wait: true,
    ///         timeout_secs: 30,
    ///     })
    ///     .await?;
    ///
    /// if matches!(
    ///     response.status,
    ///     groups::ShutdownStatus::Failed | groups::ShutdownStatus::Timeout
    /// ) {
    ///     eprintln!("shutdown issues: {:?}", response.errors);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn shutdown(
        &self,
        options: &operations::OperationOptions,
    ) -> Result<groups::ShutdownResponse, Error> {
        self.backend.groups_shutdown(options).await
    }
}

/// Pipeline-scoped admin client.
#[derive(Clone, Copy)]
pub struct PipelinesClient<'a> {
    backend: &'a dyn AdminBackend,
}

impl PipelinesClient<'_> {
    /// Returns the committed live configuration for one logical pipeline.
    ///
    /// Use this when you need the configuration that the controller currently
    /// treats as active. This does not include per-core runtime progress or
    /// overlapping-instance state; use [`Self::status`] for runtime status.
    ///
    /// Returns `Ok(None)` when the logical pipeline is not found.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use otap_df_admin_api::{AdminClient, AdminEndpoint, HttpAdminClientSettings};
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// # let client = AdminClient::builder()
    /// #     .http(HttpAdminClientSettings::new(AdminEndpoint::http(
    /// #         "engine-a.internal.example",
    /// #         8080,
    /// #     )))
    /// #     .build()?;
    /// if let Some(details) = client
    ///     .pipelines()
    ///     .details("tenant-a", "ingest")
    ///     .await?
    /// {
    ///     println!("active_generation={:?}", details.active_generation);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn details(
        &self,
        pipeline_group_id: &str,
        pipeline_id: &str,
    ) -> Result<Option<pipelines::PipelineDetails>, Error> {
        self.backend
            .pipeline_details(pipeline_group_id, pipeline_id)
            .await
    }

    /// Submits a live reconfiguration request for one logical pipeline.
    ///
    /// The controller may treat the request as a create, resize, replace, or
    /// no-op depending on how the submitted configuration differs from the
    /// current committed pipeline.
    ///
    /// With `options.wait = false`, this returns as soon as the request has
    /// either been accepted for background execution or already completed,
    /// yielding [`pipelines::ReconfigureOutcome::Accepted`] or
    /// [`pipelines::ReconfigureOutcome::Completed`].
    ///
    /// With `options.wait = true`, this waits up to `options.timeout_secs` for
    /// a terminal result and returns the latest rollout snapshot as
    /// [`pipelines::ReconfigureOutcome::Completed`],
    /// [`pipelines::ReconfigureOutcome::Failed`], or
    /// [`pipelines::ReconfigureOutcome::TimedOut`].
    ///
    /// If the server rejects the request before a rollout starts, this returns
    /// [`Error::AdminOperation`].
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use otap_df_admin_api::{
    /// #     config::pipeline::{PipelineConfigBuilder, PipelineType},
    /// #     operations, pipelines, AdminClient, AdminEndpoint, HttpAdminClientSettings,
    /// # };
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// # let client = AdminClient::builder()
    /// #     .http(HttpAdminClientSettings::new(AdminEndpoint::http(
    /// #         "engine-a.internal.example",
    /// #         8080,
    /// #     )))
    /// #     .build()?;
    /// # let request = pipelines::ReconfigureRequest {
    /// #     pipeline: PipelineConfigBuilder::new()
    /// #         .add_receiver("ingress", "receiver:otlp", None)
    /// #         .add_exporter("egress", "exporter:debug", None)
    /// #         .to("ingress", "egress")
    /// #         .build(PipelineType::Otap, "tenant-a", "ingest")?,
    /// #     step_timeout_secs: 60,
    /// #     drain_timeout_secs: 60,
    /// # };
    /// let outcome = client
    ///     .pipelines()
    ///     .reconfigure(
    ///         "tenant-a",
    ///         "ingest",
    ///         &request,
    ///         &operations::OperationOptions {
    ///             wait: true,
    ///             timeout_secs: 120,
    ///         },
    ///     )
    ///     .await?;
    ///
    /// match outcome {
    ///     pipelines::ReconfigureOutcome::Completed(status) => {
    ///         println!("rolled out generation {}", status.target_generation);
    ///     }
    ///     pipelines::ReconfigureOutcome::Accepted(status) => {
    ///         println!("poll rollout {}", status.rollout_id);
    ///     }
    ///     pipelines::ReconfigureOutcome::Failed(status)
    ///     | pipelines::ReconfigureOutcome::TimedOut(status) => {
    ///         eprintln!("rollout state: {:?}", status.state);
    ///     }
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn reconfigure(
        &self,
        pipeline_group_id: &str,
        pipeline_id: &str,
        request: &pipelines::ReconfigureRequest,
        options: &operations::OperationOptions,
    ) -> Result<pipelines::ReconfigureOutcome, Error> {
        self.backend
            .pipeline_reconfigure(pipeline_group_id, pipeline_id, request, options)
            .await
    }

    /// Returns the latest known status for one previously created rollout.
    ///
    /// Use the `rollout_id` returned from [`Self::reconfigure`] to poll an
    /// asynchronous reconfiguration operation after an
    /// [`pipelines::ReconfigureOutcome::Accepted`] result.
    ///
    /// Returns `Ok(None)` when the requested rollout status resource is not
    /// found. Terminal rollout history is retained only within a bounded
    /// in-memory window, so older rollout ids may also return `Ok(None)` after
    /// eviction.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use otap_df_admin_api::{AdminClient, AdminEndpoint, HttpAdminClientSettings};
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// # let client = AdminClient::builder()
    /// #     .http(HttpAdminClientSettings::new(AdminEndpoint::http(
    /// #         "engine-a.internal.example",
    /// #         8080,
    /// #     )))
    /// #     .build()?;
    /// let rollout_id = "rollout-42";
    ///
    /// if let Some(status) = client
    ///     .pipelines()
    ///     .rollout_status("tenant-a", "ingest", rollout_id)
    ///     .await?
    /// {
    ///     println!("rollout_state={:?}", status.state);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn rollout_status(
        &self,
        pipeline_group_id: &str,
        pipeline_id: &str,
        rollout_id: &str,
    ) -> Result<Option<pipelines::RolloutStatus>, Error> {
        self.backend
            .pipeline_rollout_status(pipeline_group_id, pipeline_id, rollout_id)
            .await
    }

    /// Returns the current runtime status for one logical pipeline.
    ///
    /// Use this when you need per-core phase, overlapping-instance state,
    /// rollout summaries, or other runtime progress. Use [`Self::details`] when
    /// you need the committed live configuration instead.
    ///
    /// Returns `Ok(None)` when the logical pipeline is not found.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use otap_df_admin_api::{AdminClient, AdminEndpoint, HttpAdminClientSettings};
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// # let client = AdminClient::builder()
    /// #     .http(HttpAdminClientSettings::new(AdminEndpoint::http(
    /// #         "engine-a.internal.example",
    /// #         8080,
    /// #     )))
    /// #     .build()?;
    /// if let Some(status) = client
    ///     .pipelines()
    ///     .status("tenant-a", "ingest")
    ///     .await?
    /// {
    ///     println!("running_cores={}", status.running_cores);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn status(
        &self,
        pipeline_group_id: &str,
        pipeline_id: &str,
    ) -> Result<Option<pipelines::Status>, Error> {
        self.backend
            .pipeline_status(pipeline_group_id, pipeline_id)
            .await
    }

    /// Requests shutdown of the currently running instances for one logical pipeline.
    ///
    /// With `options.wait = false`, this returns as soon as the shutdown has
    /// either been accepted for background execution or already completed,
    /// yielding [`pipelines::ShutdownOutcome::Accepted`] or
    /// [`pipelines::ShutdownOutcome::Completed`].
    ///
    /// With `options.wait = true`, this waits up to `options.timeout_secs` for
    /// a terminal result and returns the latest shutdown snapshot as
    /// [`pipelines::ShutdownOutcome::Completed`],
    /// [`pipelines::ShutdownOutcome::Failed`], or
    /// [`pipelines::ShutdownOutcome::TimedOut`].
    ///
    /// If the server rejects the request before shutdown work starts, this
    /// returns [`Error::AdminOperation`].
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use otap_df_admin_api::{operations, pipelines, AdminClient, AdminEndpoint, HttpAdminClientSettings};
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// # let client = AdminClient::builder()
    /// #     .http(HttpAdminClientSettings::new(AdminEndpoint::http(
    /// #         "engine-a.internal.example",
    /// #         8080,
    /// #     )))
    /// #     .build()?;
    /// let outcome = client
    ///     .pipelines()
    ///     .shutdown(
    ///         "tenant-a",
    ///         "ingest",
    ///         &operations::OperationOptions {
    ///             wait: true,
    ///             timeout_secs: 60,
    ///         },
    ///     )
    ///     .await?;
    ///
    /// match outcome {
    ///     pipelines::ShutdownOutcome::Completed(status) => {
    ///         println!("shutdown completed: {}", status.shutdown_id);
    ///     }
    ///     pipelines::ShutdownOutcome::Accepted(status) => {
    ///         println!("poll shutdown {}", status.shutdown_id);
    ///     }
    ///     pipelines::ShutdownOutcome::Failed(status)
    ///     | pipelines::ShutdownOutcome::TimedOut(status) => {
    ///         eprintln!("shutdown state: {}", status.state);
    ///     }
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn shutdown(
        &self,
        pipeline_group_id: &str,
        pipeline_id: &str,
        options: &operations::OperationOptions,
    ) -> Result<pipelines::ShutdownOutcome, Error> {
        self.backend
            .pipeline_shutdown(pipeline_group_id, pipeline_id, options)
            .await
    }

    /// Returns the latest known status for one previously created shutdown operation.
    ///
    /// Use the `shutdown_id` returned from [`Self::shutdown`] to poll an
    /// asynchronous shutdown after an
    /// [`pipelines::ShutdownOutcome::Accepted`] result.
    ///
    /// Returns `Ok(None)` when the requested shutdown status resource is not
    /// found. Terminal shutdown history is retained only within a bounded
    /// in-memory window, so older shutdown ids may also return `Ok(None)` after
    /// eviction.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use otap_df_admin_api::{AdminClient, AdminEndpoint, HttpAdminClientSettings};
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// # let client = AdminClient::builder()
    /// #     .http(HttpAdminClientSettings::new(AdminEndpoint::http(
    /// #         "engine-a.internal.example",
    /// #         8080,
    /// #     )))
    /// #     .build()?;
    /// let shutdown_id = "shutdown-42";
    ///
    /// if let Some(status) = client
    ///     .pipelines()
    ///     .shutdown_status("tenant-a", "ingest", shutdown_id)
    ///     .await?
    /// {
    ///     println!("shutdown_state={}", status.state);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn shutdown_status(
        &self,
        pipeline_group_id: &str,
        pipeline_id: &str,
        shutdown_id: &str,
    ) -> Result<Option<pipelines::ShutdownStatus>, Error> {
        self.backend
            .pipeline_shutdown_status(pipeline_group_id, pipeline_id, shutdown_id)
            .await
    }

    /// Returns the liveness probe result for one logical pipeline.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use otap_df_admin_api::{pipelines, AdminClient, AdminEndpoint, HttpAdminClientSettings};
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// # let client = AdminClient::builder()
    /// #     .http(HttpAdminClientSettings::new(AdminEndpoint::http(
    /// #         "engine-a.internal.example",
    /// #         8080,
    /// #     )))
    /// #     .build()?;
    /// let probe = client.pipelines().livez("tenant-a", "ingest").await?;
    ///
    /// if probe.status == pipelines::ProbeStatus::Failed {
    ///     eprintln!("pipeline is not live: {:?}", probe.message);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn livez(
        &self,
        pipeline_group_id: &str,
        pipeline_id: &str,
    ) -> Result<pipelines::ProbeResult, Error> {
        self.backend
            .pipeline_livez(pipeline_group_id, pipeline_id)
            .await
    }

    /// Returns the readiness probe result for one logical pipeline.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use otap_df_admin_api::{pipelines, AdminClient, AdminEndpoint, HttpAdminClientSettings};
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// # let client = AdminClient::builder()
    /// #     .http(HttpAdminClientSettings::new(AdminEndpoint::http(
    /// #         "engine-a.internal.example",
    /// #         8080,
    /// #     )))
    /// #     .build()?;
    /// let probe = client.pipelines().readyz("tenant-a", "ingest").await?;
    ///
    /// if probe.status == pipelines::ProbeStatus::Failed {
    ///     eprintln!("pipeline is not ready: {:?}", probe.message);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn readyz(
        &self,
        pipeline_group_id: &str,
        pipeline_id: &str,
    ) -> Result<pipelines::ProbeResult, Error> {
        self.backend
            .pipeline_readyz(pipeline_group_id, pipeline_id)
            .await
    }
}

/// Telemetry-scoped admin client.
#[derive(Clone, Copy)]
pub struct TelemetryClient<'a> {
    backend: &'a dyn AdminBackend,
}

impl TelemetryClient<'_> {
    /// Returns retained admin logs.
    ///
    /// Use [`telemetry::LogsQuery`] to request only entries newer than a known
    /// sequence number or to cap the number of returned entries.
    ///
    /// Returns `Ok(None)` when retained logs are not available on the target
    /// engine.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use otap_df_admin_api::{telemetry, AdminClient, AdminEndpoint, HttpAdminClientSettings};
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// # let client = AdminClient::builder()
    /// #     .http(HttpAdminClientSettings::new(AdminEndpoint::http(
    /// #         "engine-a.internal.example",
    /// #         8080,
    /// #     )))
    /// #     .build()?;
    /// let logs = client
    ///     .telemetry()
    ///     .logs(&telemetry::LogsQuery {
    ///         after: Some(1_000),
    ///         limit: Some(200),
    ///     })
    ///     .await?;
    ///
    /// if let Some(logs) = logs {
    ///     println!("next_seq={}", logs.next_seq);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn logs(
        &self,
        query: &telemetry::LogsQuery,
    ) -> Result<Option<telemetry::LogsResponse>, Error> {
        self.backend.telemetry_logs(query).await
    }

    /// Returns structured metrics with descriptor metadata for each metric field.
    ///
    /// Use this form when callers need metric names, units, instrument kinds,
    /// or temporality alongside metric values.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use otap_df_admin_api::{telemetry, AdminClient, AdminEndpoint, HttpAdminClientSettings};
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// # let client = AdminClient::builder()
    /// #     .http(HttpAdminClientSettings::new(AdminEndpoint::http(
    /// #         "engine-a.internal.example",
    /// #         8080,
    /// #     )))
    /// #     .build()?;
    /// let metrics = client
    ///     .telemetry()
    ///     .metrics(&telemetry::MetricsOptions::default())
    ///     .await?;
    ///
    /// if let Some(metric_set) = metrics.metric_sets.first() {
    ///     for point in &metric_set.metrics {
    ///         println!("{} {}", point.metadata.name, point.metadata.unit);
    ///     }
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn metrics(
        &self,
        options: &telemetry::MetricsOptions,
    ) -> Result<telemetry::MetricsResponse, Error> {
        self.backend.telemetry_metrics(options).await
    }

    /// Returns structured metrics without per-field descriptor metadata.
    ///
    /// Use this form when callers only need current metric values and want a
    /// smaller response payload than [`Self::metrics`].
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use otap_df_admin_api::{telemetry, AdminClient, AdminEndpoint, HttpAdminClientSettings};
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// # let client = AdminClient::builder()
    /// #     .http(HttpAdminClientSettings::new(AdminEndpoint::http(
    /// #         "engine-a.internal.example",
    /// #         8080,
    /// #     )))
    /// #     .build()?;
    /// let metrics = client
    ///     .telemetry()
    ///     .metrics_compact(&telemetry::MetricsOptions::default())
    ///     .await?;
    ///
    /// if let Some(metric_set) = metrics.metric_sets.first() {
    ///     println!("value_count={}", metric_set.metrics.len());
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn metrics_compact(
        &self,
        options: &telemetry::MetricsOptions,
    ) -> Result<telemetry::CompactMetricsResponse, Error> {
        self.backend.telemetry_metrics_compact(options).await
    }
}

#[async_trait]
pub(crate) trait AdminBackend: Send + Sync {
    async fn engine_status(&self) -> Result<engine::Status, Error>;
    async fn engine_livez(&self) -> Result<engine::ProbeResponse, Error>;
    async fn engine_readyz(&self) -> Result<engine::ProbeResponse, Error>;

    async fn groups_status(&self) -> Result<groups::Status, Error>;
    async fn groups_shutdown(
        &self,
        options: &operations::OperationOptions,
    ) -> Result<groups::ShutdownResponse, Error>;

    async fn pipeline_status(
        &self,
        pipeline_group_id: &str,
        pipeline_id: &str,
    ) -> Result<Option<pipelines::Status>, Error>;
    async fn pipeline_details(
        &self,
        pipeline_group_id: &str,
        pipeline_id: &str,
    ) -> Result<Option<pipelines::PipelineDetails>, Error>;
    async fn pipeline_reconfigure(
        &self,
        pipeline_group_id: &str,
        pipeline_id: &str,
        request: &pipelines::ReconfigureRequest,
        options: &operations::OperationOptions,
    ) -> Result<pipelines::ReconfigureOutcome, Error>;
    async fn pipeline_rollout_status(
        &self,
        pipeline_group_id: &str,
        pipeline_id: &str,
        rollout_id: &str,
    ) -> Result<Option<pipelines::RolloutStatus>, Error>;
    async fn pipeline_livez(
        &self,
        pipeline_group_id: &str,
        pipeline_id: &str,
    ) -> Result<pipelines::ProbeResult, Error>;
    async fn pipeline_readyz(
        &self,
        pipeline_group_id: &str,
        pipeline_id: &str,
    ) -> Result<pipelines::ProbeResult, Error>;
    async fn pipeline_shutdown(
        &self,
        pipeline_group_id: &str,
        pipeline_id: &str,
        options: &operations::OperationOptions,
    ) -> Result<pipelines::ShutdownOutcome, Error>;
    async fn pipeline_shutdown_status(
        &self,
        pipeline_group_id: &str,
        pipeline_id: &str,
        shutdown_id: &str,
    ) -> Result<Option<pipelines::ShutdownStatus>, Error>;

    async fn telemetry_logs(
        &self,
        query: &telemetry::LogsQuery,
    ) -> Result<Option<telemetry::LogsResponse>, Error>;
    async fn telemetry_metrics(
        &self,
        options: &telemetry::MetricsOptions,
    ) -> Result<telemetry::MetricsResponse, Error>;
    async fn telemetry_metrics_compact(
        &self,
        options: &telemetry::MetricsOptions,
    ) -> Result<telemetry::CompactMetricsResponse, Error>;
}
