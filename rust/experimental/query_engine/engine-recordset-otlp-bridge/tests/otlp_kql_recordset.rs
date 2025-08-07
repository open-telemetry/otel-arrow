use data_engine_expressions::*;
use data_engine_recordset::*;
use data_engine_recordset_otlp_bridge::*;

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

#[test]
fn test_strlen_function() {
    let log = LogRecord::new()
        .with_event_name("hello world".into())
        .with_attribute(
            "text",
            AnyValue::Native(OtlpAnyValue::StringValue(StringValueStorage::new(
                "test string".into(),
            ))),
        );

    let mut request = ExportLogsServiceRequest::new().with_resource_logs(
        ResourceLogs::new().with_scope_logs(ScopeLogs::new().with_log_record(log)),
    );

    let query = "source\n | extend name_length = strlen(EventName), text_length = strlen(text)";

    let pipeline = data_engine_recordset_otlp_bridge::parse_kql_query_into_pipeline(query).unwrap();

    let engine = RecordSetEngine::new_with_options(
        RecordSetEngineOptions::new()
            .with_diagnostic_level(RecordSetEngineDiagnosticLevel::Verbose),
    );

    let results = process_records(&pipeline, &engine, &mut request);

    assert_eq!(results.included_records.len(), 1);
    assert_eq!(results.dropped_records.len(), 0);

    let log = results.included_records.first().unwrap().get_record();

    let attributes = log.attributes.get_values();
    assert_eq!(
        attributes
            .get("name_length")
            .map(|v| v.to_value().to_string()),
        Some(
            AnyValue::Native(OtlpAnyValue::IntValue(IntegerValueStorage::new(11)))
                .to_value()
                .to_string()
        ) // "hello world" has 11 characters
    );
    assert_eq!(
        attributes
            .get("text_length")
            .map(|v| v.to_value().to_string()),
        Some(
            AnyValue::Native(OtlpAnyValue::IntValue(IntegerValueStorage::new(11)))
                .to_value()
                .to_string()
        ) // "test string" has 11 characters
    );
}

#[test]
fn test_replace_string_function() {
    let log = LogRecord::new()
        .with_event_name("A magic trick can turn a cat into a dog".into())
        .with_attribute(
            "text",
            AnyValue::Native(OtlpAnyValue::StringValue(StringValueStorage::new(
                "hello world hello".into(),
            ))),
        );

    let mut request = ExportLogsServiceRequest::new().with_resource_logs(
        ResourceLogs::new().with_scope_logs(ScopeLogs::new().with_log_record(log)),
    );

    let query = r#"source
 | extend
     modified_name = replace_string(EventName, "cat", "hamster"),
     modified_text = replace_string(text, "hello", "hi")"#;

    let pipeline = data_engine_recordset_otlp_bridge::parse_kql_query_into_pipeline(query).unwrap();

    let engine = RecordSetEngine::new_with_options(
        RecordSetEngineOptions::new()
            .with_diagnostic_level(RecordSetEngineDiagnosticLevel::Verbose),
    );

    let results = process_records(&pipeline, &engine, &mut request);

    assert_eq!(results.included_records.len(), 1);
    assert_eq!(results.dropped_records.len(), 0);

    let log = results.included_records.first().unwrap().get_record();

    let attributes = log.attributes.get_values();
    assert_eq!(
        attributes
            .get("modified_name")
            .map(|v| v.to_value().to_string()),
        Some(
            AnyValue::Native(OtlpAnyValue::StringValue(StringValueStorage::new(
                "A magic trick can turn a hamster into a dog".into()
            )))
            .to_value()
            .to_string()
        )
    );
    assert_eq!(
        attributes
            .get("modified_text")
            .map(|v| v.to_value().to_string()),
        Some(
            AnyValue::Native(OtlpAnyValue::StringValue(StringValueStorage::new(
                "hi world hi".into()
            )))
            .to_value()
            .to_string()
        )
    );
}
