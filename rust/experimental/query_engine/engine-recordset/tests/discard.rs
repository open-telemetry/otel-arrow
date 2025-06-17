use data_engine_recordset::{
    data_expressions::*, logical_expressions::*, primitives::*, value_expressions::*, *,
};

pub mod common;

#[test]
fn test_discard_simple() -> Result<(), Error> {
    let mut batch = common::TestLogRecordBatch::new();

    let mut log_record1 = common::TestLogRecord::new();

    log_record1.set_attribute("event_id", AnyValue::new_long_value(1));

    batch.add_log_record(log_record1);

    let mut pipeline = PipelineExpression::new();

    pipeline.add_data_expression(DiscardDataExpression::new());

    let data_engine = common::create_data_engine()?;

    let results = data_engine.process_complete_batch(&pipeline, &mut batch)?;

    assert_eq!(0, results.get_included_record_count());
    assert_eq!(1, results.get_dropped_record_count());

    Ok(())
}

#[test]
fn test_discard_with_predicate() -> Result<(), Error> {
    let mut batch = common::TestLogRecordBatch::new();

    let mut log_record1 = common::TestLogRecord::new();

    log_record1.set_attribute("event_id", AnyValue::new_long_value(1));

    batch.add_log_record(log_record1);

    let mut log_record2 = common::TestLogRecord::new();

    log_record2.set_attribute("event_id", AnyValue::new_long_value(2));

    batch.add_log_record(log_record2);

    let mut pipeline = PipelineExpression::new();

    pipeline.add_data_expression(DiscardDataExpression::new_with_predicate(
        EqualToLogicalExpression::new(
            ResolveValueExpression::new("event_id")?,
            StaticValueExpression::new(AnyValue::new_long_value(1)),
        ),
    ));

    let data_engine = common::create_data_engine()?;

    let results = data_engine.process_complete_batch(&pipeline, &mut batch)?;

    assert_eq!(1, results.get_included_record_count());
    assert_eq!(1, results.get_dropped_record_count());

    Ok(())
}
