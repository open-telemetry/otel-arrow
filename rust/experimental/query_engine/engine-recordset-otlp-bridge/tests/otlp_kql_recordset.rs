use std::collections::HashMap;

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

    /* todo: Use KQL when parsing is done:
    let query = "source | summarize Count = count()";

    let pipeline = data_engine_recordset_otlp_bridge::parse_kql_query_into_pipeline(query).unwrap();*/

    let pipeline = PipelineExpressionBuilder::new(" ")
        .with_expressions(vec![DataExpression::Summary(SummaryDataExpression::new(
            QueryLocation::new_fake(),
            HashMap::from([(
                "Count".into(),
                AggregationExpression::new(
                    QueryLocation::new_fake(),
                    AggregationFunction::Count,
                    None,
                ),
            )]),
            HashMap::new(),
        ))])
        .build()
        .unwrap();

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

    /* todo: Use KQL when parsing is done:
    let query = "source | summarize Count = count() by Body";

    let pipeline = data_engine_recordset_otlp_bridge::parse_kql_query_into_pipeline(query).unwrap();*/

    let pipeline = PipelineExpressionBuilder::new(" ")
        .with_expressions(vec![DataExpression::Summary(SummaryDataExpression::new(
            QueryLocation::new_fake(),
            HashMap::from([(
                "Count".into(),
                AggregationExpression::new(
                    QueryLocation::new_fake(),
                    AggregationFunction::Count,
                    None,
                ),
            )]),
            HashMap::from([(
                "Body".into(),
                ScalarExpression::Source(SourceScalarExpression::new(
                    QueryLocation::new_fake(),
                    ValueAccessor::new_with_selectors(vec![ScalarExpression::Static(
                        StaticScalarExpression::String(StringScalarExpression::new(
                            QueryLocation::new_fake(),
                            "Body",
                        )),
                    )]),
                )),
            )]),
        ))])
        .build()
        .unwrap();

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

        let group_by = &summary.group_by_values["Body"];

        assert_eq!(
            OwnedValue::String(StringValueStorage::new(body.into())).to_value(),
            group_by.to_value()
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
