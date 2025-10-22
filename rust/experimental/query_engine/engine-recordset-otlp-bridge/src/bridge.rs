// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use std::sync::{LazyLock, RwLock};

use data_engine_expressions::*;
use data_engine_kql_parser::*;
use data_engine_recordset::*;

use crate::*;

const CRATE_VERSION: &str = env!("CARGO_PKG_VERSION");
const OTLP_BUFFER_INITIAL_CAPACITY: usize = 1024 * 64;

static EXPRESSIONS: LazyLock<RwLock<Vec<(ParserOptions, PipelineExpression)>>> =
    LazyLock::new(|| RwLock::new(Vec::new()));

pub fn parse_kql_query_into_pipeline(
    query: &str,
    options: Option<BridgeOptions>,
) -> Result<PipelineExpression, Vec<ParserError>> {
    KqlParser::parse_with_options(query, build_parser_options(options).map_err(|e| vec![e])?)
}

pub fn register_pipeline_for_kql_query(
    query: &str,
    options: Option<BridgeOptions>,
) -> Result<usize, Vec<ParserError>> {
    let options = build_parser_options(options).map_err(|e| vec![e])?;

    let pipeline = KqlParser::parse_with_options(query, options.clone())?;

    let mut expressions = EXPRESSIONS.write().unwrap();
    expressions.push((options, pipeline));
    Ok(expressions.len() - 1)
}

pub fn process_protobuf_otlp_export_logs_service_request_using_registered_pipeline(
    pipeline: usize,
    log_level: RecordSetEngineDiagnosticLevel,
    export_logs_service_request_protobuf_data: &[u8],
) -> Result<(Vec<u8>, Vec<u8>), BridgeError> {
    let expressions = EXPRESSIONS.read().unwrap();

    let pipeline_registration = expressions.get(pipeline);

    let (options, pipeline) = match pipeline_registration {
        Some(v) => v,
        None => return Err(BridgeError::PipelineNotFound(pipeline)),
    };

    let request =
        ExportLogsServiceRequest::from_protobuf(export_logs_service_request_protobuf_data);

    if let Err(e) = request {
        return Err(BridgeError::OtlpProtobufReadError(e));
    }

    match process_export_logs_service_request_using_pipeline(
        Some(options),
        pipeline,
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
    options: Option<&ParserOptions>,
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

    if let Some(options) = options
        && let Some(ParserMapKeySchema::Map(Some(attributes_schema))) = options
            .get_source_map_schema()
            .and_then(|s| s.get_schema_for_key("Attributes"))
    {
        // Note: This block is a not-so-elegant fix to OTLP not supporting
        // roundtrip of extended types. What we do is if we know something is a
        // DateTime via the schema and the value is a string we will try to
        // convert to a real DateTime.
        let schema = attributes_schema.get_schema();

        for resource in &mut export_logs_service_request.resource_logs {
            for scope in &mut resource.scope_logs {
                for log in &mut scope.log_records {
                    let attributes = log.attributes.get_values_mut();

                    for (key, schema) in schema {
                        match schema {
                            ParserMapKeySchema::DateTime => {
                                if let Some(v) = attributes.get(key)
                                    && v.get_value_type() == ValueType::String
                                {
                                    if let Some(d) = Value::convert_to_datetime(&v.to_value()) {
                                        attributes.insert(
                                            key.clone(),
                                            AnyValue::Extended(ExtendedValue::DateTime(
                                                DateTimeValueStorage::new(d),
                                            )),
                                        );
                                    } else {
                                        attributes.remove(key);
                                    }
                                }
                            }
                            _ => {
                                // todo: Fix up Regex & TimeSpan types and look into maps if we have sub-schema
                            }
                        }
                    }
                }
            }
        }
    }

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

    let has_summaries = !final_results.summaries.included_summaries.is_empty();
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

            for summary in final_results.summaries.included_summaries {
                let diagnostics = summary.to_string();

                if let Some(map) = summary.map {
                    let mut attributes: Vec<(Box<str>, AnyValue)> =
                        Vec::with_capacity(map.len() + 1);

                    attributes.extend(
                        map.take_values()
                            .drain()
                            .map(|(key, value)| (key, value.into())),
                    );

                    if !diagnostics.is_empty() {
                        attributes.push((
                            "query_engine.output".into(),
                            AnyValue::Native(OtlpAnyValue::StringValue(StringValueStorage::new(
                                diagnostics,
                            ))),
                        ));
                    }

                    log_records.push(LogRecord::new().with_attributes(attributes));
                } else {
                    let mut attributes: Vec<(Box<str>, AnyValue)> = Vec::with_capacity(
                        summary.aggregation_values.len() + summary.group_by_values.len() + 1,
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
                                    AnyValue::Native(OtlpAnyValue::IntValue(
                                        IntegerValueStorage::new(v as i64),
                                    )),
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

                    if !diagnostics.is_empty() {
                        attributes.push((
                            "query_engine.output".into(),
                            AnyValue::Native(OtlpAnyValue::StringValue(StringValueStorage::new(
                                diagnostics,
                            ))),
                        ));
                    }

                    log_records.push(LogRecord::new().with_attributes(attributes));
                }
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

fn build_parser_options(options: Option<BridgeOptions>) -> Result<ParserOptions, ParserError> {
    let mut parser_options = ParserOptions::new().with_attached_data_names(&[
        "resource",
        "instrumentation_scope",
        "scope",
    ]);

    let (log_record_schema, summary_schema) =
        build_log_record_schema(options.and_then(|mut v| v.take_attributes_schema()))?;

    if let Some(summary_schema) = summary_schema {
        parser_options = parser_options.with_summary_map_schema(summary_schema);
    }

    Ok(parser_options.with_source_map_schema(log_record_schema))
}

fn build_log_record_schema(
    attributes_schema: Option<ParserMapSchema>,
) -> Result<(ParserMapSchema, Option<ParserMapSchema>), ParserError> {
    let mut log_record_schema = ParserMapSchema::new()
        .set_default_map_key("Attributes")
        .with_key_definition("Timestamp", ParserMapKeySchema::DateTime)
        .with_key_definition("ObservedTimestamp", ParserMapKeySchema::DateTime)
        .with_key_definition("SeverityNumber", ParserMapKeySchema::Integer)
        .with_key_definition("SeverityText", ParserMapKeySchema::String)
        .with_key_definition("Body", ParserMapKeySchema::Any)
        .with_key_definition("TraceId", ParserMapKeySchema::Array)
        .with_key_definition("SpanId", ParserMapKeySchema::Array)
        .with_key_definition("TraceFlags", ParserMapKeySchema::Integer)
        .with_key_definition("EventName", ParserMapKeySchema::String);

    if let Some(mut attributes_schema) = attributes_schema {
        let schema = attributes_schema.get_schema_mut();
        for (top_level_key, top_level_key_schema) in log_record_schema.get_schema() {
            // Note: If any top-level fields are duplicated on Attributes Schema
            // they get removed automatically. This is done for two purposes.
            // The first is to make it easy for callers to pass in something
            // like a table schema. Many backends flatten log records into
            // columns. This feature is essentially a convenience thing so
            // callers with table schema don't need to map columns back to the
            // log record schema. The second reason is to prevent
            // accidental\confusing query results. If for example "Body" is
            // present in Attributes users might query with ambiguous naming.
            // For example: source | extend Body = 'something' will write to the
            // top-level field and not Attributes.
            if let Some(removed) = schema.remove(top_level_key)
                && &removed != top_level_key_schema
            {
                return Err(ParserError::SchemaError(format!(
                    "'{top_level_key}' key cannot be declared as '{}' type",
                    &removed
                )));
            }
        }

        let allow_undefined_keys = attributes_schema.get_allow_undefined_keys();

        log_record_schema = log_record_schema.with_key_definition(
            "Attributes",
            ParserMapKeySchema::Map(Some(attributes_schema)),
        );

        let mut summary_schema = ParserMapSchema::new();

        if allow_undefined_keys {
            summary_schema = summary_schema.set_allow_undefined_keys();
        }

        for (top_level_key, top_level_key_schema) in log_record_schema.get_schema() {
            if top_level_key.as_ref() == "Attributes" {
                if let ParserMapKeySchema::Map(Some(attributes_schema)) = top_level_key_schema {
                    for (top_level_key, top_level_key_schema) in attributes_schema.get_schema() {
                        summary_schema = summary_schema
                            .with_key_definition(top_level_key, top_level_key_schema.clone());
                    }
                }
                continue;
            }
            summary_schema =
                summary_schema.with_key_definition(top_level_key, top_level_key_schema.clone());
        }

        return Ok((log_record_schema, Some(summary_schema)));
    }

    Ok((log_record_schema, None))
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
            let pipeline_id = register_pipeline_for_kql_query("s | where true", None).unwrap();

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
            let pipeline_id = register_pipeline_for_kql_query("s | where false", None).unwrap();

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

        let pipeline = parse_kql_query_into_pipeline("source | where false", None).unwrap();

        let (included_records, dropped_records) =
            process_export_logs_service_request_using_pipeline(
                None,
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

        let pipeline = parse_kql_query_into_pipeline("source | where true", None).unwrap();

        let (included_records, dropped_records) =
            process_export_logs_service_request_using_pipeline(
                None,
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

        let pipeline =
            parse_kql_query_into_pipeline("source | summarize Count = count()", None).unwrap();

        let (included_records, dropped_records) =
            process_export_logs_service_request_using_pipeline(
                None,
                &pipeline,
                RecordSetEngineDiagnosticLevel::Verbose,
                request,
            )
            .unwrap();

        assert!(included_records.is_some());
        assert!(dropped_records.is_some());
    }

    #[test]
    fn test_parse_kql_query_into_pipeline_with_attributes_schema() {
        let run_test_success = |query: &str| {
            println!("Testing: {query}");

            parse_kql_query_into_pipeline(
                query,
                Some(
                    BridgeOptions::new().with_attributes_schema(
                        ParserMapSchema::new()
                            .with_key_definition("Body", ParserMapKeySchema::Any)
                            .with_key_definition("int_value", ParserMapKeySchema::Integer),
                    ),
                ),
            )
            .unwrap();
        };

        let run_test_failure = |query: &str| {
            println!("Testing: {query}");

            parse_kql_query_into_pipeline(
                query,
                Some(
                    BridgeOptions::new().with_attributes_schema(
                        ParserMapSchema::new()
                            .with_key_definition("Body", ParserMapKeySchema::Any)
                            .with_key_definition("int_value", ParserMapKeySchema::Integer),
                    ),
                ),
            )
            .unwrap_err();
        };

        run_test_success("source | extend int_value = 1234");
        run_test_failure("source | extend Custom = 1234");

        run_test_success("source | summarize int_value = count()");
        run_test_success("source | summarize by int_value");
        run_test_success("source | summarize by int_value | extend int_value = 1");

        run_test_failure("source | summarize by unknown");
        run_test_failure("source | summarize by unknown = int_value");
        run_test_failure("source | summarize Count = count()");
        run_test_failure("source | summarize int_value = count() | extend Custom = 1234");

        run_test_success(
            "source | summarize by int_value | extend int_value = 1 | summarize int_value = count()",
        );
        run_test_failure(
            "source | summarize by int_value | extend int_value = 1 | summarize Count = count()",
        );

        run_test_success(
            "source | summarize by int_value | extend int_value = 1 | summarize int_value = count() | extend int_value = 1234",
        );
        run_test_failure(
            "source | summarize by int_value | extend int_value = 1 | summarize int_value = count() | extend Custom = 1234",
        );

        run_test_success("source | extend Body = 'hello world'");
        // Note: Body gets removed from Attributes schema because it is defined at the root
        run_test_failure("source | extend Attributes.Body = 'hello world'");
    }

    #[test]
    fn test_process_parsed_export_logs_service_request_with_attributes_schema() {
        let request = ExportLogsServiceRequest::new().with_resource_logs(
            ResourceLogs::new().with_scope_logs(ScopeLogs::new().with_log_record(
                LogRecord::new().with_attribute(
                    "TimeGenerated",
                    AnyValue::Native(crate::OtlpAnyValue::StringValue(StringValueStorage::new(
                        "10/22/2025".into(),
                    ))),
                ),
            )),
        );

        let options = build_parser_options(Some(
            BridgeOptions::new().with_attributes_schema(
                ParserMapSchema::new()
                    .with_key_definition("TimeGenerated", ParserMapKeySchema::DateTime),
            ),
        ))
        .unwrap();

        let pipeline = KqlParser::parse_with_options(
            "source | where gettype(TimeGenerated) == 'datetime'",
            options.clone(),
        )
        .unwrap();

        let (included_records, dropped_records) =
            process_export_logs_service_request_using_pipeline(
                Some(&options),
                &pipeline,
                RecordSetEngineDiagnosticLevel::Verbose,
                request,
            )
            .unwrap();

        assert!(included_records.is_some());
        assert!(dropped_records.is_none());
    }

    #[test]
    fn test_parse_kql_query_into_pipeline_with_attributes_schema_and_allow_undefined_keys() {
        let run_test_success = |query: &str| {
            parse_kql_query_into_pipeline(
                query,
                Some(
                    BridgeOptions::new().with_attributes_schema(
                        ParserMapSchema::new()
                            .with_key_definition("Body", ParserMapKeySchema::Any)
                            .with_key_definition("int_value", ParserMapKeySchema::Integer)
                            .set_allow_undefined_keys(),
                    ),
                ),
            )
            .unwrap();
        };

        run_test_success("source | extend int_value = 1234");
        run_test_success("source | extend Custom = 1234");

        run_test_success("source | summarize by int_value");
        run_test_success("source | summarize by unknown");
    }

    #[test]
    fn test_parse_kql_query_into_pipeline_with_attributes_schema_error() {
        let e = parse_kql_query_into_pipeline(
            "",
            Some(BridgeOptions::new().with_attributes_schema(
                ParserMapSchema::new().with_key_definition("Body", ParserMapKeySchema::Map(None)),
            )),
        )
        .unwrap_err();

        assert_eq!(1, e.len());
        assert!(matches!(e[0], ParserError::SchemaError(_)));
    }
}
