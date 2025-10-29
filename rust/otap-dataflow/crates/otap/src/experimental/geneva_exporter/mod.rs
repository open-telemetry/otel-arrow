// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Geneva Exporter for OTAP logs and traces
//!
//! This exporter sends OTAP log and trace data to Microsoft Geneva telemetry backend.
//! It is designed for Microsoft products and implements the `Exporter<OtapPdata>` trait
//! for integration with the OTAP dataflow engine.
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
use otap_df_engine::ExporterFactory;
use otap_df_engine::config::ExporterConfig;
use otap_df_engine::context::PipelineContext;
use otap_df_engine::control::NodeControlMsg;
use otap_df_engine::error::Error;
use otap_df_engine::exporter::ExporterWrapper;
use otap_df_engine::local::exporter::{EffectHandler, Exporter};
use otap_df_engine::message::{Message, MessageChannel};
use otap_df_engine::node::NodeId;
use otap_df_engine::terminal_state::TerminalState;
use otap_df_telemetry::metrics::MetricSet;
use otap_df_telemetry_macros::metric_set;
use serde::Deserialize;
use std::path::PathBuf;
use std::sync::Arc;

// Geneva uploader dependencies
use geneva_uploader::AuthMethod;
use geneva_uploader::client::{GenevaClient, GenevaClientConfig};
use opentelemetry_proto::tonic::collector::logs::v1::ExportLogsServiceRequest;
use opentelemetry_proto::tonic::collector::trace::v1::ExportTraceServiceRequest;
use prost::Message as ProstMessage;

// Use crate-relative paths since we're now a module within otap
use crate::OTAP_EXPORTER_FACTORIES;
use crate::pdata::{OtapPdata, OtlpProtoBytes};

/// The URN for the Geneva exporter
pub const GENEVA_EXPORTER_URN: &str = "urn:otel:geneva:exporter";

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
    /// Config major version (required)
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

const fn default_buffer_size() -> usize {
    1000
}

const fn default_max_concurrent() -> usize {
    4
}

/// Authentication configuration
/// TODO - see if we directly use AuthMethod from geneva-uploader crate
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

/// Geneva exporter metrics.
/// Grouped under `otap.exporter.geneva`.
#[metric_set(name = "otap.exporter.geneva")]
#[derive(Debug, Default, Clone)]
struct ExporterMetrics {
    // TODO: Add actual metrics counters later
    // Examples:
    // - batches_uploaded: Counter<u64>
    // - batches_failed: Counter<u64>
    // - bytes_sent: Counter<u64>
}

/// Geneva exporter that sends OTAP data to Geneva backend
pub struct GenevaExporter {
    config: Config,
    #[allow(dead_code)]
    metrics: MetricSet<ExporterMetrics>,
    geneva_client: GenevaClient,
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

        // Convert AuthConfig to AuthMethod
        let auth_method = match &config.auth {
            AuthConfig::Certificate { path, password } => AuthMethod::Certificate {
                path: PathBuf::from(path),
                password: password.clone(),
            },
            AuthConfig::SystemManagedIdentity { .. } => AuthMethod::SystemManagedIdentity,
            AuthConfig::UserManagedIdentity { client_id, .. } => AuthMethod::UserManagedIdentity {
                client_id: client_id.clone(),
            },
            AuthConfig::WorkloadIdentity { msi_resource } => AuthMethod::WorkloadIdentity {
                resource: msi_resource.clone(),
            },
        };

        // Get MSI resource if needed for managed identity
        let msi_resource = match &config.auth {
            AuthConfig::SystemManagedIdentity { msi_resource }
            | AuthConfig::UserManagedIdentity { msi_resource, .. }
            | AuthConfig::WorkloadIdentity { msi_resource } => Some(msi_resource.clone()),
            AuthConfig::Certificate { .. } => None,
        };

        // Create GenevaClient configuration
        let client_config = GenevaClientConfig {
            endpoint: config.endpoint.clone(),
            environment: config.environment.clone(),
            account: config.account.clone(),
            namespace: config.namespace.clone(),
            region: config.region.clone(),
            config_major_version: config.config_major_version,
            auth_method,
            tenant: config.tenant.clone(),
            role_name: config.role_name.clone(),
            role_instance: config.role_instance.clone(),
            msi_resource,
        };

        // Initialize Geneva client
        let geneva_client = GenevaClient::new(client_config).map_err(|e| {
            otap_df_config::error::Error::InvalidUserConfig {
                error: format!("Failed to initialize Geneva client: {}", e),
            }
        })?;

        Ok(Self {
            config,
            metrics,
            geneva_client,
        })
    }

    /// Get exporter configuration
    #[must_use]
    pub fn config(&self) -> &Config {
        &self.config
    }

    /// Handle PData message by converting OTAP to OTLP and uploading to Geneva
    ///
    /// TODO: Ideal Approach - Geneva uploader should accept OTAP directly
    ///
    /// Current implementation converts OTAP → OTLP → Geneva format, which has overhead:
    /// - OTAP (Arrow RecordBatch) → OTLP (protobuf) conversion step
    /// - OTLP (protobuf) → Geneva format conversion
    ///
    /// Ideal implementation would be:
    /// - OTAP (Arrow RecordBatch) → Geneva format directly
    ///
    /// This requires modifying the geneva-uploader crate to accept Arrow RecordBatch
    /// and process it directly, rather than going through OTLP ResourceLogs/ResourceSpans.
    async fn handle_pdata(
        &self,
        pdata: OtapPdata,
        effect_handler: &EffectHandler<OtapPdata>,
    ) -> Result<(), String> {
        // Split pdata into context and payload
        let (_context, payload) = pdata.into_parts();

        // Convert OTAP payload to OTLP bytes
        // TODO: This conversion step should be eliminated (see method documentation above)
        let otlp_bytes: OtlpProtoBytes = payload
            .try_into()
            .map_err(|e| format!("Failed to convert OTAP to OTLP: {:?}", e))?;

        // Process based on signal type
        match otlp_bytes {
            OtlpProtoBytes::ExportLogsRequest(bytes) => {
                effect_handler
                    .info("Converting and uploading logs to Geneva")
                    .await;

                // Decode OTLP bytes to ResourceLogs
                let logs_request = ExportLogsServiceRequest::decode(&bytes[..])
                    .map_err(|e| format!("Failed to decode logs request: {}", e))?;

                // Encode and compress using Geneva client
                let batches = self
                    .geneva_client
                    .encode_and_compress_logs(&logs_request.resource_logs)
                    .map_err(|e| format!("Failed to encode logs: {}", e))?;

                // TODO: This is sequential batch upload.
                // Consider revisiting to implementing concurrent uploads
                // Upload each batch
                for batch in batches {
                    self.geneva_client
                        .upload_batch(&batch)
                        .await
                        .map_err(|e| format!("Failed to upload log batch: {}", e))?;
                }

                effect_handler
                    .info(&format!(
                        "Successfully uploaded {} log batches to Geneva",
                        logs_request.resource_logs.len()
                    ))
                    .await;
            }
            OtlpProtoBytes::ExportTracesRequest(bytes) => {
                effect_handler
                    .info("Converting and uploading traces to Geneva")
                    .await;

                // Decode OTLP bytes to ResourceSpans
                let traces_request = ExportTraceServiceRequest::decode(&bytes[..])
                    .map_err(|e| format!("Failed to decode traces request: {}", e))?;

                // Encode and compress using Geneva client
                let batches = self
                    .geneva_client
                    .encode_and_compress_spans(&traces_request.resource_spans)
                    .map_err(|e| format!("Failed to encode spans: {}", e))?;

                // TODO: This is sequential batch upload.
                // Consider revisiting to implementing concurrent uploads
                // Upload each batch
                for batch in batches {
                    self.geneva_client
                        .upload_batch(&batch)
                        .await
                        .map_err(|e| format!("Failed to upload trace batch: {}", e))?;
                }

                effect_handler
                    .info(&format!(
                        "Successfully uploaded {} trace batches to Geneva",
                        traces_request.resource_spans.len()
                    ))
                    .await;
            }
            OtlpProtoBytes::ExportMetricsRequest(_) => {
                // Geneva exporter does not support metrics
                return Err("Geneva exporter does not support metrics signal".to_string());
            }
        }

        Ok(())
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
                "Geneva exporter starting: endpoint={}, namespace={}, account={}",
                self.config.endpoint, self.config.namespace, self.config.account
            ))
            .await;

        // Message loop
        loop {
            match msg_chan.recv().await? {
                Message::Control(NodeControlMsg::Shutdown { deadline, .. }) => {
                    effect_handler.info("Geneva exporter shutting down").await;

                    return Ok(TerminalState::new(deadline, [self.metrics]));
                }
                Message::Control(NodeControlMsg::CollectTelemetry {
                    mut metrics_reporter,
                }) => {
                    _ = metrics_reporter.report(&mut self.metrics);
                }
                Message::PData(pdata) => {
                    // Convert OTAP to OTLP and upload to Geneva
                    if let Err(e) = self.handle_pdata(pdata, &effect_handler).await {
                        effect_handler
                            .info(&format!("ERROR: Failed to export to Geneva: {}", e))
                            .await;
                    }
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
            "config_major_version": 1,
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

        // Assert all config fields
        assert_eq!(config.endpoint, "https://geneva.example.com");
        assert_eq!(config.environment, "production");
        assert_eq!(config.account, "test-account");
        assert_eq!(config.namespace, "test-namespace");
        assert_eq!(config.region, "westus2");
        assert_eq!(config.config_major_version, 1);
        assert_eq!(config.tenant, "test-tenant");
        assert_eq!(config.role_name, "test-role");
        assert_eq!(config.role_instance, "test-instance");
        assert_eq!(config.max_buffer_size, 1000); // default
        assert_eq!(config.max_concurrent_uploads, 4); // default

        // Assert auth config
        match config.auth {
            AuthConfig::Certificate { path, password } => {
                assert_eq!(path, "/path/to/cert.p12");
                assert_eq!(password, "secret");
            }
            _ => panic!("Expected Certificate auth variant"),
        }
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
