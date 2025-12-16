// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! KQL processor that applies KQL queries to filter and transform telemetry data.
//!
//! This processor:
//! - Accepts OtapPdata (any payload format)
//! - Converts to OTLP bytes
//! - Processes through KQL recordset engine
//! - Returns transformed data
//!
//! The processor supports:
//! - Filtering logs/traces/metrics with KQL where clauses
//! - Transforming data with KQL extend/project operations
//! - Aggregation with KQL summarize operations

use crate::pdata::OtapPdata;

use async_trait::async_trait;
use data_engine_recordset::RecordSetEngineDiagnosticLevel;
use data_engine_recordset_otlp_bridge::{BridgeError, parse_kql_query_into_pipeline};
use linkme::distributed_slice;
use otap_df_config::SignalType;
use otap_df_config::{error::Error as ConfigError, node::NodeUserConfig};
use otap_df_engine::context::PipelineContext;
use otap_df_engine::{
    ProcessorFactory,
    config::ProcessorConfig,
    error::Error,
    local::processor::{EffectHandler, Processor},
    message::Message,
    node::NodeId,
    processor::ProcessorWrapper,
};
use otap_df_pdata::{OtapPayload, OtlpProtoBytes};
use otap_df_telemetry::instrument::Counter;
use otap_df_telemetry::metrics::MetricSet;
use otap_df_telemetry_macros::metric_set;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// URN for the KQL processor
pub const KQL_PROCESSOR_URN: &str = "urn:otel:kql:processor";

/// Configuration for the KQL processor
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct KqlConfig {
    /// KQL query to apply to the data
    pub query: String,

    /// Diagnostic level for the KQL engine
    #[serde(default = "default_diagnostic_level")]
    pub diagnostic_level: String,
}

fn default_diagnostic_level() -> String {
    "error".to_string()
}

impl KqlConfig {
    fn get_diagnostic_level(&self) -> RecordSetEngineDiagnosticLevel {
        match self.diagnostic_level.to_lowercase().as_str() {
            "verbose" => RecordSetEngineDiagnosticLevel::Verbose,
            "info" => RecordSetEngineDiagnosticLevel::Info,
            "warning" | "warn" => RecordSetEngineDiagnosticLevel::Warn,
            "error" => RecordSetEngineDiagnosticLevel::Error,
            _ => RecordSetEngineDiagnosticLevel::Error,
        }
    }
}

/// Telemetry metrics for the KQL Processor
#[metric_set(name = "kql.processor.metrics")]
#[derive(Debug, Default, Clone)]
pub struct KqlProcessorMetrics {
    // Consumed items by signal and outcome
    /// Number of items consumed (logs) with outcome=success
    #[metric(unit = "{item}")]
    pub consumed_items_logs_success: Counter<u64>,
    /// Number of items consumed (metrics) with outcome=success
    #[metric(unit = "{item}")]
    pub consumed_items_metrics_success: Counter<u64>,
    /// Number of items consumed (traces) with outcome=success
    #[metric(unit = "{item}")]
    pub consumed_items_traces_success: Counter<u64>,

    /// Number of items consumed (logs) with outcome=failure
    #[metric(unit = "{item}")]
    pub consumed_items_logs_failure: Counter<u64>,
    /// Number of items consumed (metrics) with outcome=failure
    #[metric(unit = "{item}")]
    pub consumed_items_metrics_failure: Counter<u64>,
    /// Number of items consumed (traces) with outcome=failure
    #[metric(unit = "{item}")]
    pub consumed_items_traces_failure: Counter<u64>,

    // Produced items by signal and outcome
    /// Number of items produced (logs) with outcome=success
    #[metric(unit = "{item}")]
    pub produced_items_logs_success: Counter<u64>,
    /// Number of items produced (metrics) with outcome=success
    #[metric(unit = "{item}")]
    pub produced_items_metrics_success: Counter<u64>,
    /// Number of items produced (traces) with outcome=success
    #[metric(unit = "{item}")]
    pub produced_items_traces_success: Counter<u64>,

    /// Number of items dropped (logs)
    #[metric(unit = "{item}")]
    pub dropped_items_logs: Counter<u64>,
    /// Number of items dropped (metrics)
    #[metric(unit = "{item}")]
    pub dropped_items_metrics: Counter<u64>,
    /// Number of items dropped (traces)
    #[metric(unit = "{item}")]
    pub dropped_items_traces: Counter<u64>,

    /// Number of KQL query errors
    #[metric(unit = "{error}")]
    pub query_errors: Counter<u64>,
}

impl KqlProcessorMetrics {
    fn add_consumed_success(&mut self, st: SignalType, n: u64) {
        match st {
            SignalType::Logs => self.consumed_items_logs_success.add(n),
            SignalType::Metrics => self.consumed_items_metrics_success.add(n),
            SignalType::Traces => self.consumed_items_traces_success.add(n),
        }
    }

    fn add_consumed_failure(&mut self, st: SignalType, n: u64) {
        match st {
            SignalType::Logs => self.consumed_items_logs_failure.add(n),
            SignalType::Metrics => self.consumed_items_metrics_failure.add(n),
            SignalType::Traces => self.consumed_items_traces_failure.add(n),
        }
    }

    fn add_produced_success(&mut self, st: SignalType, n: u64) {
        match st {
            SignalType::Logs => self.produced_items_logs_success.add(n),
            SignalType::Metrics => self.produced_items_metrics_success.add(n),
            SignalType::Traces => self.produced_items_traces_success.add(n),
        }
    }

    fn add_dropped(&mut self, st: SignalType, n: u64) {
        match st {
            SignalType::Logs => self.dropped_items_logs.add(n),
            SignalType::Metrics => self.dropped_items_metrics.add(n),
            SignalType::Traces => self.dropped_items_traces.add(n),
        }
    }
}

/// OTAP KQL Processor
#[allow(unsafe_code)]
#[distributed_slice(crate::OTAP_PROCESSOR_FACTORIES)]
pub static KQL_PROCESSOR_FACTORY: ProcessorFactory<OtapPdata> = ProcessorFactory {
    name: KQL_PROCESSOR_URN,
    create: create_kql_processor,
};

/// KQL processor that applies KQL queries to telemetry data
pub struct KqlProcessor {
    config: KqlConfig,
    pipeline: data_engine_expressions::PipelineExpression,
    metrics: MetricSet<KqlProcessorMetrics>,
}

/// Factory function to create a KQL processor
pub fn create_kql_processor(
    pipeline_ctx: PipelineContext,
    node: NodeId,
    node_config: Arc<NodeUserConfig>,
    processor_config: &ProcessorConfig,
) -> Result<ProcessorWrapper<OtapPdata>, ConfigError> {
    let config: KqlConfig = serde_json::from_value(node_config.config.clone()).map_err(|e| {
        ConfigError::InvalidUserConfig {
            error: format!("Failed to parse KQL configuration: {e}"),
        }
    })?;

    let processor = KqlProcessor::with_pipeline_ctx(pipeline_ctx, config)?;

    let user_config = Arc::new(NodeUserConfig::new_processor_config(KQL_PROCESSOR_URN));

    Ok(ProcessorWrapper::local(
        processor,
        node,
        user_config,
        processor_config,
    ))
}

impl KqlProcessor {
    /// Creates a new KQL processor with metrics registered via PipelineContext
    pub fn with_pipeline_ctx(
        pipeline_ctx: PipelineContext,
        config: KqlConfig,
    ) -> Result<Self, ConfigError> {
        let metrics = pipeline_ctx.register_metrics::<KqlProcessorMetrics>();

        otap_df_telemetry::otel_info!(
            "Processor.Create",
            processor = "kql",
            message = "Creating KQL processor",
            query_length = config.query.len(),
            diagnostic_level = config.diagnostic_level.as_str()
        );

        // Parse the KQL query into a pipeline
        let pipeline = parse_kql_query_into_pipeline(&config.query, None).map_err(|errors| {
            otap_df_telemetry::otel_error!(
                "Processor.ParseError",
                processor = "kql",
                message = "Failed to parse KQL query"
            );
            ConfigError::InvalidUserConfig {
                error: format!("Failed to parse KQL query: {:?}", errors),
            }
        })?;

        otap_df_telemetry::otel_info!(
            "Processor.Ready",
            processor = "kql",
            message = "KQL processor initialized successfully"
        );

        Ok(Self {
            config,
            pipeline,
            metrics,
        })
    }

    async fn process_data(
        &mut self,
        data: OtapPdata,
        effect_handler: &mut EffectHandler<OtapPdata>,
    ) -> Result<(), Error> {
        let signal = data.signal_type();
        let input_items = data.num_items() as u64;

        otap_df_telemetry::otel_debug!(
            "Processor.Process",
            processor = "kql",
            input_items = input_items,
            message = "Processing data through KQL pipeline"
        );

        if input_items == 0 {
            // Empty payload - just pass through
            otap_df_telemetry::otel_debug!(
                "Processor.EmptyPayload",
                processor = "kql",
                message = "Empty payload, passing through"
            );
            effect_handler.send_message(data).await?;
            return Ok(());
        }

        // Extract context and payload
        let (ctx, payload) = data.into_parts();

        // Convert to OTLP bytes (works for any internal representation)
        let otlp_bytes: OtlpProtoBytes = match payload.try_into() {
            Ok(bytes) => {
                otap_df_telemetry::otel_debug!(
                    "Processor.Converted",
                    processor = "kql",
                    message = "Converted to OTLP bytes"
                );
                bytes
            }
            Err(e) => {
                otap_df_telemetry::otel_error!(
                    "Processor.ConversionError",
                    processor = "kql",
                    message = "Failed to convert to OTLP bytes"
                );
                self.metrics.add_consumed_failure(signal, input_items);
                self.metrics.query_errors.add(1);
                return Err(Error::InternalError {
                    message: format!("Failed to convert to OTLP bytes: {}", e),
                });
            }
        };

        // Process based on signal type
        let result = match otlp_bytes {
            OtlpProtoBytes::ExportLogsRequest(bytes) => {
                otap_df_telemetry::otel_debug!(
                    "Processor.ProcessingLogs",
                    processor = "kql",
                    message = "Processing logs through KQL engine"
                );
                self.process_logs(bytes, signal, input_items).await
            }
            OtlpProtoBytes::ExportMetricsRequest(_bytes) => {
                // TODO: Implement metrics processing when KQL bridge supports it
                otap_df_telemetry::otel_warn!(
                    "Processor.UnsupportedSignal",
                    processor = "kql",
                    signal = "metrics",
                    message = "Metrics processing not yet implemented"
                );
                self.metrics.add_consumed_failure(signal, input_items);
                self.metrics.query_errors.add(1);
                Err(Error::InternalError {
                    message: "Metrics processing not yet implemented in KQL bridge".to_string(),
                })
            }
            OtlpProtoBytes::ExportTracesRequest(_bytes) => {
                // TODO: Implement traces processing when KQL bridge supports it
                otap_df_telemetry::otel_warn!(
                    "Processor.UnsupportedSignal",
                    processor = "kql",
                    signal = "traces",
                    message = "Traces processing not yet implemented"
                );
                self.metrics.add_consumed_failure(signal, input_items);
                self.metrics.query_errors.add(1);
                Err(Error::InternalError {
                    message: "Traces processing not yet implemented in KQL bridge".to_string(),
                })
            }
        };

        match result {
            Ok(processed_bytes) => {
                // Convert back to OtapPayload and reconstruct OtapPdata
                let payload: OtapPayload = processed_bytes.into();
                let output_items = payload.num_items() as u64;
                let dropped_items = input_items.saturating_sub(output_items);

                otap_df_telemetry::otel_info!(
                    "Processor.Success",
                    processor = "kql",
                    input_items = input_items,
                    output_items = output_items,
                    dropped_items = dropped_items,
                    message = "KQL processing completed successfully"
                );

                let processed_data = OtapPdata::new(ctx, payload);

                // Update metrics
                self.metrics.add_consumed_success(signal, input_items);
                self.metrics.add_produced_success(signal, output_items);
                self.metrics.add_dropped(signal, dropped_items);

                // Send processed data downstream
                effect_handler.send_message(processed_data).await?;
                Ok(())
            }
            Err(e) => {
                otap_df_telemetry::otel_error!(
                    "Processor.ProcessingError",
                    processor = "kql",
                    message = "KQL processing failed"
                );
                self.metrics.add_consumed_failure(signal, input_items);
                Err(e)
            }
        }
    }

    async fn process_logs(
        &mut self,
        bytes: bytes::Bytes,
        signal: SignalType,
        _input_items: u64,
    ) -> Result<OtlpProtoBytes, Error> {
        use data_engine_recordset_otlp_bridge::*;
        
        let diagnostic_level = self.config.get_diagnostic_level();

        // Parse the protobuf bytes into ExportLogsServiceRequest
        let request = ExportLogsServiceRequest::from_protobuf(&bytes)
            .map_err(|e| {
                self.metrics.query_errors.add(1);
                Error::InternalError {
                    message: format!("Failed to parse OTLP protobuf: {}", e),
                }
            })?;

        // Use the KQL bridge to process the logs
        let (included_records, _dropped_records) =
            process_export_logs_service_request_using_pipeline(
                None,
                &self.pipeline,
                diagnostic_level,
                request,
            )
            .map_err(|e| self.map_bridge_error(e, signal))?;

        // Convert back to protobuf bytes
        if let Some(included) = included_records {
            let protobuf_bytes = ExportLogsServiceRequest::to_protobuf(&included, 1024 * 64)
                .map_err(|e| {
                    self.metrics.query_errors.add(1);
                    Error::InternalError {
                        message: format!("Failed to encode OTLP protobuf: {}", e),
                    }
                })?;
            Ok(OtlpProtoBytes::ExportLogsRequest(bytes::Bytes::copy_from_slice(&protobuf_bytes)))
        } else {
            // No records included, return empty
            Ok(OtlpProtoBytes::ExportLogsRequest(bytes::Bytes::new()))
        }
    }

    fn map_bridge_error(&mut self, error: BridgeError, signal: SignalType) -> Error {
        self.metrics.query_errors.add(1);
        Error::InternalError {
            message: format!("KQL bridge error for {:?}: {}", signal, error),
        }
    }
}

#[async_trait(?Send)]
impl Processor<OtapPdata> for KqlProcessor {
    async fn process(
        &mut self,
        msg: Message<OtapPdata>,
        effect_handler: &mut EffectHandler<OtapPdata>,
    ) -> Result<(), Error> {
        match msg {
            Message::PData(data) => self.process_data(data, effect_handler).await,
            Message::Control(control_msg) => {
                use otap_df_engine::control::NodeControlMsg;
                match control_msg {
                    NodeControlMsg::CollectTelemetry {
                        mut metrics_reporter,
                    } => metrics_reporter
                        .report(&mut self.metrics)
                        .map_err(|e| Error::InternalError {
                            message: e.to_string(),
                        }),
                    NodeControlMsg::Config { config } => {
                        if let Ok(new_config) = serde_json::from_value::<KqlConfig>(config) {
                            // Re-parse the pipeline if query changed
                            if new_config.query != self.config.query {
                                match parse_kql_query_into_pipeline(&new_config.query, None) {
                                    Ok(pipeline) => {
                                        self.pipeline = pipeline;
                                        self.config = new_config;
                                    }
                                    Err(errors) => {
                                        log::error!("Failed to parse new KQL query: {:?}", errors);
                                    }
                                }
                            } else {
                                self.config = new_config;
                            }
                        }
                        Ok(())
                    }
                    NodeControlMsg::Shutdown { .. } => Ok(()),
                    _ => Ok(()),
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use otap_df_engine::context::ControllerContext;
    use otap_df_telemetry::registry::MetricsRegistryHandle;
    use pretty_assertions::assert_eq;
    use serde_json::json;

    fn create_test_pipeline_context() -> PipelineContext {
        let metrics_registry = MetricsRegistryHandle::new();
        let controller_ctx = ControllerContext::new(metrics_registry);
        controller_ctx.pipeline_context_with("test_grp".into(), "test_pipeline".into(), 0, 0)
    }

    #[test]
    fn test_config_parsing() {
        let config_json = json!({
            "query": "source | where true"
        });

        let config: KqlConfig = serde_json::from_value(config_json).unwrap();
        assert_eq!(config.query, "source | where true");
        assert_eq!(config.diagnostic_level, "error");
    }

    #[test]
    fn test_config_with_diagnostic_level() {
        let config_json = json!({
            "query": "source | where SeverityText == 'INFO'",
            "diagnostic_level": "verbose"
        });

        let config: KqlConfig = serde_json::from_value(config_json).unwrap();
        assert_eq!(config.query, "source | where SeverityText == 'INFO'");
        assert_eq!(config.diagnostic_level, "verbose");
        assert_eq!(
            config.get_diagnostic_level(),
            RecordSetEngineDiagnosticLevel::Verbose
        );
    }

    #[test]
    fn test_processor_creation() {
        let pipeline_ctx = create_test_pipeline_context();
        let config = KqlConfig {
            query: "source | where true".to_string(),
            diagnostic_level: "error".to_string(),
        };

        let processor = KqlProcessor::with_pipeline_ctx(pipeline_ctx, config);
        assert!(processor.is_ok());
    }

    #[test]
    fn test_invalid_query() {
        let pipeline_ctx = create_test_pipeline_context();
        let config = KqlConfig {
            query: "invalid kql syntax <<<".to_string(),
            diagnostic_level: "error".to_string(),
        };

        let processor = KqlProcessor::with_pipeline_ctx(pipeline_ctx, config);
        assert!(processor.is_err());
    }

    #[test]
    fn test_conditional_extend_vendor_info() {
        // Test demonstrating vendor-specific enrichment based on EventName or Body matching
        let pipeline_ctx = create_test_pipeline_context();
        
        // Query that matches on EventName or Body and extends with vendor info
        let query = r#"
            source
            | extend vendor = case(
                EventName == "payment.completed" or Body has "payment", "payment-processor",
                EventName == "user.login" or Body has "login", "auth-service",
                EventName == "metric.emitted" or Body has "metric", "observability",
                "unknown"
            )
            | extend priority = case(
                EventName has "error" or SeverityNumber >= 17, "high",
                EventName has "warning" or SeverityNumber >= 13, "medium",
                "low"
            )
            | extend processed_at = now()
        "#;

        let config = KqlConfig {
            query: query.to_string(),
            diagnostic_level: "info".to_string(),
        };

        let processor = KqlProcessor::with_pipeline_ctx(pipeline_ctx, config);
        assert!(processor.is_ok(), "Query should parse successfully");
    }

    #[test]
    fn test_multiple_conditions_with_or() {
        let pipeline_ctx = create_test_pipeline_context();
        
        // Query demonstrating OR conditions and statement-by-statement enrichment
        let query = r#"
            source
            | where EventName == "critical.alert" or Body has "CRITICAL" or SeverityNumber >= 21
            | extend alert_type = case(
                Body has "OutOfMemory" or EventName has "oom", "memory",
                Body has "DiskFull" or EventName has "disk", "storage",
                Body has "NetworkTimeout" or EventName has "timeout", "network",
                "generic"
            )
            | extend team = case(
                alert_type == "memory", "infrastructure",
                alert_type == "storage", "storage-ops",
                alert_type == "network", "network-ops",
                "on-call"
            )
            | extend escalation_minutes = case(
                alert_type == "memory", 5,
                alert_type == "storage", 15,
                30
            )
        "#;

        let config = KqlConfig {
            query: query.to_string(),
            diagnostic_level: "verbose".to_string(),
        };

        let processor = KqlProcessor::with_pipeline_ctx(pipeline_ctx, config);
        assert!(processor.is_ok(), "Complex OR and case query should parse");
    }

    #[test]
    fn test_pattern_matching_extend() {
        let pipeline_ctx = create_test_pipeline_context();
        
        // Query using pattern matching with 'has' operator
        let query = r#"
            source
            | extend service_type = case(
                EventName has "http" or Body has "HTTP", "web-service",
                EventName has "grpc" or Body has "gRPC", "rpc-service",
                EventName has "kafka" or Body has "Kafka", "messaging",
                EventName has "postgres" or Body has "PostgreSQL", "database",
                "other"
            )
            | extend vendor_category = case(
                service_type == "web-service", "frontend",
                service_type == "rpc-service", "backend",
                service_type == "messaging", "integration",
                service_type == "database", "data-layer",
                "uncategorized"
            )
            | project-keep EventName, Body, service_type, vendor_category, SeverityText
        "#;

        let config = KqlConfig {
            query: query.to_string(),
            diagnostic_level: "error".to_string(),
        };

        let processor = KqlProcessor::with_pipeline_ctx(pipeline_ctx, config);
        assert!(processor.is_ok(), "Pattern matching query should parse successfully");
    }

    #[test]
    fn test_body_modification_with_strcat() {
        let pipeline_ctx = create_test_pipeline_context();
        
        // Query that appends supplemental information to the Body
        let query = r#"
            source
            | extend Body = case(
                EventName == "payment.failed" or Body has "payment failed",
                strcat(Body, "\n\nTroubleshooting: https://docs.example.com/payment-errors\nSupport: https://support.example.com/ticket"),
                EventName == "auth.error" or Body has "authentication",
                strcat(Body, "\n\nAuth Guide: https://docs.example.com/auth\nReset Password: https://portal.example.com/reset"),
                EventName has "error" or SeverityNumber >= 17,
                strcat(Body, "\n\nError Guide: https://docs.example.com/errors"),
                Body
            )
        "#;

        let config = KqlConfig {
            query: query.to_string(),
            diagnostic_level: "info".to_string(),
        };

        let processor = KqlProcessor::with_pipeline_ctx(pipeline_ctx, config);
        assert!(processor.is_ok(), "Body modification with strcat should parse successfully");
    }

    #[test]
    fn test_body_enrichment_with_vendor_links() {
        let pipeline_ctx = create_test_pipeline_context();
        
        // Query that adds vendor-specific links and guidelines based on content
        let query = r#"
            source
            | extend vendor_type = case(
                Body has "AWS" or Body has "Amazon", "aws",
                Body has "Azure" or Body has "Microsoft", "azure",
                Body has "GCP" or Body has "Google", "gcp",
                "other"
            )
            | extend Body = case(
                vendor_type == "aws",
                strcat(Body, "\n\n[AWS Guidelines]\nDocs: https://docs.aws.amazon.com\nSupport: https://console.aws.amazon.com/support\nStatus: https://health.aws.amazon.com"),
                vendor_type == "azure",
                strcat(Body, "\n\n[Azure Guidelines]\nDocs: https://learn.microsoft.com/azure\nSupport: https://portal.azure.com/#blade/Microsoft_Azure_Support\nStatus: https://status.azure.com"),
                vendor_type == "gcp",
                strcat(Body, "\n\n[GCP Guidelines]\nDocs: https://cloud.google.com/docs\nSupport: https://console.cloud.google.com/support\nStatus: https://status.cloud.google.com"),
                Body
            )
        "#;

        let config = KqlConfig {
            query: query.to_string(),
            diagnostic_level: "error".to_string(),
        };

        let processor = KqlProcessor::with_pipeline_ctx(pipeline_ctx, config);
        assert!(processor.is_ok(), "Body enrichment with vendor links should parse successfully");
    }
}
