use std::sync::{LazyLock, RwLock};

use data_engine_expressions::PipelineExpression;
use data_engine_kql_parser::*;
use data_engine_recordset2::*;

use crate::*;

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
            if included.is_some() {
                match ExportLogsServiceRequest::to_protobuf(included.as_ref().unwrap(), 8192) {
                    Ok(r) => {
                        included_records_otlp_response = r.to_vec();
                    }
                    Err(e) => {
                        return Err(BridgeError::OtlpProtobufWriteError(e));
                    }
                }
            }

            let mut dropped_records_otlp_response = Vec::new();
            if dropped.is_some() {
                match ExportLogsServiceRequest::to_protobuf(dropped.as_ref().unwrap(), 8192) {
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

    let mut batch = engine.begin_batch(pipeline);

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

    if !final_results.included_records.is_empty() {
        process_log_record_results(
            &mut export_logs_service_request,
            final_results.included_records,
        );
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
                AnyValue::Native(OtlpAnyValue::StringValue(ValueStorage::new(d))),
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
}
