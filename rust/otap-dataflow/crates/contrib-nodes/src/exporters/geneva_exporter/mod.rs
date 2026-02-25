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
//!     urn: "urn:microsoft:geneva:exporter"
//!     config:
//!       endpoint: "https://geneva.microsoft.com"
//!       environment: "production"
//!       account: "my-account"
//!       namespace: "my-namespace"
//!       # ... additional config
//! ```

use async_trait::async_trait;
use linkme::distributed_slice;
use otap_df_config::SignalType;
use otap_df_config::node::NodeUserConfig;
use otap_df_engine::ConsumerEffectHandlerExtension;
use otap_df_engine::ExporterFactory;
use otap_df_engine::config::ExporterConfig;
use otap_df_engine::context::PipelineContext;
use otap_df_engine::control::NodeControlMsg;
use otap_df_engine::control::{AckMsg, NackMsg};
use otap_df_engine::error::Error;
use otap_df_engine::exporter::ExporterWrapper;
use otap_df_engine::local::exporter::{EffectHandler, Exporter};
use otap_df_engine::message::{Message, MessageChannel};
use otap_df_engine::node::NodeId;
use otap_df_engine::terminal_state::TerminalState;
use otap_df_pdata::otlp::OtlpProtoBytes;
// Zero-copy view import (currently unused, for future optimization)
// use otap_df_pdata::views::otap::OtapLogsView;
use otap_df_pdata::{OtapArrowRecords, OtapPayload};
use otap_df_telemetry::instrument::Counter;
use otap_df_telemetry::metrics::MetricSet;
use otap_df_telemetry::otel_info;
use otap_df_telemetry_macros::metric_set;
use serde::Deserialize;
use std::path::PathBuf;
use std::sync::Arc;

// Geneva uploader dependencies
use futures::StreamExt;
use futures::stream::TryStreamExt;
use geneva_uploader::AuthMethod;
use geneva_uploader::client::{EncodedBatch, GenevaClient, GenevaClientConfig};
use opentelemetry_proto::tonic::collector::logs::v1::ExportLogsServiceRequest;
use opentelemetry_proto::tonic::collector::trace::v1::ExportTraceServiceRequest;
use prost::Message as ProstMessage;

// Use crate-relative paths since we're now a module within otap
use otap_df_otap::OTAP_EXPORTER_FACTORIES;
use otap_df_otap::metrics::ExporterPDataMetrics;
use otap_df_otap::pdata::OtapPdata;

/// The URN for the Geneva exporter
pub const GENEVA_EXPORTER_URN: &str = "urn:microsoft:geneva:exporter";

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
    /// Note: This field is currently reserved for future use and does not affect runtime behavior.
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
    /// Total number of compressed batches produced by the encoder.
    #[metric(unit = "{batch}")]
    pub batches_encoded: Counter<u64>,

    /// Total number of batches considered successfully uploaded.
    #[metric(unit = "{batch}")]
    pub batches_uploaded: Counter<u64>,
}

/// Geneva exporter that sends OTAP data to Geneva backend
pub struct GenevaExporter {
    config: Config,
    pdata_metrics: MetricSet<ExporterPDataMetrics>,
    metrics: MetricSet<ExporterMetrics>,
    geneva_client: GenevaClient,
}

impl GenevaExporter {
    /// Create a new Geneva exporter from configuration
    pub fn from_config(
        pipeline_ctx: PipelineContext,
        config: &serde_json::Value,
    ) -> Result<Self, otap_df_config::error::Error> {
        let pdata_metrics = pipeline_ctx.register_metrics::<ExporterPDataMetrics>();
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
            pdata_metrics,
            metrics,
            geneva_client,
        })
    }

    /// Get exporter configuration
    #[must_use]
    pub fn config(&self) -> &Config {
        &self.config
    }

    /// Upload batches concurrently
    async fn upload_batches_concurrent(
        &mut self,
        batches: &[EncodedBatch],
        signal_type: SignalType,
    ) -> Result<usize, String> {
        let batches_encoded = batches.len();
        self.metrics.batches_encoded.add(batches_encoded as u64);

        let max_concurrent = self.config.max_concurrent_uploads.max(1);
        let client = &self.geneva_client;

        futures::stream::iter(batches.iter())
            .map(Ok::<_, String>)
            .try_for_each_concurrent(max_concurrent, |batch| async move {
                client
                    .upload_batch(batch)
                    .await
                    .map_err(|e| format!("Failed to upload {:?} batch: {e}", signal_type))
            })
            .await?;

        self.metrics.batches_uploaded.add(batches_encoded as u64);
        Ok(batches_encoded)
    }

    /// Handle PData message with dual-path encoding
    ///
    /// Supports two data paths for Geneva encoding:
    /// - **Zero-copy path**: OTAP Arrow RecordBatch → Geneva (via LogsDataView)
    ///   Avoids protobuf deserialization by iterating directly over Arrow columns
    ///   Used when data flows through batch processor (converts OTLP → OTAP) or syslog receiver
    /// - **Fallback path**: OTLP bytes → Geneva (protobuf decoding)
    ///   Used when OTLP receiver connects directly to Geneva exporter (no batch processor)
    ///   Deserializes OTLP protobuf into structs before encoding
    async fn export_payload(
        &mut self,
        payload: OtapPayload,
        _effect_handler: &EffectHandler<OtapPdata>,
    ) -> Result<usize, String> {
        if payload.is_empty() {
            // Empty payloads are a no-op but should still be acked.
            // Avoids unnecessary encoding/upload work.
            otel_info!(
                "geneva_exporter.skip",
                message = "Geneva exporter skipping empty payload"
            );
            return Ok(0);
        }

        // Handle based on payload type
        match payload {
            // OTAP Arrow path: Convert OTAP → OTLP bytes → deserialize → use existing Geneva client methods
            OtapPayload::OtapArrowRecords(otap_records) => {
                match otap_records {
                    OtapArrowRecords::Logs(otap_records) => {
                        // TODO: Zero-copy view path for future optimization
                        // Currently commented to keep behavior consistent with main branch
                        //
                        // effect_handler
                        //     .info("Uploading logs to Geneva using zero-copy views")
                        //     .await;
                        // let logs_view = OtapLogsView::try_from(&otap_records)
                        //     .map_err(|e| format!("Failed to build logs view: {}", e))?;
                        // let batches = self
                        //     .geneva_client
                        //     .encode_and_compress_logs_view(&logs_view)
                        //     .map_err(|e| format!("Failed to encode logs from view: {}", e))?;

                        // Fallback path: Convert OTAP Arrow → OTLP bytes
                        otel_info!(
                            "geneva_exporter.convert",
                            message = "Converting OTAP logs to OTLP bytes (fallback path)"
                        );

                        let otlp_bytes: OtlpProtoBytes =
                            OtapPayload::OtapArrowRecords(OtapArrowRecords::Logs(otap_records))
                                .try_into()
                                .map_err(|e| format!("Failed to convert OTAP to OTLP: {:?}", e))?;

                        let OtlpProtoBytes::ExportLogsRequest(bytes) = otlp_bytes else {
                            return Err("Expected logs but got different signal type".to_string());
                        };

                        // Decode OTLP bytes to ResourceLogs
                        let logs_request = ExportLogsServiceRequest::decode(&bytes[..])
                            .map_err(|e| format!("Failed to decode logs request: {}", e))?;

                        // Encode and compress using Geneva client
                        let batches = self
                            .geneva_client
                            .encode_and_compress_logs(&logs_request.resource_logs)
                            .map_err(|e| format!("Failed to encode logs: {}", e))?;

                        let batches_uploaded = self
                            .upload_batches_concurrent(&batches, SignalType::Logs)
                            .await?;

                        otel_info!(
                            "geneva_exporter.upload",
                            count = batches_uploaded,
                            message = "Successfully uploaded log batches to Geneva (OTAP fallback)"
                        );

                        Ok(batches_uploaded)
                    }
                    OtapArrowRecords::Traces(otap_records) => {
                        // TODO: Zero-copy view path for future optimization (when TracesView is ready)

                        // Fallback path: Convert OTAP Arrow → OTLP bytes
                        otel_info!(
                            "geneva_exporter.convert",
                            message = "Converting OTAP traces to OTLP bytes (fallback path)"
                        );

                        let otlp_bytes: OtlpProtoBytes =
                            OtapPayload::OtapArrowRecords(OtapArrowRecords::Traces(otap_records))
                                .try_into()
                                .map_err(|e| format!("Failed to convert OTAP to OTLP: {:?}", e))?;

                        let OtlpProtoBytes::ExportTracesRequest(bytes) = otlp_bytes else {
                            return Err("Expected traces but got different signal type".to_string());
                        };

                        // Decode OTLP bytes to ResourceSpans
                        let traces_request = ExportTraceServiceRequest::decode(&bytes[..])
                            .map_err(|e| format!("Failed to decode traces request: {}", e))?;

                        // Encode and compress using Geneva client
                        let batches = self
                            .geneva_client
                            .encode_and_compress_spans(&traces_request.resource_spans)
                            .map_err(|e| format!("Failed to encode spans: {}", e))?;

                        let batches_uploaded = self
                            .upload_batches_concurrent(&batches, SignalType::Traces)
                            .await?;

                        otel_info!(
                            "geneva_exporter.upload",
                            count = batches_uploaded,
                            message =
                                "Successfully uploaded trace batches to Geneva (OTAP fallback)"
                        );

                        Ok(batches_uploaded)
                    }
                    OtapArrowRecords::Metrics(_) => {
                        Err("Geneva exporter does not support metrics signal".to_string())
                    }
                }
            }

            // OTLP path: Direct OTLP bytes from receivers without OTAP conversion (e.g., OTLP receiver → Geneva exporter without batch processor)
            OtapPayload::OtlpBytes(otlp_bytes) => {
                match otlp_bytes {
                    OtlpProtoBytes::ExportLogsRequest(bytes) => {
                        otel_info!(
                            "geneva_exporter.upload",
                            message = "Uploading logs to Geneva using OTLP path"
                        );

                        // Decode OTLP bytes to ResourceLogs
                        let logs_request = ExportLogsServiceRequest::decode(&bytes[..])
                            .map_err(|e| format!("Failed to decode logs request: {}", e))?;

                        // Encode and compress using Geneva client
                        let batches = self
                            .geneva_client
                            .encode_and_compress_logs(&logs_request.resource_logs)
                            .map_err(|e| format!("Failed to encode logs: {}", e))?;

                        let batches_uploaded = self
                            .upload_batches_concurrent(&batches, SignalType::Logs)
                            .await?;

                        otel_info!(
                            "geneva_exporter.upload",
                            count = batches_uploaded,
                            message = "Successfully uploaded log batches to Geneva (OTLP path)"
                        );

                        Ok(batches_uploaded)
                    }
                    OtlpProtoBytes::ExportTracesRequest(bytes) => {
                        otel_info!(
                            "geneva_exporter.upload",
                            message = "Uploading traces to Geneva using OTLP path"
                        );

                        // Decode OTLP bytes to ResourceSpans
                        let traces_request = ExportTraceServiceRequest::decode(&bytes[..])
                            .map_err(|e| format!("Failed to decode traces request: {}", e))?;

                        // Encode and compress using Geneva client
                        let batches = self
                            .geneva_client
                            .encode_and_compress_spans(&traces_request.resource_spans)
                            .map_err(|e| format!("Failed to encode spans: {}", e))?;

                        let batches_uploaded = self
                            .upload_batches_concurrent(&batches, SignalType::Traces)
                            .await?;

                        otel_info!(
                            "geneva_exporter.upload",
                            count = batches_uploaded,
                            message = "Successfully uploaded trace batches to Geneva (OTLP path)"
                        );

                        Ok(batches_uploaded)
                    }
                    OtlpProtoBytes::ExportMetricsRequest(_) => {
                        Err("Geneva exporter does not support metrics signal".to_string())
                    }
                }
            }
        }
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
    wiring_contract: otap_df_engine::wiring_contract::WiringContract::UNRESTRICTED,
    validate_config: otap_df_config::validation::validate_typed_config::<Config>,
};

#[async_trait(?Send)]
impl Exporter<OtapPdata> for GenevaExporter {
    async fn start(
        mut self: Box<Self>,
        mut msg_chan: MessageChannel<OtapPdata>,
        effect_handler: EffectHandler<OtapPdata>,
    ) -> Result<TerminalState, Error> {
        otel_info!(
            "geneva_exporter.start",
            endpoint = self.config.endpoint,
            namespace = self.config.namespace,
            account = self.config.account,
            message = "Geneva exporter starting"
        );

        // Message loop
        loop {
            match msg_chan.recv().await? {
                Message::Control(NodeControlMsg::Shutdown { deadline, .. }) => {
                    otel_info!(
                        "geneva_exporter.shutdown",
                        message = "Geneva exporter shutting down"
                    );

                    return Ok(TerminalState::new(
                        deadline,
                        [self.pdata_metrics.snapshot(), self.metrics.snapshot()],
                    ));
                }
                Message::Control(NodeControlMsg::CollectTelemetry {
                    mut metrics_reporter,
                }) => {
                    _ = metrics_reporter.report(&mut self.pdata_metrics);
                    _ = metrics_reporter.report(&mut self.metrics);
                }
                Message::PData(pdata) => {
                    let (context, payload) = pdata.into_parts();
                    let signal_type = payload.signal_type();
                    self.pdata_metrics.inc_consumed(signal_type);

                    let saved_payload = if context.may_return_payload() {
                        payload.clone()
                    } else {
                        OtapPayload::empty(signal_type)
                    };

                    match self.export_payload(payload, &effect_handler).await {
                        Ok(_batches_uploaded) => {
                            self.pdata_metrics.inc_exported(signal_type);
                            effect_handler
                                .notify_ack(AckMsg::new(OtapPdata::new(context, saved_payload)))
                                .await?;
                        }
                        Err(e) => {
                            self.pdata_metrics.inc_failed(signal_type);
                            otel_info!(
                                "geneva_exporter.error",
                                error = e,
                                message = "Failed to export to Geneva"
                            );
                            effect_handler
                                .notify_nack(NackMsg::new(
                                    &e,
                                    OtapPdata::new(context, saved_payload),
                                ))
                                .await?;
                        }
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
    use serde_json;

    use arrow::array::{
        ArrayRef, Int32Array, RecordBatch, StringArray, StructArray, TimestampNanosecondArray,
        UInt16Array, UInt32Array,
    };
    use arrow::datatypes::{DataType, Field, Schema, TimeUnit};
    use std::sync::Arc;

    use bytes::Bytes;
    use otap_df_engine::Interests;
    use otap_df_engine::control::PipelineControlMsg;
    use otap_df_engine::testing::exporter::{TestRuntime, create_exporter_from_factory};
    use otap_df_otap::testing::TestCallData;
    use std::time::{Duration, Instant};

    // TODO: Re-enable these imports when zero-copy view tests are uncommented
    // use otap_df_pdata::otap::OtapArrowRecords;
    // use otap_df_pdata::proto::opentelemetry::arrow::v1::ArrowPayloadType;
    // use otap_df_pdata::views::logs::{LogsDataView, ResourceLogsView, ScopeLogsView};
    // use otap_df_pdata::views::otap::OtapLogsView;

    // TODO: Re-enable when zero-copy view tests are uncommented
    /// Helper to create a simple OTAP logs RecordBatch for testing Geneva exporter
    #[allow(dead_code)]
    fn create_test_logs_batch() -> RecordBatch {
        // Define schema matching OTAP logs structure
        let resource_field = Field::new(
            "resource",
            DataType::Struct(vec![Field::new("id", DataType::UInt16, false)].into()),
            false,
        );

        let scope_field = Field::new(
            "scope",
            DataType::Struct(vec![Field::new("id", DataType::UInt16, false)].into()),
            false,
        );

        let schema = Arc::new(Schema::new(vec![
            Field::new("id", DataType::UInt16, false),
            resource_field,
            scope_field,
            Field::new(
                "time_unix_nano",
                DataType::Timestamp(TimeUnit::Nanosecond, None),
                true,
            ),
            Field::new(
                "observed_time_unix_nano",
                DataType::Timestamp(TimeUnit::Nanosecond, None),
                true,
            ),
            Field::new("severity_number", DataType::Int32, true),
            Field::new("severity_text", DataType::Utf8, true),
            Field::new("body", DataType::Utf8, true),
            Field::new("flags", DataType::UInt32, true),
            Field::new("event_name", DataType::Utf8, true),
        ]));

        // Create test data (3 log records)
        let id_array = UInt16Array::from(vec![1, 2, 3]);

        // Resource structs (all from resource_id=1)
        let resource_id_array = UInt16Array::from(vec![1, 1, 1]);
        let resource_struct = StructArray::from(vec![(
            Arc::new(Field::new("id", DataType::UInt16, false)),
            Arc::new(resource_id_array) as ArrayRef,
        )]);

        // Scope structs (logs 1-2 from scope_id=10, log 3 from scope_id=11)
        let scope_id_array = UInt16Array::from(vec![10, 10, 11]);
        let scope_struct = StructArray::from(vec![(
            Arc::new(Field::new("id", DataType::UInt16, false)),
            Arc::new(scope_id_array) as ArrayRef,
        )]);

        let time_array = TimestampNanosecondArray::from(vec![
            Some(1000000000),
            Some(2000000000),
            Some(3000000000),
        ]);

        let observed_time_array = TimestampNanosecondArray::from(vec![
            Some(1000000100),
            Some(2000000100),
            Some(3000000100),
        ]);

        let severity_array = Int32Array::from(vec![Some(9), Some(17), Some(13)]); // INFO, ERROR, WARN
        let severity_text_array =
            StringArray::from(vec![Some("INFO"), Some("ERROR"), Some("WARN")]);

        let body_array = StringArray::from(vec![
            Some("Log message 1"),
            Some("Error occurred"),
            Some("Warning message"),
        ]);

        let flags_array = UInt32Array::from(vec![Some(1), Some(1), Some(0)]);
        let event_name_array =
            StringArray::from(vec![Some("event1"), Some("event2"), Some("event3")]);

        RecordBatch::try_new(
            schema,
            vec![
                Arc::new(id_array),
                Arc::new(resource_struct),
                Arc::new(scope_struct),
                Arc::new(time_array),
                Arc::new(observed_time_array),
                Arc::new(severity_array),
                Arc::new(severity_text_array),
                Arc::new(body_array),
                Arc::new(flags_array),
                Arc::new(event_name_array),
            ],
        )
        .expect("Failed to create test logs batch")
    }

    fn test_config() -> serde_json::Value {
        serde_json::json!({
            "endpoint": "https://localhost",
            "environment": "test",
            "account": "test-account",
            "namespace": "test-namespace",
            "region": "test-region",
            "config_major_version": 1,
            "tenant": "test-tenant",
            "role_name": "test-role",
            "role_instance": "test-instance",
            "auth": {
                "type": "systemmanagedidentity",
                "msi_resource": "https://example.com"
            },
            "max_buffer_size": 1000,
            "max_concurrent_uploads": 2
        })
    }

    #[test]
    fn geneva_exporter_emits_ack_for_empty_payload() {
        let test_runtime = TestRuntime::new();
        let exporter = create_exporter_from_factory(&GENEVA_EXPORTER, test_config()).unwrap();

        test_runtime
            .set_exporter(exporter)
            .run_test(|ctx| async move {
                let payload: OtapPayload = OtlpProtoBytes::ExportLogsRequest(Bytes::new()).into();
                let pdata = OtapPdata::new_default(payload).test_subscribe_to(
                    Interests::ACKS,
                    TestCallData::default().into(),
                    4242,
                );
                ctx.send_pdata(pdata).await.unwrap();
                ctx.send_shutdown(Instant::now() + Duration::from_secs(1), "test shutdown")
                    .await
                    .unwrap();
            })
            .run_validation(|mut ctx, result| async move {
                result.expect("success");

                let mut pipeline_rx = ctx.take_pipeline_ctrl_receiver().unwrap();
                match pipeline_rx.recv().await.unwrap() {
                    PipelineControlMsg::DeliverAck { ack, node_id } => {
                        assert_eq!(node_id, 4242);
                        let got: TestCallData = ack.calldata.try_into().unwrap();
                        assert_eq!(TestCallData::default(), got);
                        assert_eq!(ack.accepted.num_items(), 0);
                    }
                    other => panic!("expected DeliverAck, got: {other:?}"),
                }
            });
    }

    #[test]
    fn geneva_exporter_emits_nack_for_decode_failure() {
        let test_runtime = TestRuntime::new();
        let exporter = create_exporter_from_factory(&GENEVA_EXPORTER, test_config()).unwrap();

        test_runtime
            .set_exporter(exporter)
            .run_test(|ctx| async move {
                // Non-empty but invalid protobuf bytes to trigger decode error (no network).
                let payload: OtapPayload =
                    OtlpProtoBytes::ExportLogsRequest(Bytes::from_static(b"\xff")).into();
                let pdata = OtapPdata::new_default(payload).test_subscribe_to(
                    Interests::NACKS,
                    TestCallData::default().into(),
                    777,
                );
                ctx.send_pdata(pdata).await.unwrap();
                ctx.send_shutdown(Instant::now() + Duration::from_secs(1), "test shutdown")
                    .await
                    .unwrap();
            })
            .run_validation(|mut ctx, result| async move {
                result.expect("success");

                let mut pipeline_rx = ctx.take_pipeline_ctrl_receiver().unwrap();
                match pipeline_rx.recv().await.unwrap() {
                    PipelineControlMsg::DeliverNack { nack, node_id } => {
                        assert_eq!(node_id, 777);
                        let got: TestCallData = nack.calldata.try_into().unwrap();
                        assert_eq!(TestCallData::default(), got);
                        assert!(
                            nack.reason.contains("Failed to decode logs request"),
                            "unexpected nack reason: {}",
                            nack.reason
                        );
                        assert_eq!(nack.refused.num_items(), 0);
                    }
                    other => panic!("expected DeliverNack, got: {other:?}"),
                }
            });
    }

    // TODO: Re-enable these tests when zero-copy view path is uncommented
    // #[test]
    // fn test_geneva_exporter_creates_view_from_otap_records() {
    //     // This test verifies that the Geneva exporter can successfully create
    //     // an OtapLogsView from OtapArrowRecords using the TryFrom implementation.
    //
    //     let logs_batch = create_test_logs_batch();
    //
    //     // Create OtapArrowRecords (simulating what batch processor would send)
    //     let mut otap_records = OtapArrowRecords::Logs(Default::default());
    //     otap_records.set(ArrowPayloadType::Logs, logs_batch.clone());
    //
    //     // This is what the Geneva exporter does internally
    //     let logs_view = OtapLogsView::try_from(&otap_records)
    //         .expect("Geneva exporter should create view from OTAP records");
    //
    //     // Verify the view can be used (basic sanity check)
    //     let mut log_count = 0;
    //     for resource_logs in logs_view.resources() {
    //         for scope_logs in resource_logs.scopes() {
    //             for _log_record in scope_logs.log_records() {
    //                 log_count += 1;
    //             }
    //         }
    //     }
    //
    //     assert_eq!(log_count, 3, "Expected 3 logs");
    // }
    //
    // #[test]
    // fn test_geneva_exporter_handles_missing_logs_batch() {
    //     // Verify that Geneva exporter properly handles the case where
    //     // OtapArrowRecords is missing the required logs batch
    //
    //     let otap_records = OtapArrowRecords::Logs(Default::default());
    //
    //     // This should fail because logs batch is missing
    //     let result = OtapLogsView::try_from(&otap_records);
    //
    //     assert!(result.is_err(), "Should fail when logs batch is missing");
    // }

    // Configuration tests
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
        assert_eq!(GENEVA_EXPORTER_URN, "urn:microsoft:geneva:exporter");
    }

    // TODO: Add integration tests when we can mock GenevaClient:
    // - test_geneva_exporter_encodes_and_uploads_logs_view()
    // - test_geneva_exporter_handles_upload_failure()
    // - test_geneva_exporter_fallback_to_otlp_bytes()
    // - test_geneva_exporter_metrics_tracking()
}
