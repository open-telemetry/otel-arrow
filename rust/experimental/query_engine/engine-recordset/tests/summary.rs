use std::time::SystemTime;

use chrono::{DateTime, Timelike, Utc};

use data_engine_recordset::{
    data::*, data_expressions::*, logical_expressions::*, primitives::*, value_expressions::*, *,
};

pub mod common;

#[test]
fn test_summary() -> Result<(), Error> {
    run_summary_test(1)
}

#[test]
fn test_summary_with_reservoir() -> Result<(), Error> {
    run_summary_test(10)
}

#[test]
fn test_summary_on_incoming() -> Result<(), Error> {
    run_summary_on_incoming_test(1)
}

#[test]
fn test_summary_on_incoming_with_reservoir() -> Result<(), Error> {
    run_summary_on_incoming_test(10)
}

fn run_summary_test(reservoir_size: u32) -> Result<(), Error> {
    let mut resource = common::TestResource::new();

    resource.set_attribute("service.name", AnyValue::new_string_value("MyService"));

    let instrumentation_scope = common::TestInstrumentationScope::new("CategoryNameA");

    let mut batch =
        common::TestLogRecordBatch::new_with_attached(&resource, &instrumentation_scope);

    let timestamp = SystemTime::now();
    for i in 0..100 {
        let mut log_record_event_id_1 = common::TestLogRecord::new_with_timestamp(timestamp);

        log_record_event_id_1.set_attribute("event_id", AnyValue::new_long_value(1));
        log_record_event_id_1.set_attribute("id", AnyValue::new_long_value(i));

        batch.add_log_record(log_record_event_id_1);
    }

    let mut log_record_event_id_2 = common::TestLogRecord::new_with_timestamp(timestamp);

    log_record_event_id_2.set_attribute("event_id", AnyValue::new_long_value(2));
    log_record_event_id_2.set_attribute("id", AnyValue::new_long_value(-1));

    batch.add_log_record(log_record_event_id_2);

    let mut pipeline = PipelineExpression::new();

    let mut summary_predicate = LogicalGroupExpression::new(EqualToLogicalExpression::new(
        ResolveFromAttachedValueExpression::new("resource", "@attributes:'service.name'")?,
        StaticValueExpression::new(AnyValue::new_string_value("MyService")),
    ));

    summary_predicate.add_logical_expression_with_and(EqualToLogicalExpression::new(
        StaticValueExpression::new(AnyValue::new_string_value("CategoryNameA")),
        ResolveFromAttachedValueExpression::new("instrumentation_scope", "@name")?,
    ));

    let mut summary = SummarizeByDataExpression::new_with_predicate(
        SummaryWindow::new_timestamp_based(5000),
        SummaryReservoir::new_simple(reservoir_size),
        summary_predicate,
    );

    summary.add_value_expression(ResolveValueExpression::new("event_id").unwrap());

    pipeline.add_data_expression(summary);

    let data_engine = common::create_data_engine()?;

    let results = data_engine.process_complete_batch(&pipeline, &mut batch)?;

    assert_eq!(reservoir_size + 1, results.get_included_record_count());
    assert_eq!(2, results.get_summaries().len());

    let summaries = results.get_summaries().to_vec();
    let (included, _) = common::unwrap_results(results);

    let validate = |record_index: usize,
                    event_id_value: i64,
                    included_count_value: u32,
                    total_count_value: u32| {
        let log_record = &included[record_index];

        let attributes = log_record.get_attributes_map();

        assert_eq!(2, attributes.get_values().len());

        let summary_id = log_record.get_summary_id();

        assert!(summary_id.is_some());

        let summary = summaries
            .iter()
            .find(|v| v.get_id() == summary_id.unwrap())
            .expect("Summary not found");

        assert_eq!(included_count_value, summary.get_included_count());
        assert_eq!(total_count_value, summary.get_total_count());

        let grouping = summary.get_grouping();
        assert_eq!(3, grouping.len());

        let group1 = &grouping[0];

        assert_eq!(
            "resource",
            group1.get_key().get_name().expect("Name not found")
        );
        assert_eq!("@attributes:'service.name'", group1.get_key().get_path());
        assert_eq!(
            "MyService",
            group1
                .get_value()
                .to_string_value()
                .expect("Value was not a string")
        );

        let group2 = &grouping[1];

        assert_eq!(
            "instrumentation_scope",
            group2.get_key().get_name().expect("Name not found")
        );
        assert_eq!("@name", group2.get_key().get_path());
        assert_eq!(
            "CategoryNameA",
            group2
                .get_value()
                .to_string_value()
                .expect("Value was not a string")
        );

        let group3 = &grouping[2];

        assert_eq!(None, group3.get_key().get_name());
        assert_eq!("event_id", group3.get_key().get_path());
        assert_eq!(
            event_id_value,
            group3
                .get_value()
                .to_long_value()
                .expect("Value was not a long")
        );
    };

    validate(0, 1, reservoir_size, 100);
    validate(reservoir_size as usize, 2, 1, 1);

    Ok(())
}

fn run_summary_on_incoming_test(reservoir_size: u32) -> Result<(), Error> {
    let timestamp = SystemTime::now();

    let timestamp_utc: DateTime<Utc> = timestamp.into();
    let seconds_from_midnight = timestamp_utc.num_seconds_from_midnight() as i64;
    let window_start =
        timestamp_utc.timestamp() - seconds_from_midnight + (seconds_from_midnight % 5000);

    let mut resource = common::TestResource::new();

    resource.set_attribute("service.name", AnyValue::new_string_value("MyService"));

    let instrumentation_scope = common::TestInstrumentationScope::new("CategoryNameA");

    let mut batch =
        common::TestLogRecordBatch::new_with_attached(&resource, &instrumentation_scope);

    let grouping = vec![
        SummaryGroupKeyValue::new(
            SummaryGroupKey::new(Some("resource"), "@attributes:'service.name'"),
            SummaryGroupValue::new_from_any_value(&AnyValue::new_string_value("MyService")),
        ),
        SummaryGroupKeyValue::new(
            SummaryGroupKey::new(Some("instrumentation_scope"), "@name"),
            SummaryGroupValue::new_from_any_value(&AnyValue::new_string_value("CategoryNameA")),
        ),
        SummaryGroupKeyValue::new(
            SummaryGroupKey::new(None, "event_id"),
            SummaryGroupValue::new_from_any_value(&AnyValue::new_long_value(1)),
        ),
    ];

    let summary1 = Summary::new(
        None,
        SystemTime::now(),
        SummaryWindow::Timestamp(5000),
        window_start,
        window_start + 5000,
        SummaryReservoir::SimpleReservoir(1),
        grouping.clone(),
        1,
        100,
    );

    let summary1_id: Box<str> = summary1.get_id().into();

    batch.add_summary(summary1);

    let mut log_record1_for_summary1 = common::TestLogRecord::new_with_timestamp(timestamp);

    log_record1_for_summary1.set_attribute("event_id", AnyValue::new_long_value(1));

    log_record1_for_summary1.set_summary_id(&summary1_id);

    batch.add_log_record(log_record1_for_summary1);

    let summary2 = Summary::new(
        None,
        SystemTime::now(),
        SummaryWindow::Timestamp(5000),
        window_start,
        window_start + 5000,
        SummaryReservoir::SimpleReservoir(1),
        grouping.clone(),
        1,
        1000,
    );

    let summary2_id: Box<str> = summary2.get_id().into();

    batch.add_summary(summary2);

    let mut log_record1_for_summary2 = common::TestLogRecord::new_with_timestamp(timestamp);

    log_record1_for_summary2.set_attribute("event_id", AnyValue::new_long_value(1));

    log_record1_for_summary2.set_summary_id(&summary2_id);

    batch.add_log_record(log_record1_for_summary2);

    for i in 0..100 {
        let mut log_record_event_id_1 = common::TestLogRecord::new_with_timestamp(timestamp);

        log_record_event_id_1.set_attribute("event_id", AnyValue::new_long_value(1));
        log_record_event_id_1.set_attribute("id", AnyValue::new_long_value(i));

        batch.add_log_record(log_record_event_id_1);
    }

    let mut log_record_event_id_2 = common::TestLogRecord::new_with_timestamp(timestamp);

    log_record_event_id_2.set_attribute("event_id", AnyValue::new_long_value(2));
    log_record_event_id_2.set_attribute("id", AnyValue::new_long_value(-1));

    batch.add_log_record(log_record_event_id_2);

    let mut pipeline = PipelineExpression::new();

    let mut summary_predicate = LogicalGroupExpression::new(EqualToLogicalExpression::new(
        ResolveFromAttachedValueExpression::new("resource", "@attributes:'service.name'")?,
        StaticValueExpression::new(AnyValue::new_string_value("MyService")),
    ));

    summary_predicate.add_logical_expression_with_and(EqualToLogicalExpression::new(
        StaticValueExpression::new(AnyValue::new_string_value("CategoryNameA")),
        ResolveFromAttachedValueExpression::new("instrumentation_scope", "@name")?,
    ));

    let mut summary = SummarizeByDataExpression::new_with_predicate(
        SummaryWindow::new_timestamp_based(5000),
        SummaryReservoir::new_simple(reservoir_size),
        summary_predicate,
    );

    summary.add_value_expression(ResolveValueExpression::new("event_id").unwrap());

    pipeline.add_data_expression(summary);

    let data_engine = common::create_data_engine()?;

    let results = data_engine.process_complete_batch(&pipeline, &mut batch)?;

    assert_eq!(2 + reservoir_size + 1, results.get_included_record_count());

    let summaries = results.get_summaries().to_vec();

    assert_eq!(2, summaries.len());
    assert_eq!(2 + reservoir_size, summaries[0].get_included_count());
    assert_eq!(1, summaries[1].get_included_count());
    assert_eq!(1200, summaries[0].get_total_count());

    let (included, _) = common::unwrap_results(results);

    for data_record in included {
        let event_id = data_record
            .get_attribute("event_id")
            .expect("event_id not found")
            .get_long_value()
            .expect("event_id was not a long value");

        match event_id {
            1 => assert_eq!(
                summaries[0].get_id(),
                data_record
                    .get_summary_id()
                    .expect("summary_id not set on data record")
            ),
            2 => assert_eq!(
                summaries[1].get_id(),
                data_record
                    .get_summary_id()
                    .expect("summary_id not set on data record")
            ),
            _ => panic!("event_id value was unexpected"),
        }
    }

    Ok(())
}
