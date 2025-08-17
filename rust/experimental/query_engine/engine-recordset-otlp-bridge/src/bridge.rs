use std::sync::{LazyLock, RwLock};

use data_engine_expressions::*;
use data_engine_kql_parser::*;
use data_engine_recordset::*;

use crate::*;

const CRATE_VERSION: &str = env!("CARGO_PKG_VERSION");
const OTLP_BUFFER_INITIAL_CAPACITY: usize = 1024 * 64;

static EXPRESSIONS: LazyLock<RwLock<Vec<PipelineExpression>>> =
    LazyLock::new(|| RwLock::new(Vec::new()));

pub fn parse_kql_query_into_pipeline(query: &str) -> Result<PipelineExpression, Vec<ParserError>> {
    let options = ParserOptions::new()
        .with_attached_data_names(&["resource", "instrumentation_scope", "scope"])
        .with_source_map_schema(
            ParserMapSchema::new()
                .set_default_map_key("Attributes")
                .with_key_definition("Timestamp", ParserMapKeySchema::DateTime)
                .with_key_definition("ObservedTimestamp", ParserMapKeySchema::DateTime)
                .with_key_definition("SeverityNumber", ParserMapKeySchema::Integer)
                .with_key_definition("SeverityText", ParserMapKeySchema::String)
                .with_key_definition("Body", ParserMapKeySchema::Any)
                .with_key_definition("Attributes", ParserMapKeySchema::Map)
                .with_key_definition("TraceId", ParserMapKeySchema::Array)
                .with_key_definition("SpanId", ParserMapKeySchema::Array)
                .with_key_definition("TraceFlags", ParserMapKeySchema::Integer)
                .with_key_definition("EventName", ParserMapKeySchema::String),
        );

    KqlParser::parse_with_options(query, options)
}

pub fn register_pipeline_for_kql_query(query: &str) -> Result<usize, Vec<ParserError>> {
    let pipeline = parse_kql_query_into_pipeline(query);

    match pipeline {
        Ok(p) => {
            let mut expressions = EXPRESSIONS.write().unwrap();
            expressions.push(p);
            Ok(expressions.len() - 1)
        }
        Err(e) => Err(e),
    }
}

pub fn process_protobuf_otlp_export_logs_service_request_using_registered_pipeline(
    pipeline: usize,
    log_level: RecordSetEngineDiagnosticLevel,
    export_logs_service_request_protobuf_data: &[u8],
) -> Result<(Vec<u8>, Vec<u8>), BridgeError> {
    let expressions = EXPRESSIONS.read().unwrap();

    let pipeline_expression = expressions.get(pipeline);

    if pipeline_expression.is_none() {
        return Err(BridgeError::PipelineNotFound(pipeline));
    }

    let request =
        ExportLogsServiceRequest::from_protobuf(export_logs_service_request_protobuf_data);

    if let Err(e) = request {
        return Err(BridgeError::OtlpProtobufReadError(e));
    }

    match process_export_logs_service_request_using_pipeline(
        pipeline_expression.unwrap(),
        log_level,
        request.unwrap(),
    ) {
        Ok((included, dropped)) => {
            let mut included_records_otlp_response = Vec::new();
            if let Some(ref included) = included {
                match ExportLogsServiceRequest::to_protobuf(included, OTLP_BUFFER_INITIAL_CAPACITY)
                {
                    Ok(r) => {
                        included_records_otlp_response = r.to_vec();
                    }
                    Err(e) => {
                        return Err(BridgeError::OtlpProtobufWriteError(e));
                    }
                }
            }

            let mut dropped_records_otlp_response = Vec::new();
            if let Some(ref dropped) = dropped {
                match ExportLogsServiceRequest::to_protobuf(dropped, OTLP_BUFFER_INITIAL_CAPACITY) {
                    Ok(r) => {
                        dropped_records_otlp_response = r.to_vec();
                    }
                    Err(e) => {
                        return Err(BridgeError::OtlpProtobufWriteError(e));
                    }
                }
            }

            Ok((
                included_records_otlp_response,
                dropped_records_otlp_response,
            ))
        }
        Err(e) => Err(e),
    }
}

pub fn process_export_logs_service_request_using_pipeline(
    pipeline: &PipelineExpression,
    log_level: RecordSetEngineDiagnosticLevel,
    mut export_logs_service_request: ExportLogsServiceRequest,
) -> Result<
    (
        Option<ExportLogsServiceRequest>,
        Option<ExportLogsServiceRequest>,
    ),
    BridgeError,
> {
    let engine = RecordSetEngine::new_with_options(
        RecordSetEngineOptions::new().with_diagnostic_level(log_level),
    );

    let mut batch = engine
        .begin_batch(pipeline)
        .map_err(|e| BridgeError::PipelineInitializationError(e.to_string()))?;

    let dropped_records = batch.push_records(&mut export_logs_service_request);

    let mut final_results = batch.flush();

    for record in dropped_records.into_iter() {
        final_results.dropped_records.push(record);
    }

    let mut dropped_records = None;

    if !final_results.dropped_records.is_empty() {
        let mut dropped_records_request = ExportLogsServiceRequest::new();

        for original_resource_logs in &export_logs_service_request.resource_logs {
            let mut resource_logs = ResourceLogs::new();

            resource_logs.resource = original_resource_logs.resource.clone();
            resource_logs.extra_fields = original_resource_logs.extra_fields.clone();

            for original_scope_logs in &original_resource_logs.scope_logs {
                let mut scope_logs = ScopeLogs::new();

                scope_logs.instrumentation_scope =
                    original_scope_logs.instrumentation_scope.clone();
                scope_logs.extra_fields = original_scope_logs.extra_fields.clone();

                resource_logs.scope_logs.push(scope_logs);
            }

            dropped_records_request.resource_logs.push(resource_logs);
        }

        process_log_record_results(&mut dropped_records_request, final_results.dropped_records);

        dropped_records = Some(dropped_records_request);
    }

    let mut included_records = None;

    let has_summaries = !final_results.summaries.is_empty();
    let has_included_records = !final_results.included_records.is_empty();
    if has_summaries || has_included_records {
        if has_included_records {
            process_log_record_results(
                &mut export_logs_service_request,
                final_results.included_records,
            );
        }

        if has_summaries {
            let mut log_records = Vec::new();

            for summary in final_results.summaries {
                let mut attributes: Vec<(Box<str>, AnyValue)> = Vec::with_capacity(
                    summary.aggregation_values.len() + summary.group_by_values.len(),
                );

                for (key, value) in summary.group_by_values {
                    attributes.push((key, value.into()));
                }

                for (key, value) in summary.aggregation_values {
                    match value {
                        SummaryAggregation::Average { count, sum } => {
                            let avg = sum.to_double() / count as f64;

                            attributes.push((
                                key,
                                AnyValue::Native(OtlpAnyValue::DoubleValue(
                                    DoubleValueStorage::new(avg),
                                )),
                            ));
                        }
                        SummaryAggregation::Count(v) => {
                            attributes.push((
                                key,
                                AnyValue::Native(OtlpAnyValue::IntValue(IntegerValueStorage::new(
                                    v as i64,
                                ))),
                            ));
                        }
                        SummaryAggregation::Maximum(v) | SummaryAggregation::Minimum(v) => {
                            attributes.push((key, v.into()));
                        }
                        SummaryAggregation::Sum(v) => {
                            let v = match v {
                                SummaryValue::Double(d) => AnyValue::Native(
                                    OtlpAnyValue::DoubleValue(DoubleValueStorage::new(d)),
                                ),
                                SummaryValue::Integer(i) => AnyValue::Native(
                                    OtlpAnyValue::IntValue(IntegerValueStorage::new(i)),
                                ),
                            };
                            attributes.push((key, v));
                        }
                    }
                }

                log_records.push(LogRecord::new().with_attributes(attributes));
            }

            let summary_otlp = ResourceLogs::new().with_scope_logs(
                ScopeLogs::new()
                    .with_instrumentation_scope(
                        InstrumentationScope::new()
                            .with_name("query_engine.otlp_bridge".into())
                            .with_version(CRATE_VERSION.into()),
                    )
                    .with_log_records(log_records),
            );

            export_logs_service_request.resource_logs.push(summary_otlp);
        }

        included_records = Some(export_logs_service_request);
    }

    Ok((included_records, dropped_records))
}

fn process_log_record_results(
    response: &mut ExportLogsServiceRequest,
    records: Vec<RecordSetEngineRecord<LogRecord>>,
) {
    for record_result in records {
        let mut diagnostic_output = None;

        let diagnostics = record_result.get_diagnostics();
        if !diagnostics.is_empty() {
            diagnostic_output = Some(record_result.to_string());
        }

        let mut log_record = record_result.take_record();
        if let Some(d) = diagnostic_output {
            log_record.attributes.get_values_mut().insert(
                "query_engine.output".into(),
                AnyValue::Native(OtlpAnyValue::StringValue(StringValueStorage::new(d))),
            );
        }

        let resource_id = log_record
            .resource_id
            .expect("resource_id was not set on log record");

        let resource_logs = response
            .resource_logs
            .get_mut(resource_id)
            .expect("resource_logs were not found");

        let scope_id = log_record
            .scope_id
            .expect("scope_id was not set on log record");

        let scope_logs = resource_logs
            .scope_logs
            .get_mut(scope_id)
            .expect("scope_logs were not found");

        scope_logs.log_records.push(log_record);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use bytes::BytesMut;
    use prost::Message;

    type OtlpExportLogsServiceRequest =
        opentelemetry_proto::tonic::collector::logs::v1::ExportLogsServiceRequest;
    type OtlpResourceLogs = opentelemetry_proto::tonic::logs::v1::ResourceLogs;
    type OtlpScopeLogs = opentelemetry_proto::tonic::logs::v1::ScopeLogs;
    type OtlpLogRecord = opentelemetry_proto::tonic::logs::v1::LogRecord;
    type OtlpResource = opentelemetry_proto::tonic::resource::v1::Resource;
    type OtlpEntityRef = opentelemetry_proto::tonic::common::v1::EntityRef;
    type OtlpInstrumentationScope = opentelemetry_proto::tonic::common::v1::InstrumentationScope;
    type OtlpKeyValue = opentelemetry_proto::tonic::common::v1::KeyValue;
    type OtlpAnyValue = opentelemetry_proto::tonic::common::v1::AnyValue;
    type OtlpValue = opentelemetry_proto::tonic::common::v1::any_value::Value;

    #[test]
    fn test_otlp_round_trip() {
        let request = OtlpExportLogsServiceRequest {
            resource_logs: vec![
                OtlpResourceLogs {
                    resource: Some(OtlpResource {
                        attributes: vec![OtlpKeyValue {
                            key: "key1".into(),
                            value: Some(OtlpAnyValue {
                                value: Some(OtlpValue::IntValue(18)),
                            }),
                        }],
                        dropped_attributes_count: 18,
                        entity_refs: vec![OtlpEntityRef {
                            schema_url: "SchemaA".into(),
                            r#type: "service".into(),
                            id_keys: vec!["key1".into()],
                            description_keys: vec![],
                        }],
                    }),
                    scope_logs: vec![
                        OtlpScopeLogs {
                            scope: Some(OtlpInstrumentationScope {
                                name: "name".into(),
                                version: "version".into(),
                                attributes: vec![OtlpKeyValue {
                                    key: "key1".into(),
                                    value: Some(OtlpAnyValue {
                                        value: Some(OtlpValue::IntValue(18)),
                                    }),
                                }],
                                dropped_attributes_count: 18,
                            }),
                            log_records: vec![OtlpLogRecord {
                                time_unix_nano: 1,
                                observed_time_unix_nano: 1,
                                severity_number: 1,
                                severity_text: "Info".into(),
                                body: Some(OtlpAnyValue {
                                    value: Some(OtlpValue::StringValue("value".into())),
                                }),
                                attributes: vec![OtlpKeyValue {
                                    key: "key1".into(),
                                    value: Some(OtlpAnyValue {
                                        value: Some(OtlpValue::IntValue(18)),
                                    }),
                                }],
                                dropped_attributes_count: 18,
                                flags: 1,
                                trace_id: vec![0x00, 0xFF],
                                span_id: vec![0x00, 0xFF],
                                event_name: "eventname".into(),
                            }],
                            schema_url: "".into(),
                        },
                        OtlpScopeLogs {
                            scope: None,
                            log_records: vec![],
                            schema_url: "SchemaB".into(),
                        },
                    ],
                    schema_url: "".into(),
                },
                OtlpResourceLogs {
                    resource: None,
                    scope_logs: vec![],
                    schema_url: "SchemaA".into(),
                },
            ],
        };

        let mut protobuf_data = BytesMut::new();

        request.encode(&mut protobuf_data).unwrap();

        let protobuf_data = protobuf_data.freeze();

        {
            // Include everything
            let pipeline_id = register_pipeline_for_kql_query("s | where true").unwrap();

            let (included, _) =
                process_protobuf_otlp_export_logs_service_request_using_registered_pipeline(
                    pipeline_id,
                    RecordSetEngineDiagnosticLevel::Error,
                    &protobuf_data,
                )
                .unwrap();

            let response = OtlpExportLogsServiceRequest::decode(&included[..]).unwrap();

            assert_eq!(request, response);
        }

        {
            // Drop everything
            let pipeline_id = register_pipeline_for_kql_query("s | where false").unwrap();

            let (_, dropped) =
                process_protobuf_otlp_export_logs_service_request_using_registered_pipeline(
                    pipeline_id,
                    RecordSetEngineDiagnosticLevel::Error,
                    &protobuf_data,
                )
                .unwrap();

            let response = OtlpExportLogsServiceRequest::decode(&dropped[..]).unwrap();

            assert_eq!(request, response);
        }
    }

    #[test]
    fn test_process_parsed_export_logs_service_request_all_dropped() {
        let request = ExportLogsServiceRequest::new().with_resource_logs(
            ResourceLogs::new().with_scope_logs(ScopeLogs::new().with_log_record(LogRecord::new())),
        );

        let pipeline = parse_kql_query_into_pipeline("source | where false").unwrap();

        let (included_records, dropped_records) =
            process_export_logs_service_request_using_pipeline(
                &pipeline,
                RecordSetEngineDiagnosticLevel::Verbose,
                request,
            )
            .unwrap();

        assert!(included_records.is_none());
        assert!(dropped_records.is_some());
    }

    #[test]
    fn test_process_parsed_export_logs_service_request_all_included() {
        let request = ExportLogsServiceRequest::new().with_resource_logs(
            ResourceLogs::new().with_scope_logs(ScopeLogs::new().with_log_record(LogRecord::new())),
        );

        let pipeline = parse_kql_query_into_pipeline("source | where true").unwrap();

        let (included_records, dropped_records) =
            process_export_logs_service_request_using_pipeline(
                &pipeline,
                RecordSetEngineDiagnosticLevel::Verbose,
                request,
            )
            .unwrap();

        assert!(included_records.is_some());
        assert!(dropped_records.is_none());
    }

    #[test]
    fn test_process_parsed_export_logs_service_request_summary() {
        let request = ExportLogsServiceRequest::new().with_resource_logs(
            ResourceLogs::new().with_scope_logs(ScopeLogs::new().with_log_record(LogRecord::new())),
        );

        let pipeline = parse_kql_query_into_pipeline("source | summarize Count = count()").unwrap();

        let (included_records, dropped_records) =
            process_export_logs_service_request_using_pipeline(
                &pipeline,
                RecordSetEngineDiagnosticLevel::Verbose,
                request,
            )
            .unwrap();

        assert!(included_records.is_some());
        assert!(dropped_records.is_some());
    }
}
