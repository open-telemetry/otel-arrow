// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

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
fn test_summarize_count_only() {
    let mut request = ExportLogsServiceRequest::new().with_resource_logs(
        ResourceLogs::new().with_scope_logs(
            ScopeLogs::new()
                .with_log_record(LogRecord::new())
                .with_log_record(LogRecord::new()),
        ),
    );

    let query = "source | summarize Count = count()";

    let pipeline = data_engine_recordset_otlp_bridge::parse_kql_query_into_pipeline(query).unwrap();

    let engine = RecordSetEngine::new_with_options(
        RecordSetEngineOptions::new()
            .with_diagnostic_level(RecordSetEngineDiagnosticLevel::Verbose),
    );

    let results = process_records(&pipeline, &engine, &mut request);

    assert_eq!(results.summaries.len(), 1);
    assert_eq!(results.included_records.len(), 0);
    assert_eq!(results.dropped_records.len(), 2);

    let summary = results.summaries.first().unwrap();

    assert_eq!(
        "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
        summary.summary_id
    );

    assert!(summary.group_by_values.is_empty());
    assert_eq!(summary.aggregation_values.len(), 1);

    let (key, value) = summary.aggregation_values.iter().next().unwrap();

    assert_eq!("Count", key.as_ref());

    if let SummaryAggregation::Count(v) = value {
        assert_eq!(2, *v);
    } else {
        panic!()
    }
}

#[test]
fn test_summarize_count_and_group_by() {
    let mut request = ExportLogsServiceRequest::new().with_resource_logs(
        ResourceLogs::new().with_scope_logs(
            ScopeLogs::new()
                .with_log_record(LogRecord::new().with_body(AnyValue::Native(
                    OtlpAnyValue::StringValue(StringValueStorage::new("hello world".into())),
                )))
                .with_log_record(LogRecord::new().with_body(AnyValue::Native(
                    OtlpAnyValue::StringValue(StringValueStorage::new("hello world".into())),
                )))
                .with_log_record(LogRecord::new().with_body(AnyValue::Native(
                    OtlpAnyValue::StringValue(StringValueStorage::new("goodbye world".into())),
                ))),
        ),
    );

    let query = "source | summarize Count = count() by Body";

    let pipeline = data_engine_recordset_otlp_bridge::parse_kql_query_into_pipeline(query).unwrap();

    let engine = RecordSetEngine::new_with_options(
        RecordSetEngineOptions::new()
            .with_diagnostic_level(RecordSetEngineDiagnosticLevel::Verbose),
    );

    let results = process_records(&pipeline, &engine, &mut request);

    assert_eq!(results.summaries.len(), 2);
    assert_eq!(results.included_records.len(), 0);
    assert_eq!(results.dropped_records.len(), 3);

    let mut summaries = results.summaries;
    summaries.sort_by(|l, r| l.summary_id.cmp(&r.summary_id));

    assert_summary(
        &summaries[0],
        "33072fa213f92249f89e47b5c5a3959191dd8a72662068a2d44f5e66d579e09c",
        "goodbye world",
        1,
    );

    assert_summary(
        &summaries[1],
        "daa898d673bd24e7a11ff9724e4f549c88eaa763d12da08762fcb73e8337a37f",
        "hello world",
        2,
    );

    fn assert_summary(summary: &RecordSetEngineSummary, sumary_id: &str, body: &str, count: usize) {
        assert_eq!(sumary_id, summary.summary_id);

        assert_eq!(summary.group_by_values.len(), 1);

        let (key, value) = &summary.group_by_values[0];

        assert_eq!("Body", key.as_ref());

        assert_eq!(
            OwnedValue::String(StringValueStorage::new(body.into())).to_value(),
            value.to_value()
        );

        assert_eq!(summary.aggregation_values.len(), 1);

        let (key, value) = summary.aggregation_values.iter().next().unwrap();

        assert_eq!("Count", key.as_ref());

        if let SummaryAggregation::Count(v) = value {
            assert_eq!(count, *v);
        } else {
            panic!()
        }
    }
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

#[test]
fn test_substring_function() {
    let run_test = |statement: &str, expected: &str| {
        let log = LogRecord::new().with_attribute(
            "greeting",
            AnyValue::Native(OtlpAnyValue::StringValue(StringValueStorage::new(
                "hello world".into(),
            ))),
        );

        let mut request = ExportLogsServiceRequest::new().with_resource_logs(
            ResourceLogs::new().with_scope_logs(ScopeLogs::new().with_log_record(log)),
        );

        let pipeline = data_engine_recordset_otlp_bridge::parse_kql_query_into_pipeline(
            format!("source | extend e = {statement}").as_str(),
        )
        .unwrap();

        let engine = RecordSetEngine::new_with_options(
            RecordSetEngineOptions::new()
                .with_diagnostic_level(RecordSetEngineDiagnosticLevel::Verbose),
        );

        let results = process_records(&pipeline, &engine, &mut request);

        assert_eq!(results.included_records.len(), 1);
        assert_eq!(results.dropped_records.len(), 0);

        let log = results.included_records.first().unwrap().get_record();

        assert_eq!(
            expected,
            log.attributes.get("e").unwrap().to_value().to_string()
        );
    };

    run_test("substring(Attributes['greeting'], 6)", "world");
    run_test("substring(Attributes['greeting'], 0, 5)", "hello");
}

#[test]
fn test_coalesce_function() {
    let run_test = |statement: &str, expected: &str| {
        let log = LogRecord::new()
            .with_attribute("null_key1", AnyValue::Null)
            .with_attribute(
                "string_key1",
                AnyValue::Native(OtlpAnyValue::StringValue(StringValueStorage::new(
                    "hello world".into(),
                ))),
            );

        let mut request = ExportLogsServiceRequest::new().with_resource_logs(
            ResourceLogs::new().with_scope_logs(ScopeLogs::new().with_log_record(log)),
        );

        let pipeline = data_engine_recordset_otlp_bridge::parse_kql_query_into_pipeline(
            format!("source | extend e = {statement}").as_str(),
        )
        .unwrap();

        let engine = RecordSetEngine::new_with_options(
            RecordSetEngineOptions::new()
                .with_diagnostic_level(RecordSetEngineDiagnosticLevel::Verbose),
        );

        let results = process_records(&pipeline, &engine, &mut request);

        assert_eq!(results.included_records.len(), 1);
        assert_eq!(results.dropped_records.len(), 0);

        let log = results.included_records.first().unwrap().get_record();

        assert_eq!(
            expected,
            log.attributes.get("e").unwrap().to_value().to_string()
        );
    };

    run_test("coalesce('hello', 'world')", "hello");
    run_test("coalesce(Attributes['null_key1'], 'world')", "world");
    run_test(
        "coalesce(Attributes['null_key1'], Attributes['null_key1'], 'world')",
        "world",
    );
    run_test(
        "coalesce(Attributes['null_key1'], Attributes['null_key1'], Attributes['null_key1'])",
        "null",
    );
    run_test(
        "coalesce(Attributes['null_key1'], Attributes['string_key1'])",
        "hello world",
    );
    run_test("coalesce(tolong('invalid'), 18)", "18");
}

#[test]
fn test_parse_function() {
    let log = LogRecord::new().with_attribute(
        "message",
        AnyValue::Native(OtlpAnyValue::StringValue(StringValueStorage::new(
            "Event: NotifySliceRelease".into(),
        ))),
    );

    let mut request = ExportLogsServiceRequest::new().with_resource_logs(
        ResourceLogs::new().with_scope_logs(ScopeLogs::new().with_log_record(log)),
    );

    let query = "source | parse Attributes['message'] with \"Event:\" resourceName:string";

    let pipeline =
        data_engine_recordset_otlp_bridge::parse_kql_query_into_pipeline(&query).unwrap();

    let engine = RecordSetEngine::new_with_options(
        RecordSetEngineOptions::new()
            .with_diagnostic_level(RecordSetEngineDiagnosticLevel::Verbose),
    );

    let results = process_records(&pipeline, &engine, &mut request);

    assert_eq!(results.included_records.len(), 1);
    assert_eq!(results.dropped_records.len(), 0);

    let log = results.included_records.first().unwrap().get_record();

    // This would validate that the parsed value was extracted and stored
    assert_eq!(
        " NotifySliceRelease",
        log.attributes
            .get("resourceName")
            .unwrap()
            .to_value()
            .to_string()
    );
}

#[test]
fn test_matches_regex_function() {
    let log1 = LogRecord::new().with_attribute(
        "message",
        AnyValue::Native(OtlpAnyValue::StringValue(StringValueStorage::new(
            "Event: NotifySliceRelease".into(),
        ))),
    );

    let log2 = LogRecord::new().with_attribute(
        "message",
        AnyValue::Native(OtlpAnyValue::StringValue(StringValueStorage::new(
            "Warning: Something went wrong".into(),
        ))),
    );

    let log3 = LogRecord::new().with_attribute(
        "message",
        AnyValue::Native(OtlpAnyValue::StringValue(StringValueStorage::new(
            "Event: DataProcessed".into(),
        ))),
    );

    let mut request = ExportLogsServiceRequest::new().with_resource_logs(
        ResourceLogs::new().with_scope_logs(
            ScopeLogs::new()
                .with_log_record(log1)
                .with_log_record(log2)
                .with_log_record(log3),
        ),
    );

    let query = "source | where Attributes['message'] matches regex 'Event: .*'";

    let pipeline =
        data_engine_recordset_otlp_bridge::parse_kql_query_into_pipeline(&query).unwrap();

    let engine = RecordSetEngine::new_with_options(
        RecordSetEngineOptions::new()
            .with_diagnostic_level(RecordSetEngineDiagnosticLevel::Verbose),
    );

    let results = process_records(&pipeline, &engine, &mut request);

    // Should include only the two logs that match the regex pattern
    assert_eq!(results.included_records.len(), 2);
    assert_eq!(results.dropped_records.len(), 1);

    // Verify the included records contain the expected messages
    let included_messages: Vec<String> = results
        .included_records
        .iter()
        .map(|record| {
            record
                .get_record()
                .attributes
                .get("message")
                .unwrap()
                .to_value()
                .to_string()
        })
        .collect();

    assert!(included_messages.contains(&"Event: NotifySliceRelease".to_string()));
    assert!(included_messages.contains(&"Event: DataProcessed".to_string()));
    assert!(!included_messages.contains(&"Warning: Something went wrong".to_string()));
}
