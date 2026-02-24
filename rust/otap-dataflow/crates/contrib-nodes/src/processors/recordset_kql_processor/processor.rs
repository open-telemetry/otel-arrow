// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use super::config::RecordsetKqlProcessorConfig;
use super::create_recordset_kql_processor;
use otap_df_otap::pdata::OtapPdata;

use async_trait::async_trait;
use data_engine_recordset::RecordSetEngineDiagnosticLevel;
use data_engine_recordset_otlp_bridge::{
    BridgeDiagnosticOptions, BridgeError, BridgeOptions, BridgePipeline,
    parse_kql_query_into_pipeline,
    process_protobuf_otlp_export_logs_service_request_using_pipeline,
};
use linkme::distributed_slice;
use otap_df_config::SignalType;
use otap_df_config::error::Error as ConfigError;
use otap_df_engine::{
    ConsumerEffectHandlerExtension, ProcessorFactory,
    context::PipelineContext,
    control::NackMsg,
    error::Error,
    local::processor::{EffectHandler, Processor},
    message::Message,
};
use otap_df_pdata::{OtapPayload, OtlpProtoBytes};

/// URN identifier for the processor
pub const RECORDSET_KQL_PROCESSOR_URN: &str = "urn:microsoft:recordset_kql:processor";

/// OTAP KQL Processor
#[allow(unsafe_code)]
#[distributed_slice(otap_df_otap::OTAP_PROCESSOR_FACTORIES)]
pub static RECORDSET_KQL_PROCESSOR_FACTORY: ProcessorFactory<OtapPdata> = ProcessorFactory {
    name: RECORDSET_KQL_PROCESSOR_URN,
    create: create_recordset_kql_processor,
    wiring_contract: otap_df_engine::wiring_contract::WiringContract::UNRESTRICTED,
    validate_config: otap_df_config::validation::validate_typed_config::<RecordsetKqlProcessorConfig>,
};

/// KQL processor that applies KQL queries to telemetry data
pub struct RecordsetKqlProcessor {
    config: RecordsetKqlProcessorConfig,
    pipeline: BridgePipeline,
}

impl RecordsetKqlProcessor {
    /// Creates a new KQL processor
    pub fn with_pipeline_ctx(
        _pipeline_ctx: PipelineContext,
        config: RecordsetKqlProcessorConfig,
    ) -> Result<Self, ConfigError> {
        let parsed_bridge_options = Self::parse_bridge_options(&config.bridge_options)?;
        let pipeline = parse_kql_query_into_pipeline(
            &config.query,
            Some(Self::apply_bridge_options_defaults(parsed_bridge_options)),
        )
        .map_err(|errors| ConfigError::InvalidUserConfig {
            error: format!("Failed to parse KQL query: {:?}", errors),
        })?;

        otap_df_telemetry::otel_info!("recordset_kql_processor.ready");

        Ok(Self { config, pipeline })
    }

    /// Parse bridge options from JSON value
    pub fn parse_bridge_options(
        bridge_options_json: &Option<serde_json::Value>,
    ) -> Result<Option<BridgeOptions>, ConfigError> {
        Ok(if let Some(json_value) = bridge_options_json {
            let json_str =
                serde_json::to_string(json_value).map_err(|e| ConfigError::InvalidUserConfig {
                    error: format!("Failed to serialize bridge options: {}", e),
                })?;
            Some(BridgeOptions::from_json(&json_str).map_err(|e| {
                ConfigError::InvalidUserConfig {
                    error: format!("Failed to parse bridge options: {}", e),
                }
            })?)
        } else {
            None
        })
    }

    pub fn apply_bridge_options_defaults(options: Option<BridgeOptions>) -> BridgeOptions {
        options
            .unwrap_or_default()
            .set_include_dropped_records(false)
            .set_diagnostic_options(BridgeDiagnosticOptions::Callback(|d| {
                for diagnostic in d.get_diagnostics() {
                    let (query_line_number, query_column_number) = diagnostic
                        .get_expression()
                        .get_query_location()
                        .get_line_and_column_numbers();
                    let message = diagnostic.get_message();
                    match diagnostic.get_diagnostic_level() {
                        RecordSetEngineDiagnosticLevel::Verbose => {
                            otap_df_telemetry::otel_debug!(
                                "recordset_kql_processor.query_output",
                                query_line_number,
                                query_column_number,
                                message
                            );
                        }
                        RecordSetEngineDiagnosticLevel::Info => {
                            otap_df_telemetry::otel_info!(
                                "recordset_kql_processor.query_output",
                                query_line_number,
                                query_column_number,
                                message
                            );
                        }
                        RecordSetEngineDiagnosticLevel::Warn => {
                            otap_df_telemetry::otel_warn!(
                                "recordset_kql_processor.query_output",
                                query_line_number,
                                query_column_number,
                                message
                            );
                        }
                        RecordSetEngineDiagnosticLevel::Error => {
                            otap_df_telemetry::otel_error!(
                                "recordset_kql_processor.query_output",
                                query_line_number,
                                query_column_number,
                                message
                            );
                        }
                    }
                }
            }))
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
                    "recordset_kql_processor.processing_logs",
                    input_items
                );
                self.process_logs(bytes, signal)
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
                    "recordset_kql_processor.success",
                    input_items,
                    output_items,
                );

                let processed_data = OtapPdata::new(ctx, payload);

                effect_handler.send_message(processed_data).await?;
                Ok(())
            }
            Err(e) => {
                let message = e.to_string();
                otap_df_telemetry::otel_error!(
                    "recordset_kql_processor.failure",
                    input_items,
                    message,
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

    fn process_logs(
        &mut self,
        bytes: bytes::Bytes,
        signal: SignalType,
    ) -> Result<OtlpProtoBytes, Error> {
        let response = process_protobuf_otlp_export_logs_service_request_using_pipeline(
            &self.pipeline,
            RecordSetEngineDiagnosticLevel::Warn,
            &bytes,
        )
        .map_err(|e| Self::map_bridge_error(e, signal))?;

        let included_records = response
            .into_otlp_bytes()
            .map_err(|e| Self::map_bridge_error(e, signal))?
            .0;

        Ok(OtlpProtoBytes::ExportLogsRequest(included_records.into()))
    }

    fn map_bridge_error(error: BridgeError, signal: SignalType) -> Error {
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
                        if let Ok(new_config) =
                            serde_json::from_value::<RecordsetKqlProcessorConfig>(config)
                        {
                            // Re-parse the pipeline (if changed)
                            if new_config.query != self.config.query
                                || new_config.bridge_options != self.config.bridge_options
                            {
                                let parsed_bridge_options =
                                    match Self::parse_bridge_options(&new_config.bridge_options) {
                                        Err(e) => {
                                            otap_df_telemetry::otel_warn!(
                                                "recordset_kql_processor.reconfigure_error",
                                                message = %e
                                            );
                                            None
                                        }
                                        Ok(v) => v,
                                    };

                                match parse_kql_query_into_pipeline(
                                    &new_config.query,
                                    Some(Self::apply_bridge_options_defaults(
                                        parsed_bridge_options,
                                    )),
                                ) {
                                    Ok(pipeline) => {
                                        otap_df_telemetry::otel_info!(
                                            "recordset_kql_processor.reconfigured"
                                        );

                                        self.pipeline = pipeline;
                                        self.config = new_config;
                                    }
                                    Err(errors) => {
                                        let message =
                                            format!("Failed to parse updated query: {:?}", errors);
                                        otap_df_telemetry::otel_error!(
                                            "recordset_kql_processor.reconfigure_error",
                                            message,
                                        );
                                    }
                                }
                            } else {
                                self.config = new_config;
                            }
                        }
                        Ok(())
                    }
                    _ => Ok(()),
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use super::*;
    use bytes::BytesMut;
    use otap_df_config::node::NodeUserConfig;
    use otap_df_engine::context::ControllerContext;
    use otap_df_engine::message::Message;
    use otap_df_engine::testing::{node::test_node, processor::TestRuntime};
    use otap_df_otap::pdata::OtapPdata;
    use otap_df_pdata::OtlpProtoBytes;
    use otap_df_pdata::proto::opentelemetry::{
        collector::logs::v1::ExportLogsServiceRequest,
        common::v1::{AnyValue, InstrumentationScope, KeyValue, any_value::Value::*},
        logs::v1::{LogRecord, ResourceLogs, ScopeLogs, SeverityNumber},
        resource::v1::Resource,
    };
    use otap_df_telemetry::registry::TelemetryRegistryHandle;
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
        let telemetry_registry_handle = TelemetryRegistryHandle::new();
        let controller_ctx = ControllerContext::new(telemetry_registry_handle);
        let pipeline_ctx =
            controller_ctx.pipeline_context_with("grp".into(), "pipeline".into(), 0, 1, 0);

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
                    !log_record.attributes.iter().any(|kv| kv.key == "Bar"),
                    "Bar should not exist"
                );
                assert!(
                    !log_record.attributes.iter().any(|kv| kv.key == "Baz"),
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
                    !log_record.attributes.iter().any(|kv| kv.key == "Bar"),
                    "Bar should be removed"
                );
                assert!(
                    !log_record.attributes.iter().any(|kv| kv.key == "Baz"),
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
                    !log_record.attributes.iter().any(|kv| kv.key == "Foo"),
                    "Foo should be renamed"
                );
                assert!(
                    !log_record.attributes.iter().any(|kv| kv.key == "Bar"),
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
        test_kql_with_count(
            input.clone(),
            "source | where Attributes['Status'] == 'error'",
            1,
        );

        test_kql_with_count(input.clone(), "source | where Priority > 5", 1);
        test_kql_with_count(
            input.clone(),
            "source | where Attributes['Priority'] > 5",
            1,
        );

        // Test extend: both syntaxes should work identically
        run_kql_test(
            input.clone(),
            "source | extend NewStatus = Status | project NewStatus",
            |decoded| {
                let log_record = &decoded.resource_logs[0].scope_logs[0].log_records[0];
                let attr = log_record
                    .attributes
                    .iter()
                    .find(|kv| kv.key == "NewStatus")
                    .expect("NewStatus not found");
                assert_eq!(
                    attr.value.as_ref().unwrap().value.as_ref().unwrap(),
                    &StringValue("error".to_string())
                );
            },
        );

        run_kql_test(
            input.clone(),
            "source | extend NewStatus = Attributes['Status'] | project NewStatus",
            |decoded| {
                let log_record = &decoded.resource_logs[0].scope_logs[0].log_records[0];
                let attr = log_record
                    .attributes
                    .iter()
                    .find(|kv| kv.key == "NewStatus")
                    .expect("NewStatus not found");
                assert_eq!(
                    attr.value.as_ref().unwrap().value.as_ref().unwrap(),
                    &StringValue("error".to_string())
                );
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
