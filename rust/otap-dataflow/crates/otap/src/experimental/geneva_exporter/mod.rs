// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Geneva Exporter for OTAP logs
//!
//! This exporter sends OTAP log data to Microsoft Geneva telemetry backend.
//! It is designed for Microsoft products and implements the `Exporter<OtapPdata>` trait
//! for integration with the OTAP dataflow engine.
//!
//! ## Current Status
//!
//! **Implemented:**
//! - Configuration parsing (serde)
//! - `Exporter<OtapPdata>` trait with async message loop
//! - Distributed slice registration (automatic discovery by df_engine)
//! - Message handling (shutdown, telemetry collection, pdata reception)
//! - Metrics registration
//!
//! **Not Yet Implemented (no-op placeholders):**
//! - Arrow RecordBatch extraction from OtapPdata
//! - Integrating with Geneva Uploader from opentelemetry-rust-contrib
//!
//! ## Usage
//!
//! This exporter is automatically discovered by the `df_engine` binary via `linkme`.
//! Users configure it in YAML:
//!
//! ```yaml
//! nodes:
//!   - id: geneva-exporter
//!     urn: "urn:otel:geneva:exporter"
//!     config:
//!       endpoint: "https://geneva.microsoft.com"
//!       environment: "production"
//!       account: "my-account"
//!       namespace: "my-namespace"
//!       # ... additional config
//! ```

use async_trait::async_trait;
use linkme::distributed_slice;
use otap_df_config::node::NodeUserConfig;
use otap_df_engine::config::ExporterConfig;
use otap_df_engine::context::PipelineContext;
use otap_df_engine::control::NodeControlMsg;
use otap_df_engine::error::Error;
use otap_df_engine::exporter::ExporterWrapper;
use otap_df_engine::local::exporter::{EffectHandler, Exporter};
use otap_df_engine::message::{Message, MessageChannel};
use otap_df_engine::node::NodeId;
use otap_df_engine::terminal_state::TerminalState;
use otap_df_engine::ExporterFactory;
use otap_df_telemetry::metrics::MetricSet;
use serde::Deserialize;
use std::sync::Arc;

// Use crate-relative paths since we're now a module within otap
use crate::pdata::OtapPdata;
use crate::OTAP_EXPORTER_FACTORIES;

/// The URN for the Geneva exporter
pub const GENEVA_EXPORTER_URN: &str = "urn:otel:geneva:exporter";

/// Crate version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Configuration for the Geneva Exporter
#[derive(Debug, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct Config {
    /// Geneva endpoint URL
    pub endpoint: String,
    /// Environment (e.g., "production", "staging")
    pub environment: String,
    /// Geneva account name
    pub account: String,
    /// Geneva namespace
    pub namespace: String,
    /// Azure region
    pub region: String,
    /// Config major version
    #[serde(default = "default_config_version")]
    pub config_major_version: u32,
    /// Tenant name
    pub tenant: String,
    /// Role name
    pub role_name: String,
    /// Role instance identifier
    pub role_instance: String,
    /// Authentication configuration
    pub auth: AuthConfig,
    /// Maximum buffer size before forcing flush (default: 1000)
    #[serde(default = "default_buffer_size")]
    pub max_buffer_size: usize,
    /// Maximum concurrent uploads (default: 4)
    #[serde(default = "default_max_concurrent")]
    pub max_concurrent_uploads: usize,
}

fn default_config_version() -> u32 {
    1
}

fn default_buffer_size() -> usize {
    1000
}

fn default_max_concurrent() -> usize {
    4
}

/// Authentication configuration
#[derive(Debug, Deserialize, Clone)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum AuthConfig {
    /// Certificate-based authentication (PKCS#12 format)
    Certificate {
        /// Path to PKCS#12 (.p12) certificate file
        path: String,
        /// Password to decrypt the PKCS#12 file
        password: String,
    },
    /// System-assigned managed identity
    SystemManagedIdentity {
        /// MSI resource identifier
        msi_resource: String,
    },
    /// User-assigned managed identity (by client ID)
    UserManagedIdentity {
        /// Client ID of the managed identity
        client_id: String,
        /// MSI resource identifier
        msi_resource: String,
    },
    /// Workload identity (Kubernetes)
    WorkloadIdentity {
        /// MSI resource identifier
        msi_resource: String,
    },
}

/// Placeholder metrics (will be replaced with actual metrics)
#[derive(Default, Debug)]
struct ExporterMetrics {
    // Placeholder - will add actual metrics counters later
}

// Implement MetricSetHandler trait for metrics
impl otap_df_telemetry::metrics::MetricSetHandler for ExporterMetrics {
    fn descriptor(&self) -> &'static otap_df_telemetry::descriptor::MetricsDescriptor {
        static DESCRIPTOR: otap_df_telemetry::descriptor::MetricsDescriptor =
            otap_df_telemetry::descriptor::MetricsDescriptor {
                name: "geneva_exporter",
                metrics: &[],
            };
        &DESCRIPTOR
    }

    fn snapshot_values(&self) -> Vec<u64> {
        // No metrics yet
        Vec::new()
    }

    fn clear_values(&mut self) {
        // No-op - no metrics to clear yet
    }

    fn needs_flush(&self) -> bool {
        // No metrics yet, so nothing to flush
        false
    }
}

/// Geneva exporter that sends OTAP data to Geneva backend (scaffold)
pub struct GenevaExporter {
    config: Config,
    #[allow(dead_code)]
    metrics: MetricSet<ExporterMetrics>,
}

impl GenevaExporter {
    /// Create a new Geneva exporter from configuration
    pub fn from_config(
        pipeline_ctx: PipelineContext,
        config: &serde_json::Value,
    ) -> Result<Self, otap_df_config::error::Error> {
        let metrics = pipeline_ctx.register_metrics::<ExporterMetrics>();

        let config: Config = serde_json::from_value(config.clone()).map_err(|e| {
            otap_df_config::error::Error::InvalidUserConfig {
                error: e.to_string(),
            }
        })?;

        Ok(Self { config, metrics })
    }

    /// Get exporter configuration
    pub fn config(&self) -> &Config {
        &self.config
    }
}

/// Register Geneva exporter with the OTAP exporter factory
///
/// Unsafe code is temporarily used here to allow the use of `distributed_slice` macro
/// This macro is part of the `linkme` crate which is considered safe and well maintained.
#[allow(unsafe_code)]
#[distributed_slice(OTAP_EXPORTER_FACTORIES)]
pub static GENEVA_EXPORTER: ExporterFactory<OtapPdata> = ExporterFactory {
    name: GENEVA_EXPORTER_URN,
    create: |pipeline: PipelineContext,
             node: NodeId,
             node_config: Arc<NodeUserConfig>,
             exporter_config: &ExporterConfig| {
        Ok(ExporterWrapper::local(
            GenevaExporter::from_config(pipeline, &node_config.config)?,
            node,
            node_config,
            exporter_config,
        ))
    },
};

#[async_trait(?Send)]
impl Exporter<OtapPdata> for GenevaExporter {
    async fn start(
        mut self: Box<Self>,
        mut msg_chan: MessageChannel<OtapPdata>,
        effect_handler: EffectHandler<OtapPdata>,
    ) -> Result<TerminalState, Error> {
        effect_handler
            .info(&format!(
                "Geneva exporter starting (no-op scaffold): endpoint={}, namespace={}",
                self.config.endpoint, self.config.namespace
            ))
            .await;

        // No-op message loop - just handles shutdown
        loop {
            match msg_chan.recv().await? {
                Message::Control(NodeControlMsg::Shutdown { deadline, .. }) => {
                    effect_handler
                        .info("Geneva exporter shutting down (no-op scaffold)")
                        .await;

                    return Ok(TerminalState::new(deadline, [self.metrics]));
                }
                Message::Control(NodeControlMsg::CollectTelemetry {
                    mut metrics_reporter,
                }) => {
                    _ = metrics_reporter.report(&mut self.metrics);
                }
                Message::PData(_pdata) => {
                    // No-op: In real implementation, this would:
                    // 1. Extract OTAP Arrow batches
                    // 2. Encode to Geneva Bond format
                    // 3. Compress with LZ4
                    // 4. Upload to Geneva
                    effect_handler
                        .info("Geneva exporter received PData (no-op - discarded)")
                        .await;
                }
                _ => {
                    // Ignore other messages
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_deserialization() {
        let json = serde_json::json!({
            "endpoint": "https://geneva.example.com",
            "environment": "production",
            "account": "test-account",
            "namespace": "test-namespace",
            "region": "westus2",
            "tenant": "test-tenant",
            "role_name": "test-role",
            "role_instance": "test-instance",
            "auth": {
                "type": "certificate",
                "path": "/path/to/cert.p12",
                "password": "secret"
            }
        });

        let config: Config = serde_json::from_value(json).unwrap();
        assert_eq!(config.endpoint, "https://geneva.example.com");
        assert_eq!(config.environment, "production");
        assert_eq!(config.config_major_version, 1); // default
        assert_eq!(config.max_buffer_size, 1000); // default
    }

    #[test]
    fn test_auth_config_variants() {
        let cert_json = serde_json::json!({
            "type": "certificate",
            "path": "/path/to/cert.p12",
            "password": "secret"
        });
        let _cert_auth: AuthConfig = serde_json::from_value(cert_json).unwrap();

        let system_mi_json = serde_json::json!({
            "type": "systemmanagedidentity",
            "msi_resource": "https://resource"
        });
        let _system_mi: AuthConfig = serde_json::from_value(system_mi_json).unwrap();
    }

    #[test]
    fn test_urn_constant() {
        assert_eq!(GENEVA_EXPORTER_URN, "urn:otel:geneva:exporter");
    }
}
