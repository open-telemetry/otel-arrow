use std::collections::HashMap;

use crate::{execution_context::*, scalars::*, *};
use data_engine_expressions::*;

pub fn execute_summary_data_expression<'a, TRecord: Record>(
    execution_context: &ExecutionContext<'a, '_, '_, TRecord>,
    summary_data_expression: &'a SummaryDataExpression,
) -> Result<(), ExpressionError> {
    let group_by_expressions = summary_data_expression.get_group_by_expressions();

    let mut group_by_values = Vec::with_capacity(group_by_expressions.len());

    for (key, expression) in group_by_expressions {
        let value = execute_scalar_expression(execution_context, expression)?;

        group_by_values.push((key.clone(), value));
    }

    let aggregation_expressions = summary_data_expression.get_aggregation_expressions();

    let mut aggregation_values = HashMap::with_capacity(aggregation_expressions.len());

    for (key, expression) in aggregation_expressions {
        let aggregation_function = expression.get_aggregation_function();

        let mut resolved_aggregate_value = None;

        if !matches!(aggregation_function, AggregationFunction::Count) {
            if let Some(e) = expression.get_value_expression() {
                resolved_aggregate_value = Some(execute_scalar_expression(execution_context, e)?);
            } else {
                execution_context.add_diagnostic_if_enabled(
                    RecordSetEngineDiagnosticLevel::Warn,
                    expression,
                    || {
                        format!(
                            "Value expression was not specified for '{aggregation_function:?}' aggregation",
                        )
                    },
                );
                continue;
            }
        }

        match aggregation_function {
            AggregationFunction::Average => {
                let aggregate_value = resolved_aggregate_value
                    .as_ref()
                    .map(|v| v.to_value())
                    .expect("Value was not resolved");

                match get_summary_value(&aggregate_value) {
                    Some(v) => {
                        aggregation_values
                            .insert(key.clone(), SummaryAggregationUpdate::Average(v));
                    }
                    None => {
                        execution_context.add_diagnostic_if_enabled(
                            RecordSetEngineDiagnosticLevel::Warn,
                            expression.get_value_expression().as_ref().unwrap(),
                            || {
                                format!(
                                    "Value expression value of '{:?}' type could not be converted to integer or double",
                                    aggregate_value.get_value_type()
                                )
                            },
                        );
                    }
                }
            }
            AggregationFunction::Count => {
                aggregation_values.insert(key.clone(), SummaryAggregationUpdate::Count);
            }
            AggregationFunction::Maximum => {
                let aggregate_value = resolved_aggregate_value.expect("Value was not resolved");

                aggregation_values.insert(
                    key.clone(),
                    SummaryAggregationUpdate::Maximum(aggregate_value),
                );
            }
            AggregationFunction::Minimum => {
                let aggregate_value = resolved_aggregate_value.expect("Value was not resolved");

                aggregation_values.insert(
                    key.clone(),
                    SummaryAggregationUpdate::Minimum(aggregate_value),
                );
            }
            AggregationFunction::Sum => {
                let aggregate_value = resolved_aggregate_value
                    .as_ref()
                    .map(|v| v.to_value())
                    .expect("Value was not resolved");

                match get_summary_value(&aggregate_value) {
                    Some(v) => {
                        aggregation_values.insert(key.clone(), SummaryAggregationUpdate::Sum(v));
                    }
                    None => {
                        execution_context.add_diagnostic_if_enabled(
                            RecordSetEngineDiagnosticLevel::Warn,
                            expression.get_value_expression().as_ref().unwrap(),
                            || {
                                format!(
                                    "Value expression value of '{:?}' type could not be converted to integer or double",
                                    aggregate_value.get_value_type()
                                )
                            },
                        );
                    }
                }
            }
        };
    }

    execution_context.get_summaries().create_or_update_summary(
        execution_context,
        summary_data_expression,
        group_by_values,
        aggregation_values,
    );

    Ok(())
}

fn get_summary_value(value: &Value) -> Option<SummaryValue> {
    match value {
        Value::Integer(i) => Some(SummaryValue::Integer(i.get_value())),
        Value::Double(d) => Some(SummaryValue::Double(d.get_value())),
        _ => value.convert_to_double().map(SummaryValue::Double),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_execute_summary_data_expression_group_by() {
        fn run_test(
            summary_data_expression: SummaryDataExpression,
            assert: impl FnOnce(&Vec<RecordSetEngineSummary>),
        ) {
            let record1 = TestRecord::new().with_key_value(
                "key1".into(),
                OwnedValue::String(StringValueStorage::new("value1".into())),
            );

            let record2 = TestRecord::new()
                .with_key_value(
                    "key1".into(),
                    OwnedValue::String(StringValueStorage::new("value2".into())),
                )
                .with_key_value(
                    "key3".into(),
                    OwnedValue::String(StringValueStorage::new("value3".into())),
                );

            let pipeline = PipelineExpressionBuilder::new(" ")
                .with_expressions(vec![DataExpression::Summary(summary_data_expression)])
                .build()
                .unwrap();

            let engine = RecordSetEngine::new();

            let mut batch = engine.begin_batch(&pipeline).unwrap();

            batch.push_records(&mut TestRecordSet::new(vec![
                record1.clone(),
                record1,
                record2,
            ]));

            let results = batch.flush();

            let mut summaries = results.summaries;

            summaries.sort_by(|l, r| l.summary_id.cmp(&r.summary_id));

            println!("{summaries:?}");

            (assert)(&summaries)
        }

        run_test(
            SummaryDataExpression::new(QueryLocation::new_fake(), HashMap::new(), HashMap::new()),
            |s| {
                assert_eq!(1, s.len());

                let summary = &s[0];

                assert_eq!(
                    "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
                    summary.summary_id
                );
                assert_eq!(0, summary.aggregation_values.len());
                assert_eq!(0, summary.group_by_values.len());
            },
        );

        let key1_selector = ScalarExpression::Source(SourceScalarExpression::new(
            QueryLocation::new_fake(),
            ValueAccessor::new_with_selectors(vec![ScalarExpression::Static(
                StaticScalarExpression::String(StringScalarExpression::new(
                    QueryLocation::new_fake(),
                    "key1",
                )),
            )]),
        ));

        let key2_selector = ScalarExpression::Source(SourceScalarExpression::new(
            QueryLocation::new_fake(),
            ValueAccessor::new_with_selectors(vec![ScalarExpression::Static(
                StaticScalarExpression::String(StringScalarExpression::new(
                    QueryLocation::new_fake(),
                    "key2",
                )),
            )]),
        ));

        let key3_selector = ScalarExpression::Source(SourceScalarExpression::new(
            QueryLocation::new_fake(),
            ValueAccessor::new_with_selectors(vec![ScalarExpression::Static(
                StaticScalarExpression::String(StringScalarExpression::new(
                    QueryLocation::new_fake(),
                    "key3",
                )),
            )]),
        ));

        run_test(
            SummaryDataExpression::new(
                QueryLocation::new_fake(),
                HashMap::from([("key1".into(), key1_selector.clone())]),
                HashMap::new(),
            ),
            |s| {
                assert_eq!(2, s.len());

                let summary1 = &s[0];

                assert_eq!(
                    "28dce9b9d827274fb9eaf24a0aa0745ff5f5504335b1044841ea1dc054552e3c",
                    summary1.summary_id
                );
                assert_eq!(0, summary1.aggregation_values.len());
                assert_eq!(1, summary1.group_by_values.len());
                assert_eq!("key1", summary1.group_by_values[0].0.as_ref());
                assert_eq!(
                    "value1",
                    summary1.group_by_values[0].1.to_value().to_string()
                );

                let summary2 = &s[1];

                assert_eq!(
                    "c21e7824f3a91543ce9ec4847830951aa4023c711dc1fa2d3a101811c017f230",
                    summary2.summary_id
                );
                assert_eq!(0, summary2.aggregation_values.len());
                assert_eq!(1, summary2.group_by_values.len());
                assert_eq!("key1", summary2.group_by_values[0].0.as_ref());
                assert_eq!(
                    "value2",
                    summary2.group_by_values[0].1.to_value().to_string()
                );
            },
        );

        run_test(
            SummaryDataExpression::new(
                QueryLocation::new_fake(),
                HashMap::from([
                    ("key1".into(), key1_selector.clone()),
                    ("key2".into(), key2_selector.clone()),
                ]),
                HashMap::new(),
            ),
            |s| {
                assert_eq!(2, s.len());

                let summary1 = &s[0];

                assert_eq!(
                    "552e13f92eb63dcf95868fb35aa3c0195ef3f0425c0edf1087f329c3dd240bec",
                    summary1.summary_id
                );
                assert_eq!(0, summary1.aggregation_values.len());
                assert_eq!(2, summary1.group_by_values.len());
                assert_eq!(
                    "value1",
                    summary1.group_by_values[0].1.to_value().to_string()
                );
                assert_eq!("null", summary1.group_by_values[1].1.to_value().to_string());

                let summary2 = &s[1];

                assert_eq!(
                    "e7cac1b570e106c0e2b401e0d3d93365494a906c71cc8fdefb4272804a538603",
                    summary2.summary_id
                );
                assert_eq!(0, summary2.aggregation_values.len());
                assert_eq!(2, summary2.group_by_values.len());
                assert_eq!(
                    "value2",
                    summary2.group_by_values[0].1.to_value().to_string()
                );
                assert_eq!("null", summary2.group_by_values[1].1.to_value().to_string());
            },
        );

        run_test(
            SummaryDataExpression::new(
                QueryLocation::new_fake(),
                HashMap::from([
                    ("key1".into(), key1_selector.clone()),
                    ("key3".into(), key3_selector.clone()),
                ]),
                HashMap::new(),
            ),
            |s| {
                assert_eq!(2, s.len());

                let summary1 = &s[0];

                assert_eq!(
                    "3409c11163154a373c62f96335ba2acff5adea33f5eb26fe13c3e7755bde9d5a",
                    summary1.summary_id
                );
                assert_eq!(0, summary1.aggregation_values.len());
                assert_eq!(2, summary1.group_by_values.len());
                assert_eq!(
                    "value2",
                    summary1.group_by_values[0].1.to_value().to_string()
                );
                assert_eq!(
                    "value3",
                    summary1.group_by_values[1].1.to_value().to_string()
                );

                let summary2 = &s[1];

                assert_eq!(
                    "7dc0a32fde14f9ea69e3899daaaaacb3df4da1c1a61d0451b7734765eb3fed62",
                    summary2.summary_id
                );
                assert_eq!(0, summary2.aggregation_values.len());
                assert_eq!(2, summary2.group_by_values.len());
                assert_eq!(
                    "value1",
                    summary2.group_by_values[0].1.to_value().to_string()
                );
                assert_eq!("null", summary2.group_by_values[1].1.to_value().to_string());
            },
        );
    }

    #[test]
    fn test_execute_summary_data_expression_aggregation() {
        fn run_test(
            summary_data_expression: SummaryDataExpression,
            assert: impl FnOnce(&Vec<RecordSetEngineSummary>),
        ) {
            let record1 = TestRecord::new().with_key_value(
                "key1".into(),
                OwnedValue::Integer(IntegerValueStorage::new(-18)),
            );

            let record2 = TestRecord::new().with_key_value(
                "key1".into(),
                OwnedValue::Integer(IntegerValueStorage::new(18)),
            );

            let pipeline = PipelineExpressionBuilder::new(" ")
                .with_expressions(vec![DataExpression::Summary(summary_data_expression)])
                .build()
                .unwrap();

            let engine = RecordSetEngine::new();

            let mut batch = engine.begin_batch(&pipeline).unwrap();

            batch.push_records(&mut TestRecordSet::new(vec![record1, record2]));

            let results = batch.flush();

            let mut summaries = results.summaries;

            summaries.sort_by(|l, r| l.summary_id.cmp(&r.summary_id));

            println!("{summaries:?}");

            (assert)(&summaries)
        }

        let key1_selector = ScalarExpression::Source(SourceScalarExpression::new(
            QueryLocation::new_fake(),
            ValueAccessor::new_with_selectors(vec![ScalarExpression::Static(
                StaticScalarExpression::String(StringScalarExpression::new(
                    QueryLocation::new_fake(),
                    "key1",
                )),
            )]),
        ));

        run_test(
            SummaryDataExpression::new(
                QueryLocation::new_fake(),
                HashMap::new(),
                HashMap::from([(
                    "Count".into(),
                    AggregationExpression::new(
                        QueryLocation::new_fake(),
                        AggregationFunction::Count,
                        None,
                    ),
                )]),
            ),
            |s| {
                assert_eq!(1, s.len());

                let summary1 = &s[0];

                assert_eq!(
                    "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
                    summary1.summary_id
                );
                assert_eq!(1, summary1.aggregation_values.len());
                assert!(matches!(
                    summary1.aggregation_values["Count"],
                    SummaryAggregation::Count(2)
                ));

                assert_eq!(0, summary1.group_by_values.len());
            },
        );

        run_test(
            SummaryDataExpression::new(
                QueryLocation::new_fake(),
                HashMap::new(),
                HashMap::from([(
                    "Min".into(),
                    AggregationExpression::new(
                        QueryLocation::new_fake(),
                        AggregationFunction::Minimum,
                        Some(key1_selector.clone()),
                    ),
                )]),
            ),
            |s| {
                assert_eq!(1, s.len());

                let summary1 = &s[0];

                assert_eq!(
                    "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
                    summary1.summary_id
                );
                assert_eq!(1, summary1.aggregation_values.len());
                if let SummaryAggregation::Minimum(OwnedValue::Integer(i)) =
                    &summary1.aggregation_values["Min"]
                {
                    assert_eq!(-18, i.get_value());
                } else {
                    panic!()
                }

                assert_eq!(0, summary1.group_by_values.len());
            },
        );

        run_test(
            SummaryDataExpression::new(
                QueryLocation::new_fake(),
                HashMap::new(),
                HashMap::from([(
                    "Max".into(),
                    AggregationExpression::new(
                        QueryLocation::new_fake(),
                        AggregationFunction::Maximum,
                        Some(key1_selector.clone()),
                    ),
                )]),
            ),
            |s| {
                assert_eq!(1, s.len());

                let summary1 = &s[0];

                assert_eq!(
                    "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
                    summary1.summary_id
                );
                assert_eq!(1, summary1.aggregation_values.len());
                if let SummaryAggregation::Maximum(OwnedValue::Integer(i)) =
                    &summary1.aggregation_values["Max"]
                {
                    assert_eq!(18, i.get_value());
                } else {
                    panic!()
                }

                assert_eq!(0, summary1.group_by_values.len());
            },
        );

        run_test(
            SummaryDataExpression::new(
                QueryLocation::new_fake(),
                HashMap::new(),
                HashMap::from([(
                    "Sum".into(),
                    AggregationExpression::new(
                        QueryLocation::new_fake(),
                        AggregationFunction::Sum,
                        Some(key1_selector.clone()),
                    ),
                )]),
            ),
            |s| {
                assert_eq!(1, s.len());

                let summary1 = &s[0];

                assert_eq!(
                    "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
                    summary1.summary_id
                );
                assert_eq!(1, summary1.aggregation_values.len());
                if let SummaryAggregation::Sum(SummaryValue::Integer(i)) =
                    &summary1.aggregation_values["Sum"]
                {
                    assert_eq!(0, *i);
                } else {
                    panic!()
                }

                assert_eq!(0, summary1.group_by_values.len());
            },
        );

        run_test(
            SummaryDataExpression::new(
                QueryLocation::new_fake(),
                HashMap::new(),
                HashMap::from([(
                    "Avg".into(),
                    AggregationExpression::new(
                        QueryLocation::new_fake(),
                        AggregationFunction::Average,
                        Some(key1_selector.clone()),
                    ),
                )]),
            ),
            |s| {
                assert_eq!(1, s.len());

                let summary1 = &s[0];

                assert_eq!(
                    "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
                    summary1.summary_id
                );
                assert_eq!(1, summary1.aggregation_values.len());
                if let SummaryAggregation::Average {
                    count,
                    sum: SummaryValue::Integer(i),
                } = &summary1.aggregation_values["Avg"]
                {
                    assert_eq!(2, *count);
                    assert_eq!(0, *i);
                } else {
                    panic!()
                }

                assert_eq!(0, summary1.group_by_values.len());
            },
        );

        // Test multiple aggregations in play
        run_test(
            SummaryDataExpression::new(
                QueryLocation::new_fake(),
                HashMap::new(),
                HashMap::from([
                    (
                        "Count".into(),
                        AggregationExpression::new(
                            QueryLocation::new_fake(),
                            AggregationFunction::Count,
                            None,
                        ),
                    ),
                    (
                        "Sum".into(),
                        AggregationExpression::new(
                            QueryLocation::new_fake(),
                            AggregationFunction::Sum,
                            Some(key1_selector.clone()),
                        ),
                    ),
                ]),
            ),
            |s| {
                assert_eq!(1, s.len());

                let summary1 = &s[0];

                assert_eq!(
                    "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
                    summary1.summary_id
                );
                assert_eq!(2, summary1.aggregation_values.len());

                if let SummaryAggregation::Count(i) = &summary1.aggregation_values["Count"] {
                    assert_eq!(2, *i);
                } else {
                    panic!()
                }

                if let SummaryAggregation::Sum(SummaryValue::Integer(i)) =
                    &summary1.aggregation_values["Sum"]
                {
                    assert_eq!(0, *i);
                } else {
                    panic!()
                }

                assert_eq!(0, summary1.group_by_values.len());
            },
        );
    }
}
