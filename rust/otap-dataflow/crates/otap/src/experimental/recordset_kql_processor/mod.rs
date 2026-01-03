// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use crate::pdata::OtapPdata;
use async_trait::async_trait;
use data_engine_recordset::RecordSetEngineDiagnosticLevel;
use data_engine_recordset_otlp_bridge::{
    BridgeError, BridgeOptions,
    process_protobuf_otlp_export_logs_service_request_using_registered_pipeline,
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

/// URN identifier for the processor
pub const RECORDSET_KQL_PROCESSOR_URN: &str = "urn:otel:recordset_kql:processor";

/// Configuration for the KQL processor
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Config {
    /// KQL query to apply to the data
    pub query: String,
    /// Optional bridge options for KQL processing
    #[serde(default)]
    pub bridge_options: Option<serde_json::Value>,
}

// TODO metrics

/// OTAP KQL Processor
#[allow(unsafe_code)]
#[distributed_slice(crate::OTAP_PROCESSOR_FACTORIES)]
pub static RECORDSET_KQL_PROCESSOR_FACTORY: ProcessorFactory<OtapPdata> = ProcessorFactory {
    name: RECORDSET_KQL_PROCESSOR_URN,
    create: create_recordset_kql_processor,
};

/// KQL processor that applies KQL queries to telemetry data
pub struct RecordsetKqlProcessor {
    config: Config,

    // Note! library uses a static for registration so all we store is a usize,
    // TODO reconcile, otherwise memory management is unclear.
    kql_query_id: usize,
}

/// Factory function to create a KQL processor
pub fn create_recordset_kql_processor(
    pipeline_ctx: PipelineContext,
    node: NodeId,
    node_config: Arc<NodeUserConfig>,
    processor_config: &ProcessorConfig,
) -> Result<ProcessorWrapper<OtapPdata>, ConfigError> {
    let config: Config = serde_json::from_value(node_config.config.clone()).map_err(|e| {
        ConfigError::InvalidUserConfig {
            error: format!("Failed to parse KQL configuration: {e}"),
        }
    })?;

    let processor = RecordsetKqlProcessor::with_pipeline_ctx(pipeline_ctx, config)?;

    Ok(ProcessorWrapper::local(
        processor,
        node,
        node_config,
        processor_config,
    ))
}

impl RecordsetKqlProcessor {
    /// Creates a new KQL processor
    pub fn with_pipeline_ctx(
        _pipeline_ctx: PipelineContext,
        config: Config,
    ) -> Result<Self, ConfigError> {
        let bridge_options = Self::parse_bridge_options(&config.bridge_options)?;
        let kql_query_id =
            register_pipeline_for_kql_query(&config.query, bridge_options).map_err(|errors| {
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

    /// Parse bridge options from JSON value
    fn parse_bridge_options(
        bridge_options_json: &Option<serde_json::Value>,
    ) -> Result<Option<BridgeOptions>, ConfigError> {
        if let Some(json_value) = bridge_options_json {
            let json_str =
                serde_json::to_string(json_value).map_err(|e| ConfigError::InvalidUserConfig {
                    error: format!("Failed to serialize bridge options: {}", e),
                })?;
            let bridge_options = BridgeOptions::from_json(&json_str).map_err(|e| {
                ConfigError::InvalidUserConfig {
                    error: format!("Failed to parse bridge options: {}", e),
                }
            })?;
            Ok(Some(bridge_options))
        } else {
            Ok(None)
        }
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
                    processor = "recordset_kql",
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
                    processor = "recordset_kql",
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
                    processor = "recordset_kql",
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
impl Processor<OtapPdata> for RecordsetKqlProcessor {
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
                        if let Ok(new_config) = serde_json::from_value::<Config>(config) {
                            // Re-parse the pipeline (if changed)
                            if new_config.query != self.config.query
                                || new_config.bridge_options != self.config.bridge_options
                            {
                                let bridge_options =
                                    Self::parse_bridge_options(&new_config.bridge_options)
                                        .ok()
                                        .flatten();
                                match register_pipeline_for_kql_query(
                                    &new_config.query,
                                    bridge_options,
                                ) {
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
mod config_tests {
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

        let config: Config = serde_json::from_value(config_json).unwrap();
        assert_eq!(config.query, "source | where true");
        assert_eq!(config.bridge_options, None);
    }

    #[test]
    fn test_config_parsing_with_bridge_options() {
        let config_json = json!({
            "query": "source | where true",
            "bridge_options": {
                "attributes_schema": {
                    "double_key": "Double",
                    "string_key": "String"
                }
            }
        });

        let config: Config = serde_json::from_value(config_json).unwrap();
        assert_eq!(config.query, "source | where true");
        assert!(config.bridge_options.is_some());

        // Verify bridge options can be parsed
        let bridge_options = RecordsetKqlProcessor::parse_bridge_options(&config.bridge_options);
        assert!(bridge_options.is_ok());
        assert!(bridge_options.unwrap().is_some());
    }

    #[test]
    fn test_processor_creation() {
        let pipeline_ctx = create_test_pipeline_context();
        let config = Config {
            query: "source | where true".to_string(),
            bridge_options: None,
        };

        let processor = RecordsetKqlProcessor::with_pipeline_ctx(pipeline_ctx, config);
        assert!(processor.is_ok());
    }

    #[test]
    fn test_processor_creation_with_bridge_options() {
        let pipeline_ctx = create_test_pipeline_context();
        let bridge_options_json = json!({
            "attributes_schema": {
                "custom_field": "String",
                "metric_value": "Double"
            }
        });

        let config = Config {
            query: "source | where true".to_string(),
            bridge_options: Some(bridge_options_json),
        };

        let processor = RecordsetKqlProcessor::with_pipeline_ctx(pipeline_ctx, config);
        assert!(processor.is_ok());
    }

    #[test]
    fn test_invalid_query() {
        let pipeline_ctx = create_test_pipeline_context();
        let config = Config {
            query: "invalid kql syntax <<<".to_string(),
            bridge_options: None,
        };

        let processor = RecordsetKqlProcessor::with_pipeline_ctx(pipeline_ctx, config);
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

        let config = Config {
            query: query.to_string(),
            bridge_options: None,
        };

        let processor = RecordsetKqlProcessor::with_pipeline_ctx(pipeline_ctx, config);
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

        let config = Config {
            query: query.to_string(),
            bridge_options: None,
        };

        let processor = RecordsetKqlProcessor::with_pipeline_ctx(pipeline_ctx, config);
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

        let config = Config {
            query: query.to_string(),
            bridge_options: None,
        };

        let processor = RecordsetKqlProcessor::with_pipeline_ctx(pipeline_ctx, config);
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

        let config = Config {
            query: query.to_string(),
            bridge_options: None,
        };

        let processor = RecordsetKqlProcessor::with_pipeline_ctx(pipeline_ctx, config);
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

        let config = Config {
            query: query.to_string(),
            bridge_options: None,
        };

        let processor = RecordsetKqlProcessor::with_pipeline_ctx(pipeline_ctx, config);
        assert!(
            processor.is_ok(),
            "Body enrichment with vendor links should parse successfully"
        );
    }
}

#[cfg(test)]
mod runtime_tests {
    use super::*;
    use crate::pdata::OtapPdata;
    use bytes::BytesMut;
    use otap_df_engine::context::ControllerContext;
    use otap_df_engine::message::Message;
    use otap_df_engine::testing::{node::test_node, processor::TestRuntime};
    use otap_df_pdata::OtlpProtoBytes;
    use otap_df_pdata::proto::opentelemetry::{
        collector::logs::v1::ExportLogsServiceRequest,
        common::v1::{AnyValue, InstrumentationScope, KeyValue, any_value::Value::*},
        logs::v1::{LogRecord, ResourceLogs, ScopeLogs, SeverityNumber},
        resource::v1::Resource,
    };
    use otap_df_telemetry::registry::MetricsRegistryHandle;
    use prost::Message as _;
    use serde_json::json;

    // Helper functions for test setup and common operations

    /// Builds a log request with the given log records
    fn build_log_request(log_records: Vec<LogRecord>) -> ExportLogsServiceRequest {
        ExportLogsServiceRequest::new(vec![ResourceLogs::new(
            Resource {
                ..Default::default()
            },
            vec![ScopeLogs::new(
                InstrumentationScope {
                    ..Default::default()
                },
                log_records,
            )],
        )])
    }

    /// Builds a single log record with the given attributes
    fn build_log_with_attrs(log_attrs: Vec<KeyValue>) -> ExportLogsServiceRequest {
        build_log_request(vec![
            LogRecord::build()
                .time_unix_nano(1u64)
                .severity_number(SeverityNumber::Info)
                .event_name("")
                .attributes(log_attrs)
                .finish(),
        ])
    }

    /// Runs a KQL processor test with custom validation
    fn run_kql_test<F>(input: ExportLogsServiceRequest, query: &str, validate: F)
    where
        F: FnOnce(ExportLogsServiceRequest) + 'static,
    {
        run_kql_test_with_bridge_options(input, query, None, validate);
    }

    /// Runs a KQL processor test with bridge options and custom validation
    fn run_kql_test_with_bridge_options<F>(
        input: ExportLogsServiceRequest,
        query: &str,
        bridge_options: Option<serde_json::Value>,
        validate: F,
    ) where
        F: FnOnce(ExportLogsServiceRequest) + 'static,
    {
        let metrics_registry_handle = MetricsRegistryHandle::new();
        let controller_ctx = ControllerContext::new(metrics_registry_handle);
        let pipeline_ctx =
            controller_ctx.pipeline_context_with("grp".into(), "pipeline".into(), 0, 0);

        let node = test_node("recordset-kql-processor-test");
        let rt: TestRuntime<OtapPdata> = TestRuntime::new();
        let mut node_config = NodeUserConfig::new_processor_config(RECORDSET_KQL_PROCESSOR_URN);

        node_config.config = if let Some(opts) = bridge_options {
            json!({ "query": query, "bridge_options": opts })
        } else {
            json!({ "query": query })
        };

        let proc =
            create_recordset_kql_processor(pipeline_ctx, node, Arc::new(node_config), rt.config())
                .expect("create processor");
        let phase = rt.set_processor(proc);

        phase
            .run_test(|mut ctx| async move {
                let mut bytes = BytesMut::new();
                input.encode(&mut bytes).expect("encode");
                let bytes = bytes.freeze();
                let pdata_in =
                    OtapPdata::new_default(OtlpProtoBytes::ExportLogsRequest(bytes).into());
                ctx.process(Message::PData(pdata_in))
                    .await
                    .expect("process");

                let out = ctx.drain_pdata().await;
                let first = out.into_iter().next().expect("one output").payload();

                let otlp_bytes: OtlpProtoBytes = first.try_into().expect("convert to otlp");
                let bytes = match otlp_bytes {
                    OtlpProtoBytes::ExportLogsRequest(b) => b,
                    _ => panic!("unexpected otlp variant"),
                };
                let decoded = ExportLogsServiceRequest::decode(bytes.as_ref()).expect("decode");

                validate(decoded);
            })
            .validate(|_| async move {});
    }

    /// Tests a KQL processor with expected log record count
    fn test_kql_with_count(input: ExportLogsServiceRequest, query: &str, expected_count: usize) {
        run_kql_test(input, query, move |decoded| {
            let total_count: usize = decoded
                .resource_logs
                .iter()
                .flat_map(|rl| &rl.scope_logs)
                .map(|sl| sl.log_records.len())
                .sum();
            assert_eq!(
                total_count, expected_count,
                "Expected {} log records, got {}",
                expected_count, total_count
            );
        });
    }

    #[test]
    fn test_extend_operators() {
        // Test basic extend with literals, conditionals (iif, case), and expressions
        let input = build_log_request(vec![
            LogRecord::build()
                .time_unix_nano(1u64)
                .attributes(vec![
                    KeyValue::new("Level", AnyValue::new_string("error")),
                    KeyValue::new("Priority", AnyValue::new_int(9)),
                ])
                .finish(),
            LogRecord::build()
                .time_unix_nano(2u64)
                .attributes(vec![
                    KeyValue::new("Level", AnyValue::new_string("warning")),
                    KeyValue::new("Priority", AnyValue::new_int(5)),
                ])
                .finish(),
            LogRecord::build()
                .time_unix_nano(3u64)
                .attributes(vec![
                    KeyValue::new("Level", AnyValue::new_string("info")),
                    KeyValue::new("Priority", AnyValue::new_int(2)),
                ])
                .finish(),
        ]);

        run_kql_test(
            input,
            r#"source 
            | extend Literal = 'constant'
            | extend Body = 'transformed'
            | extend IsHighPriority = (Priority > 7)
            | extend AlertLevel = iif(Level == 'error', 'HIGH', 'NORMAL')
            | extend Severity = case(Level == 'error', 'critical', Level == 'warning', 'medium', 'low')
            "#,
            |decoded| {
                let log_records = &decoded.resource_logs[0].scope_logs[0].log_records;
                assert_eq!(log_records.len(), 3);

                // Verify first record (error, priority 9)
                let rec1 = &log_records[0];
                assert_eq!(
                    rec1.attributes
                        .iter()
                        .find(|kv| kv.key == "Literal")
                        .unwrap()
                        .value
                        .as_ref()
                        .unwrap()
                        .value
                        .as_ref()
                        .unwrap(),
                    &StringValue("constant".to_string())
                );
                assert_eq!(
                    rec1.body.as_ref().unwrap().value.as_ref().unwrap(),
                    &StringValue("transformed".to_string())
                );
                assert_eq!(
                    rec1.attributes
                        .iter()
                        .find(|kv| kv.key == "IsHighPriority")
                        .unwrap()
                        .value
                        .as_ref()
                        .unwrap()
                        .value
                        .as_ref()
                        .unwrap(),
                    &BoolValue(true)
                );
                assert_eq!(
                    rec1.attributes
                        .iter()
                        .find(|kv| kv.key == "AlertLevel")
                        .unwrap()
                        .value
                        .as_ref()
                        .unwrap()
                        .value
                        .as_ref()
                        .unwrap(),
                    &StringValue("HIGH".to_string())
                );
                assert_eq!(
                    rec1.attributes
                        .iter()
                        .find(|kv| kv.key == "Severity")
                        .unwrap()
                        .value
                        .as_ref()
                        .unwrap()
                        .value
                        .as_ref()
                        .unwrap(),
                    &StringValue("critical".to_string())
                );

                // Verify second record (warning, priority 5)
                let rec2 = &log_records[1];
                assert_eq!(
                    rec2.attributes
                        .iter()
                        .find(|kv| kv.key == "IsHighPriority")
                        .unwrap()
                        .value
                        .as_ref()
                        .unwrap()
                        .value
                        .as_ref()
                        .unwrap(),
                    &BoolValue(false)
                );
                assert_eq!(
                    rec2.attributes
                        .iter()
                        .find(|kv| kv.key == "AlertLevel")
                        .unwrap()
                        .value
                        .as_ref()
                        .unwrap()
                        .value
                        .as_ref()
                        .unwrap(),
                    &StringValue("NORMAL".to_string())
                );
                assert_eq!(
                    rec2.attributes
                        .iter()
                        .find(|kv| kv.key == "Severity")
                        .unwrap()
                        .value
                        .as_ref()
                        .unwrap()
                        .value
                        .as_ref()
                        .unwrap(),
                    &StringValue("medium".to_string())
                );

                // Verify third record (info, priority 2)
                let rec3 = &log_records[2];
                assert_eq!(
                    rec3.attributes
                        .iter()
                        .find(|kv| kv.key == "Severity")
                        .unwrap()
                        .value
                        .as_ref()
                        .unwrap()
                        .value
                        .as_ref()
                        .unwrap(),
                    &StringValue("low".to_string())
                );
            },
        );
    }

    #[test]
    fn test_where_operators() {
        // Test string operators: has, contains, in, !=, =~
        let string_input = build_log_request(vec![
            LogRecord::build()
                .time_unix_nano(1u64)
                .body(AnyValue::new_string("ERROR in payment system"))
                .attributes(vec![KeyValue::new("Status", AnyValue::new_string("error"))])
                .finish(),
            LogRecord::build()
                .time_unix_nano(2u64)
                .body(AnyValue::new_string("warning: high load"))
                .attributes(vec![KeyValue::new(
                    "Status",
                    AnyValue::new_string("warning"),
                )])
                .finish(),
            LogRecord::build()
                .time_unix_nano(3u64)
                .body(AnyValue::new_string("info: normal operation"))
                .attributes(vec![KeyValue::new("Status", AnyValue::new_string("info"))])
                .finish(),
        ]);
        test_kql_with_count(string_input.clone(), "source | where Body has 'payment'", 1);
        test_kql_with_count(
            string_input.clone(),
            "source | where Body contains 'ERROR'",
            1,
        );
        test_kql_with_count(
            string_input.clone(),
            "source | where Status in ('error', 'warning')",
            2,
        );
        test_kql_with_count(string_input.clone(), "source | where Status != 'info'", 2);
        test_kql_with_count(string_input, "source | where Status =~ 'ERROR'", 1);

        // Test comparison operators: >, >=, <, <=, ==
        let numeric_input = build_log_request(vec![
            LogRecord::build()
                .time_unix_nano(1u64)
                .attributes(vec![KeyValue::new("Priority", AnyValue::new_int(8))])
                .finish(),
            LogRecord::build()
                .time_unix_nano(2u64)
                .attributes(vec![KeyValue::new("Priority", AnyValue::new_int(5))])
                .finish(),
            LogRecord::build()
                .time_unix_nano(3u64)
                .attributes(vec![KeyValue::new("Priority", AnyValue::new_int(3))])
                .finish(),
        ]);
        test_kql_with_count(numeric_input.clone(), "source | where Priority > 5", 1);
        test_kql_with_count(numeric_input.clone(), "source | where Priority >= 5", 2);
        test_kql_with_count(numeric_input.clone(), "source | where Priority < 5", 1);
        test_kql_with_count(numeric_input.clone(), "source | where Priority <= 5", 2);
        test_kql_with_count(numeric_input, "source | where Priority == 5", 1);

        // Test logical operators: and, or
        let logical_input = build_log_request(vec![
            LogRecord::build()
                .time_unix_nano(1u64)
                .attributes(vec![
                    KeyValue::new("Level", AnyValue::new_string("error")),
                    KeyValue::new("Priority", AnyValue::new_int(10)),
                ])
                .finish(),
            LogRecord::build()
                .time_unix_nano(2u64)
                .attributes(vec![
                    KeyValue::new("Level", AnyValue::new_string("error")),
                    KeyValue::new("Priority", AnyValue::new_int(3)),
                ])
                .finish(),
            LogRecord::build()
                .time_unix_nano(3u64)
                .attributes(vec![
                    KeyValue::new("Level", AnyValue::new_string("info")),
                    KeyValue::new("Priority", AnyValue::new_int(9)),
                ])
                .finish(),
        ]);
        test_kql_with_count(
            logical_input.clone(),
            "source | where Level == 'error' and Priority > 5",
            1,
        );
        test_kql_with_count(
            logical_input.clone(),
            "source | where Level == 'error' or Priority >= 9",
            3,
        );
        test_kql_with_count(logical_input, "source | where false", 0);
    }

    #[test]
    fn test_project_operators() {
        // Test project (select and transform columns)
        let project_input = build_log_with_attrs(vec![
            KeyValue::new("Foo", AnyValue::new_string("hello")),
            KeyValue::new("Bar", AnyValue::new_string("world")),
            KeyValue::new("Baz", AnyValue::new_string("test")),
        ]);
        run_kql_test(
            project_input,
            "source | project Foo, NewBar = strcat(Bar, '!')",
            |decoded| {
                let log_record = &decoded.resource_logs[0].scope_logs[0].log_records[0];
                let foo_attr = log_record
                    .attributes
                    .iter()
                    .find(|kv| kv.key == "Foo")
                    .expect("Foo not found");
                assert_eq!(
                    foo_attr.value.as_ref().unwrap().value.as_ref().unwrap(),
                    &StringValue("hello".to_string())
                );
                let new_bar_attr = log_record
                    .attributes
                    .iter()
                    .find(|kv| kv.key == "NewBar")
                    .expect("NewBar not found");
                assert_eq!(
                    new_bar_attr.value.as_ref().unwrap().value.as_ref().unwrap(),
                    &StringValue("world!".to_string())
                );
                assert!(
                    log_record
                        .attributes
                        .iter()
                        .find(|kv| kv.key == "Bar")
                        .is_none(),
                    "Bar should not exist"
                );
                assert!(
                    log_record
                        .attributes
                        .iter()
                        .find(|kv| kv.key == "Baz")
                        .is_none(),
                    "Baz should not exist"
                );
            },
        );

        // Test project-away (remove specified columns)
        let project_away_input = build_log_with_attrs(vec![
            KeyValue::new("Foo", AnyValue::new_string("hello")),
            KeyValue::new("Bar", AnyValue::new_string("world")),
            KeyValue::new("Baz", AnyValue::new_string("test")),
        ]);
        run_kql_test(
            project_away_input,
            "source | project-away Bar, Baz",
            |decoded| {
                let log_record = &decoded.resource_logs[0].scope_logs[0].log_records[0];
                let foo_attr = log_record
                    .attributes
                    .iter()
                    .find(|kv| kv.key == "Foo")
                    .expect("Foo should exist");
                assert_eq!(
                    foo_attr.value.as_ref().unwrap().value.as_ref().unwrap(),
                    &StringValue("hello".to_string())
                );
                assert!(
                    log_record
                        .attributes
                        .iter()
                        .find(|kv| kv.key == "Bar")
                        .is_none(),
                    "Bar should be removed"
                );
                assert!(
                    log_record
                        .attributes
                        .iter()
                        .find(|kv| kv.key == "Baz")
                        .is_none(),
                    "Baz should be removed"
                );
            },
        );

        // Test project-rename (rename columns)
        let project_rename_input = build_log_with_attrs(vec![
            KeyValue::new("Foo", AnyValue::new_string("hello")),
            KeyValue::new("Bar", AnyValue::new_string("world")),
        ]);
        run_kql_test(
            project_rename_input,
            "source | project-rename NewFoo = Foo, NewBar = Bar",
            |decoded| {
                let log_record = &decoded.resource_logs[0].scope_logs[0].log_records[0];
                let new_foo_attr = log_record
                    .attributes
                    .iter()
                    .find(|kv| kv.key == "NewFoo")
                    .expect("NewFoo should exist");
                assert_eq!(
                    new_foo_attr.value.as_ref().unwrap().value.as_ref().unwrap(),
                    &StringValue("hello".to_string())
                );
                let new_bar_attr = log_record
                    .attributes
                    .iter()
                    .find(|kv| kv.key == "NewBar")
                    .expect("NewBar should exist");
                assert_eq!(
                    new_bar_attr.value.as_ref().unwrap().value.as_ref().unwrap(),
                    &StringValue("world".to_string())
                );
                assert!(
                    log_record
                        .attributes
                        .iter()
                        .find(|kv| kv.key == "Foo")
                        .is_none(),
                    "Foo should be renamed"
                );
                assert!(
                    log_record
                        .attributes
                        .iter()
                        .find(|kv| kv.key == "Bar")
                        .is_none(),
                    "Bar should be renamed"
                );
            },
        );
    }

    #[test]
    fn test_attributes_syntax() {
        // Test that Attributes['key'] syntax is identical to just 'key'
        let input = build_log_request(vec![
            LogRecord::build()
                .time_unix_nano(1u64)
                .attributes(vec![
                    KeyValue::new("Status", AnyValue::new_string("error")),
                    KeyValue::new("Priority", AnyValue::new_int(10)),
                ])
                .finish(),
            LogRecord::build()
                .time_unix_nano(2u64)
                .attributes(vec![
                    KeyValue::new("Status", AnyValue::new_string("warning")),
                    KeyValue::new("Priority", AnyValue::new_int(5)),
                ])
                .finish(),
            LogRecord::build()
                .time_unix_nano(3u64)
                .attributes(vec![
                    KeyValue::new("Status", AnyValue::new_string("info")),
                    KeyValue::new("Priority", AnyValue::new_int(3)),
                ])
                .finish(),
        ]);

        // Test where clause: direct reference vs Attributes[] syntax
        test_kql_with_count(input.clone(), "source | where Status == 'error'", 1);
        test_kql_with_count(input.clone(), "source | where Attributes['Status'] == 'error'", 1);
        
        test_kql_with_count(input.clone(), "source | where Priority > 5", 1);
        test_kql_with_count(input.clone(), "source | where Attributes['Priority'] > 5", 1);

        // Test extend: both syntaxes should work identically
        run_kql_test(
            input.clone(),
            "source | extend NewStatus = Status | project NewStatus",
            |decoded| {
                let log_record = &decoded.resource_logs[0].scope_logs[0].log_records[0];
                let attr = log_record.attributes.iter().find(|kv| kv.key == "NewStatus").expect("NewStatus not found");
                assert_eq!(attr.value.as_ref().unwrap().value.as_ref().unwrap(),
                    &StringValue("error".to_string()));
            },
        );

        run_kql_test(
            input.clone(),
            "source | extend NewStatus = Attributes['Status'] | project NewStatus",
            |decoded| {
                let log_record = &decoded.resource_logs[0].scope_logs[0].log_records[0];
                let attr = log_record.attributes.iter().find(|kv| kv.key == "NewStatus").expect("NewStatus not found");
                assert_eq!(attr.value.as_ref().unwrap().value.as_ref().unwrap(),
                    &StringValue("error".to_string()));
            },
        );

        // Test summarize: grouping by direct vs Attributes[] syntax
        run_kql_test(
            input.clone(),
            "source | summarize Count = count() by Status",
            |decoded| {
                let summary_records = &decoded.resource_logs[1].scope_logs[0].log_records;
                assert_eq!(summary_records.len(), 3);
            },
        );

        run_kql_test(
            input,
            "source | summarize Count = count() by Attributes['Status']",
            |decoded| {
                let summary_records = &decoded.resource_logs[1].scope_logs[0].log_records;
                assert_eq!(summary_records.len(), 3);
            },
        );
    }

    #[test]
    fn test_summarize_basic() {
        // Test basic aggregation with count() and group by
        let input = build_log_request(vec![
            LogRecord::build()
                .time_unix_nano(1u64)
                .attributes(vec![
                    KeyValue::new("Level", AnyValue::new_string("error")),
                    KeyValue::new("Priority", AnyValue::new_int(8)),
                ])
                .finish(),
            LogRecord::build()
                .time_unix_nano(2u64)
                .attributes(vec![
                    KeyValue::new("Level", AnyValue::new_string("info")),
                    KeyValue::new("Priority", AnyValue::new_int(3)),
                ])
                .finish(),
            LogRecord::build()
                .time_unix_nano(3u64)
                .attributes(vec![
                    KeyValue::new("Level", AnyValue::new_string("info")),
                    KeyValue::new("Priority", AnyValue::new_int(5)),
                ])
                .finish(),
        ]);

        run_kql_test(
            input,
            "source | summarize Count = count(), MaxPriority = max(Priority) by Level | where Count > 1",
            |decoded| {
                assert_eq!(decoded.resource_logs.len(), 2);
                assert_eq!(decoded.resource_logs[0].scope_logs[0].log_records.len(), 0);

                let summary_records = &decoded.resource_logs[1].scope_logs[0].log_records;
                assert_eq!(
                    summary_records.len(),
                    1,
                    "Should only have info level (Count > 1)"
                );

                let record = &summary_records[0];
                let level_attr = record
                    .attributes
                    .iter()
                    .find(|kv| kv.key == "Level")
                    .expect("Level not found");
                assert_eq!(
                    level_attr.value.as_ref().unwrap().value.as_ref().unwrap(),
                    &StringValue("info".to_string())
                );

                let count_attr = record
                    .attributes
                    .iter()
                    .find(|kv| kv.key == "Count")
                    .expect("Count not found");
                assert_eq!(
                    count_attr.value.as_ref().unwrap().value.as_ref().unwrap(),
                    &IntValue(2)
                );

                let max_priority_attr = record
                    .attributes
                    .iter()
                    .find(|kv| kv.key == "MaxPriority")
                    .expect("MaxPriority not found");
                assert_eq!(
                    max_priority_attr
                        .value
                        .as_ref()
                        .unwrap()
                        .value
                        .as_ref()
                        .unwrap(),
                    &IntValue(5)
                );
            },
        );
    }

    #[test]
    fn test_summarize_bin_timestamp() {
        use std::time::{SystemTime, UNIX_EPOCH};

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos() as u64;
        let minute_nanos = 60_000_000_000u64;
        let base_time = (now / minute_nanos) * minute_nanos;

        let input = build_log_request(vec![
            LogRecord::build()
                .time_unix_nano(base_time)
                .attributes(vec![KeyValue::new("Priority", AnyValue::new_int(5))])
                .finish(),
            LogRecord::build()
                .time_unix_nano(base_time + 1000)
                .attributes(vec![KeyValue::new("Priority", AnyValue::new_int(8))])
                .finish(),
            LogRecord::build()
                .time_unix_nano(base_time + minute_nanos)
                .attributes(vec![KeyValue::new("Priority", AnyValue::new_int(3))])
                .finish(),
        ]);

        run_kql_test(
            input,
            "source | summarize MaxPriority = max(Priority) by bin(Timestamp, 1m)",
            |decoded| {
                assert_eq!(decoded.resource_logs.len(), 2);
                assert_eq!(decoded.resource_logs[0].scope_logs[0].log_records.len(), 0);

                let summary_records = &decoded.resource_logs[1].scope_logs[0].log_records;
                assert_eq!(summary_records.len(), 2);

                let priorities: Vec<i64> = summary_records
                    .iter()
                    .map(|r| {
                        let attr = r
                            .attributes
                            .iter()
                            .find(|kv| kv.key == "MaxPriority")
                            .expect("MaxPriority not found");
                        match attr.value.as_ref().unwrap().value.as_ref().unwrap() {
                            IntValue(v) => *v,
                            _ => panic!("MaxPriority should be int"),
                        }
                    })
                    .collect();

                assert!(priorities.contains(&8), "Missing first minute (priority 8)");
                assert!(
                    priorities.contains(&3),
                    "Missing second minute (priority 3)"
                );
            },
        );
    }

    #[test]
    fn test_summarize_bin_custom_timestamp() {
        use std::time::{SystemTime, UNIX_EPOCH};

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let base_time_secs = (now / 60) * 60;
        let base_time_rfc3339 = chrono::DateTime::from_timestamp(base_time_secs as i64, 0)
            .unwrap()
            .to_rfc3339_opts(chrono::SecondsFormat::Secs, true);
        let next_minute_rfc3339 = chrono::DateTime::from_timestamp((base_time_secs + 60) as i64, 0)
            .unwrap()
            .to_rfc3339_opts(chrono::SecondsFormat::Secs, true);

        let input = build_log_request(vec![
            LogRecord::build()
                .time_unix_nano(1u64)
                .attributes(vec![KeyValue::new(
                    "CustomTimestamp",
                    AnyValue::new_string(&base_time_rfc3339),
                )])
                .finish(),
            LogRecord::build()
                .time_unix_nano(2u64)
                .attributes(vec![KeyValue::new(
                    "CustomTimestamp",
                    AnyValue::new_string(&base_time_rfc3339),
                )])
                .finish(),
            LogRecord::build()
                .time_unix_nano(3u64)
                .attributes(vec![KeyValue::new(
                    "CustomTimestamp",
                    AnyValue::new_string(&next_minute_rfc3339),
                )])
                .finish(),
        ]);

        // This test hinges on the ability of engine to recognize CustomTimestamp as a DateTime
        // Because of the input schema, query doesn't need to use todatetime() wrapper
        let bridge_options = json!({
            "attributes_schema": {
                "schema": {
                    "CustomTimestamp": "DateTime"
                },
                "options": {
                    "allow_undefined_keys": true
                }
            }
        });

        run_kql_test_with_bridge_options(
            input,
            "source | summarize EventCount = count() by bin(CustomTimestamp, 1m)",
            Some(bridge_options),
            move |decoded| {
                assert_eq!(decoded.resource_logs.len(), 2);
                let summary_records = &decoded.resource_logs[1].scope_logs[0].log_records;
                assert_eq!(summary_records.len(), 2);

                let counts: Vec<i64> = summary_records
                    .iter()
                    .map(|r| {
                        let attr = r
                            .attributes
                            .iter()
                            .find(|kv| kv.key == "EventCount")
                            .expect("EventCount not found");
                        match attr.value.as_ref().unwrap().value.as_ref().unwrap() {
                            IntValue(v) => *v,
                            _ => panic!("EventCount should be int"),
                        }
                    })
                    .collect();

                assert!(counts.contains(&2), "Should have a bin with count=2");
                assert!(counts.contains(&1), "Should have a bin with count=1");
            },
        );
    }
}
