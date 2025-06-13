use data_engine_recordset::{
    data_expressions::*, primitives::*, transform_expressions::*, value_expressions::*, *,
};

pub mod common;

#[test]
fn test_set_simple() -> Result<(), Error> {
    let mut batch = common::TestLogRecordBatch::new();

    let mut log_record1 = common::TestLogRecord::new();

    log_record1.set_attribute("event_id", AnyValue::new_long_value(1));

    batch.add_log_record(log_record1);

    let mut transformations = TransformDataExpression::new();

    transformations.add_transformation_expression(SetTransformationExpression::new(
        ResolveValueExpression::new("my_attribute")?,
        StaticValueExpression::new(AnyValue::new_string_value("hello world")),
    ));

    let mut pipeline = PipelineExpression::new();

    pipeline.add_data_expression(transformations);

    let data_engine = common::create_data_engine()?;

    let results = data_engine.process_complete_batch(&pipeline, &mut batch)?;

    assert_eq!(1, results.get_included_record_count());

    let (included, _) = common::unwrap_results(results);

    let attributes = included[0].get_attributes_map();

    assert_eq!(2, attributes.get_values().len());

    assert_eq!(
        1,
        attributes.get_long("event_id").expect("event_id not found")
    );
    assert_eq!(
        "hello world",
        attributes
            .get_string("my_attribute")
            .expect("my_attribute not found")
    );

    Ok(())
}
