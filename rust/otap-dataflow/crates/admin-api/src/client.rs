// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Public admin client facade and resource clients.

use crate::endpoint::{AdminAuth, AdminEndpoint};
use crate::http_backend::HttpBackend;
use crate::{Error, engine, operations, pipeline_groups, pipelines, telemetry};
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
    /// Creates new HTTP client settings.
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

    /// Sets the auth mode.
    #[must_use]
    pub fn with_auth(mut self, auth: AdminAuth) -> Self {
        self.auth = auth;
        self
    }

    /// Sets the TCP connect timeout.
    #[must_use]
    pub fn with_connect_timeout(mut self, connect_timeout: Duration) -> Self {
        self.connect_timeout = connect_timeout;
        self
    }

    /// Sets the request timeout.
    #[must_use]
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }

    /// Clears any request timeout.
    #[must_use]
    pub fn without_timeout(mut self) -> Self {
        self.timeout = None;
        self
    }

    /// Sets whether to enable `TCP_NODELAY`.
    #[must_use]
    pub fn with_tcp_nodelay(mut self, tcp_nodelay: bool) -> Self {
        self.tcp_nodelay = tcp_nodelay;
        self
    }

    /// Sets the TCP keepalive timeout.
    #[must_use]
    pub fn with_tcp_keepalive(mut self, tcp_keepalive: Option<Duration>) -> Self {
        self.tcp_keepalive = tcp_keepalive;
        self
    }

    /// Sets the TCP keepalive probe interval.
    #[must_use]
    pub fn with_tcp_keepalive_interval(mut self, tcp_keepalive_interval: Option<Duration>) -> Self {
        self.tcp_keepalive_interval = tcp_keepalive_interval;
        self
    }

    /// Sets the TLS or mTLS configuration for HTTPS endpoints.
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
    /// Creates a new builder.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Configures the client to use the HTTP admin backend.
    #[must_use]
    pub fn http(mut self, settings: HttpAdminClientSettings) -> Self {
        self.backend = Some(BackendConfig::Http(settings));
        self
    }

    /// Builds the configured admin client.
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
    /// Creates a new client builder.
    #[must_use]
    pub fn builder() -> AdminClientBuilder {
        AdminClientBuilder::new()
    }

    /// Returns the engine-scoped resource client.
    #[must_use]
    pub fn engine(&self) -> EngineClient<'_> {
        EngineClient {
            backend: self.backend.as_ref(),
        }
    }

    /// Returns the pipeline-group-scoped resource client.
    #[must_use]
    pub fn pipeline_groups(&self) -> PipelineGroupsClient<'_> {
        PipelineGroupsClient {
            backend: self.backend.as_ref(),
        }
    }

    /// Returns the pipeline-scoped resource client.
    #[must_use]
    pub fn pipelines(&self) -> PipelinesClient<'_> {
        PipelinesClient {
            backend: self.backend.as_ref(),
        }
    }

    /// Returns the telemetry-scoped resource client.
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
    /// Returns global pipeline status.
    pub async fn status(&self) -> Result<engine::Status, Error> {
        self.backend.engine_status().await
    }

    /// Returns the global liveness probe response.
    pub async fn livez(&self) -> Result<engine::ProbeResponse, Error> {
        self.backend.engine_livez().await
    }

    /// Returns the global readiness probe response.
    pub async fn readyz(&self) -> Result<engine::ProbeResponse, Error> {
        self.backend.engine_readyz().await
    }
}

/// Pipeline-group-scoped admin client.
#[derive(Clone, Copy)]
pub struct PipelineGroupsClient<'a> {
    backend: &'a dyn AdminBackend,
}

impl PipelineGroupsClient<'_> {
    /// Returns pipeline-group status.
    pub async fn status(&self) -> Result<pipeline_groups::Status, Error> {
        self.backend.pipeline_groups_status().await
    }

    /// Requests shutdown for all pipelines.
    pub async fn shutdown(
        &self,
        options: &operations::OperationOptions,
    ) -> Result<pipeline_groups::ShutdownResponse, Error> {
        self.backend.pipeline_groups_shutdown(options).await
    }
}

/// Pipeline-scoped admin client.
#[derive(Clone, Copy)]
pub struct PipelinesClient<'a> {
    backend: &'a dyn AdminBackend,
}

impl PipelinesClient<'_> {
    /// Returns status for one pipeline.
    pub async fn status(
        &self,
        pipeline_group_id: &str,
        pipeline_id: &str,
    ) -> Result<Option<pipelines::Status>, Error> {
        self.backend
            .pipeline_status(pipeline_group_id, pipeline_id)
            .await
    }

    /// Returns the liveness probe for one pipeline.
    pub async fn livez(
        &self,
        pipeline_group_id: &str,
        pipeline_id: &str,
    ) -> Result<pipelines::ProbeResult, Error> {
        self.backend
            .pipeline_livez(pipeline_group_id, pipeline_id)
            .await
    }

    /// Returns the readiness probe for one pipeline.
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
    /// Returns retained logs or `None` when the logs endpoint is unavailable.
    pub async fn logs(
        &self,
        query: &telemetry::LogsQuery,
    ) -> Result<Option<telemetry::LogsResponse>, Error> {
        self.backend.telemetry_logs(query).await
    }

    /// Returns full structured metrics.
    pub async fn metrics(
        &self,
        options: &telemetry::MetricsOptions,
    ) -> Result<telemetry::MetricsResponse, Error> {
        self.backend.telemetry_metrics(options).await
    }

    /// Returns compact structured metrics.
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

    async fn pipeline_groups_status(&self) -> Result<pipeline_groups::Status, Error>;
    async fn pipeline_groups_shutdown(
        &self,
        options: &operations::OperationOptions,
    ) -> Result<pipeline_groups::ShutdownResponse, Error>;

    async fn pipeline_status(
        &self,
        pipeline_group_id: &str,
        pipeline_id: &str,
    ) -> Result<Option<pipelines::Status>, Error>;
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
