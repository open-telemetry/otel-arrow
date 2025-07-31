use data_engine_expressions::*;
use data_engine_recordset_otlp_bridge::*;
use data_engine_recordset2::*;

use crate::common::*;

pub mod common;

#[test]
fn test_project_keep() {
    let log = LogRecord::new()
        .with_event_name("event_name".into())
        .with_attribute("key1", AnyValue::Null)
        .with_attribute("name", AnyValue::Null);

    let mut request = ExportLogsServiceRequest::new().with_resource_logs(
        ResourceLogs::new().with_scope_logs(ScopeLogs::new().with_log_record(log)),
    );

    let query = "source\n | project-keep key*";

    let pipeline = data_engine_recordset_otlp_bridge::parse_kql_query_into_pipeline(query).unwrap();

    let engine = RecordSetEngine::new_with_options(
        RecordSetEngineOptions::new()
            .with_diagnostic_level(RecordSetEngineDiagnosticLevel::Verbose),
    );

    let results = process_records(&pipeline, &engine, &mut request);

    assert_eq!(results.included_records.len(), 1);
    assert_eq!(results.dropped_records.len(), 0);

    let log = results.included_records.first().unwrap().get_record();

    assert!(log.event_name.is_none());

    let attributes = log.attributes.get_values();
    assert_eq!(attributes.len(), 1);
    assert_eq!(
        attributes.get("key1").map(|v| v.to_value().to_string()),
        Some(AnyValue::Null.to_value().to_string())
    );
}

#[test]
fn test_project_away() {
    let log = LogRecord::new()
        .with_event_name("event_name".into())
        .with_attribute("key1", AnyValue::Null)
        .with_attribute("name", AnyValue::Null);

    let mut request = ExportLogsServiceRequest::new().with_resource_logs(
        ResourceLogs::new().with_scope_logs(ScopeLogs::new().with_log_record(log)),
    );

    let query = "source\n | project-away key*";

    let pipeline = data_engine_recordset_otlp_bridge::parse_kql_query_into_pipeline(query).unwrap();

    let engine = RecordSetEngine::new_with_options(
        RecordSetEngineOptions::new()
            .with_diagnostic_level(RecordSetEngineDiagnosticLevel::Verbose),
    );

    let results = process_records(&pipeline, &engine, &mut request);

    assert_eq!(results.included_records.len(), 1);
    assert_eq!(results.dropped_records.len(), 0);

    let log = results.included_records.first().unwrap().get_record();

    assert!(log.event_name.is_some());

    let attributes = log.attributes.get_values();
    assert_eq!(attributes.len(), 1);
    assert_eq!(
        attributes.get("name").map(|v| v.to_value().to_string()),
        Some(AnyValue::Null.to_value().to_string())
    );
}
