// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use crate::pdata::OtapPdata;
use async_trait::async_trait;
use data_engine_recordset::RecordSetEngineDiagnosticLevel;
use data_engine_recordset_otlp_bridge::{
    BridgeError, process_protobuf_otlp_export_logs_service_request_using_registered_pipeline,
    register_pipeline_for_kql_query,
};
use linkme::distributed_slice;
use otap_df_config::SignalType;
use otap_df_config::{error::Error as ConfigError, node::NodeUserConfig};
use otap_df_engine::{
    ConsumerEffectHandlerExtension, ProcessorFactory,
    config::ProcessorConfig,
    context::PipelineContext,
    control::NackMsg,
    error::Error,
    local::processor::{EffectHandler, Processor},
    message::Message,
    node::NodeId,
    processor::ProcessorWrapper,
};
use otap_df_pdata::{OtapPayload, OtlpProtoBytes};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

pub const KQL_PROCESSOR_URN: &str = "urn:otel:kql:processor";

/// Configuration for the KQL processor
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct KqlConfig {
    /// KQL query to apply to the data
    pub query: String,
}

// TODO metrics

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

    // Note! library uses a static for registration so all we store is a usize,
    // TODO reconcile, otherwise memory management is unclear.
    kql_query_id: usize,
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

    Ok(ProcessorWrapper::local(
        processor,
        node,
        node_config,
        processor_config,
    ))
}

impl KqlProcessor {
    /// Creates a new KQL processor
    pub fn with_pipeline_ctx(
        _pipeline_ctx: PipelineContext,
        config: KqlConfig,
    ) -> Result<Self, ConfigError> {
        let kql_query_id =
            register_pipeline_for_kql_query(&config.query, None).map_err(|errors| {
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
            kql_query_id,
        })
    }

    async fn process_data(
        &mut self,
        data: OtapPdata,
        effect_handler: &mut EffectHandler<OtapPdata>,
    ) -> Result<(), Error> {
        let signal = data.signal_type();
        let input_items = data.num_items() as u64;

        // Extract context and payload, convert to OTLP bytes
        let (ctx, payload) = data.into_parts();
        let otlp_bytes: OtlpProtoBytes = payload.try_into()?;

        // Process based on signal type
        let result = match otlp_bytes {
            OtlpProtoBytes::ExportLogsRequest(bytes) => {
                otap_df_telemetry::otel_debug!(
                    "Processor.ProcessingLogs",
                    processor = "kql",
                    message = "Processing KQL query",
                    input_items = input_items,
                );
                self.process_logs(bytes, signal).await
            }
            OtlpProtoBytes::ExportMetricsRequest(_bytes) => Err(Error::InternalError {
                message: "Metrics processing not yet implemented in KQL bridge".to_string(),
            }),
            OtlpProtoBytes::ExportTracesRequest(_bytes) => Err(Error::InternalError {
                message: "Traces processing not yet implemented in KQL bridge".to_string(),
            }),
        };

        match result {
            Ok(processed_bytes) => {
                // Convert back to OtapPayload and reconstruct OtapPdata
                let payload: OtapPayload = processed_bytes.into();
                // Note! we are recomputing the number of matched items, which
                // the engine could tell us.
                let output_items = payload.num_items() as u64;

                otap_df_telemetry::otel_debug!(
                    "Processor.Success",
                    processor = "kql",
                    input_items = input_items,
                    output_items = output_items,
                );

                let processed_data = OtapPdata::new(ctx, payload);

                effect_handler.send_message(processed_data).await?;
                Ok(())
            }
            Err(e) => {
                let message = e.to_string();
                otap_df_telemetry::otel_debug!(
                    "Processor.Failure",
                    processor = "kql",
                    input_items = input_items,
                    message = message,
                );

                effect_handler
                    .notify_nack(NackMsg::new(
                        message,
                        OtapPdata::new(ctx, OtapPayload::empty(SignalType::Logs)),
                    ))
                    .await?;
                Err(e)
            }
        }
    }

    async fn process_logs(
        &mut self,
        bytes: bytes::Bytes,
        signal: SignalType,
    ) -> Result<OtlpProtoBytes, Error> {
        let (included_records, _) =
            process_protobuf_otlp_export_logs_service_request_using_registered_pipeline(
                self.kql_query_id,
                RecordSetEngineDiagnosticLevel::Warn,
                &bytes,
            )
            .map_err(|e| self.map_bridge_error(e, signal))?;

        Ok(OtlpProtoBytes::ExportLogsRequest(included_records.into()))
    }

    fn map_bridge_error(&mut self, error: BridgeError, signal: SignalType) -> Error {
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
                    NodeControlMsg::Config { config } => {
                        if let Ok(new_config) = serde_json::from_value::<KqlConfig>(config) {
                            // Re-parse the pipeline (if changed)
                            if new_config.query != self.config.query {
                                match register_pipeline_for_kql_query(&new_config.query, None) {
                                    Ok(kql_query_id) => {
                                        otap_df_telemetry::otel_error!(
                                            "Processor.Reconfigured",
                                            processor = "kql",
                                        );

                                        self.kql_query_id = kql_query_id;
                                        self.config = new_config;
                                    }
                                    Err(errors) => {
                                        let message =
                                            format!("Failed to parse updated query: {:?}", errors);
                                        otap_df_telemetry::otel_error!(
                                            "Processor.ReconfigureError",
                                            processor = "kql",
                                            message = message,
                                        );
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
    }

    #[test]
    fn test_processor_creation() {
        let pipeline_ctx = create_test_pipeline_context();
        let config = KqlConfig {
            query: "source | where true".to_string(),
        };

        let processor = KqlProcessor::with_pipeline_ctx(pipeline_ctx, config);
        assert!(processor.is_ok());
    }

    #[test]
    fn test_invalid_query() {
        let pipeline_ctx = create_test_pipeline_context();
        let config = KqlConfig {
            query: "invalid kql syntax <<<".to_string(),
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
        };

        let processor = KqlProcessor::with_pipeline_ctx(pipeline_ctx, config);
        assert!(
            processor.is_ok(),
            "Pattern matching query should parse successfully"
        );
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
        };

        let processor = KqlProcessor::with_pipeline_ctx(pipeline_ctx, config);
        assert!(
            processor.is_ok(),
            "Body modification with strcat should parse successfully"
        );
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
        };

        let processor = KqlProcessor::with_pipeline_ctx(pipeline_ctx, config);
        assert!(
            processor.is_ok(),
            "Body enrichment with vendor links should parse successfully"
        );
    }
}
