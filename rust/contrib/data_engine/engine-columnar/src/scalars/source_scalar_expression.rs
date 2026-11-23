// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use otel_arrow_contrib_data_engine_expressions::*;

use crate::{
    execution_context::ExecutionContext, resolved_value::*, selection::select_from_record_table, *,
};

pub fn execute_source_scalar_expression<'a, 'pipeline, TRecords: ColumnarRecords<'pipeline>>(
    execution_context: &'a ExecutionContext<'a, 'pipeline, TRecords>,
    source_scalar_expression: &'pipeline SourceScalarExpression,
) -> ResolvedScalarValue<'pipeline, 'a> {
    let record = match execution_context.get_records() {
        Some(r) => r,
        None => {
            execution_context.add_diagnostic_if_enabled(
                ColumnarEngineDiagnosticLevel::Warn,
                source_scalar_expression,
                || "Source could not be found".into(),
            );
            return ResolvedScalarValue::new_null();
        }
    };

    select_from_record_table(
        execution_context,
        record,
        source_scalar_expression
            .get_value_accessor()
            .get_selectors(),
    )
}

#[cfg(test)]
mod tests {
    use std::{cell::RefCell, collections::HashMap};

    use arrow::{array::UInt8Array, datatypes::DataType};

    use crate::test_helpers::*;

    use super::*;

    fn run_scalar_selection_with_implicit_keys(
        selector: StaticScalarExpression,
        wrap_value: impl Fn(usize, ValueOrRef<'static>) -> ValueOrRef<'static>,
    ) {
        for (key_type, capacity, wider_type) in [
            (DictionaryKeyType::Int8, 128usize, DictionaryKeyType::Int16),
            (DictionaryKeyType::UInt8, 256, DictionaryKeyType::UInt16),
        ] {
            for (row_count, remap) in [
                (capacity, false),
                (capacity + 1, false),
                (capacity, true),
                (capacity + 1, true),
                (capacity + 4, true),
            ] {
                let distinct_count = if remap { row_count - 3 } else { row_count };
                let mut expected = (0..distinct_count)
                    .map(|value| ValueOrRef::Integer(i64::try_from(value).expect("value fits")))
                    .collect::<Vec<_>>();
                if remap {
                    expected.insert(0, ValueOrRef::Integer(0));
                    expected.insert(1, ValueOrRef::Null);
                    expected.push(ValueOrRef::Integer(1));
                }
                let values = expected
                    .iter()
                    .cloned()
                    .enumerate()
                    .map(|(row, value)| wrap_value(row, value))
                    .collect::<Vec<_>>();
                let records = TestRecords::new().with_values(HashMap::from([(
                    "values".into(),
                    Dictionary::new_unique_values(key_type.clone(), values.into()),
                )]));
                let expression = SourceScalarExpression::new(
                    QueryLocation::new_fake(),
                    ValueAccessor::new_with_selectors(vec![
                        ScalarExpression::Static(StaticScalarExpression::String(
                            StringScalarExpression::new(QueryLocation::new_fake(), "values"),
                        )),
                        ScalarExpression::Static(selector.clone()),
                    ]),
                );
                let diagnostics = RefCell::new(vec![]);
                let pipeline = PipelineExpression::default();
                let context = ExecutionContext::new(
                    ColumnarEngineDiagnosticLevel::Warn,
                    &diagnostics,
                    &pipeline,
                    Some(records),
                );

                let result = execute_source_scalar_expression(&context, &expression);

                let ResolvedScalarValue::Dictionary(actual) = result else {
                    panic!("expected dictionary, got {result:?}");
                };
                assert_eq!(row_count, actual.len());
                for (row, value) in expected.iter().enumerate() {
                    assert_eq!(
                        Ok(value),
                        actual.get_value(row).as_ref(),
                        "{key_type:?} selector at row {row}, remap={remap}",
                    );
                }
                let expected_type = if row_count > capacity {
                    wider_type.clone()
                } else {
                    key_type.clone()
                };
                assert_eq!(expected_type, actual.keys().data_type());
                if !remap {
                    assert_eq!(
                        &DictionaryKeyArray::new_unique_values(expected_type, row_count),
                        actual.keys(),
                    );
                }
                assert!(
                    diagnostics.borrow().is_empty(),
                    "{:?}",
                    diagnostics.borrow(),
                );
            }
        }
    }

    /// Scenario: A scalar key projects distinct implicit maps to duplicate, null, or distinct fields.
    /// Guarantees: Selection preserves construction-time key widths, values, and nulls; injective results keep implicit keys.
    #[test]
    fn test_select_scalar_map_key_remaps_implicit_keys_losslessly() {
        run_scalar_selection_with_implicit_keys(
            StaticScalarExpression::String(StringScalarExpression::new(
                QueryLocation::new_fake(),
                "selected",
            )),
            |row, value| {
                ValueOrRef::Map(MapValueOrRef::from([
                    ("selected".into(), value),
                    (
                        "row".into(),
                        ValueOrRef::Integer(i64::try_from(row).expect("row fits")),
                    ),
                ]))
            },
        );
    }

    /// Scenario: A scalar index projects distinct implicit arrays to duplicate, null, or distinct elements.
    /// Guarantees: Selection preserves valid input key widths without data loss or warnings, while unchanged indexes stay implicit.
    #[test]
    fn test_select_scalar_array_index_remaps_implicit_keys_losslessly() {
        run_scalar_selection_with_implicit_keys(
            StaticScalarExpression::Integer(IntegerScalarExpression::new(
                QueryLocation::new_fake(),
                0,
            )),
            |row, value| {
                ValueOrRef::Array(ArrayValueOrRef::from([
                    value,
                    ValueOrRef::Integer(i64::try_from(row).expect("row fits")),
                ]))
            },
        );
    }

    /// Scenario: Source access is evaluated without any source records in the execution context.
    /// Guarantees: The result is null and one warning identifies the unavailable source expression.
    #[test]
    fn test_select_from_missing_source() {
        let diagnostics = RefCell::new(vec![]);
        let pipeline = PipelineExpression::default();
        let expression =
            SourceScalarExpression::new(QueryLocation::new_fake(), ValueAccessor::new());
        let context = ExecutionContext::<TestRecords>::new(
            ColumnarEngineDiagnosticLevel::Warn,
            &diagnostics,
            &pipeline,
            None,
        );

        let result = execute_source_scalar_expression(&context, &expression);

        assert!(matches!(
            result,
            ResolvedScalarValue::Single(ValueOrRef::Null)
        ));
        let diagnostics = diagnostics.borrow();
        assert_eq!(1, diagnostics.len());
        assert_eq!(
            ColumnarEngineDiagnosticLevel::Warn,
            diagnostics[0].get_diagnostic_level(),
        );
        assert_eq!("Source could not be found", diagnostics[0].get_message());
        assert!(std::ptr::addr_eq(
            diagnostics[0].get_expression(),
            &expression as &dyn Expression,
        ));
    }

    /// Scenario: Row-dependent source column names use every supported dictionary key type.
    /// Guarantees: Selection reads each column at the current row and preserves sufficient key types and nulls.
    #[test]
    fn test_select_from_source_table_using_dynamic_column_names() {
        for key_type in [
            DictionaryKeyType::Int8,
            DictionaryKeyType::Int16,
            DictionaryKeyType::Int32,
            DictionaryKeyType::Int64,
            DictionaryKeyType::UInt8,
            DictionaryKeyType::UInt16,
            DictionaryKeyType::UInt32,
            DictionaryKeyType::UInt64,
        ] {
            let columns = Dictionary::new_unique_values(
                key_type.clone(),
                vec![
                    ValueOrRef::String(StringValueOrRef::new_owned("right".into())),
                    ValueOrRef::String(StringValueOrRef::new_owned("left".into())),
                    ValueOrRef::String(StringValueOrRef::new_owned("right".into())),
                    ValueOrRef::Null,
                    ValueOrRef::String(StringValueOrRef::new_owned("missing".into())),
                    ValueOrRef::String(StringValueOrRef::new_owned("left".into())),
                    ValueOrRef::Integer(0),
                    ValueOrRef::Boolean(true),
                    ValueOrRef::String(StringValueOrRef::new_owned("right".into())),
                ]
                .into(),
            );
            let expression = ScalarExpression::Source(SourceScalarExpression::new(
                QueryLocation::new_fake(),
                ValueAccessor::new_with_selectors(vec![ScalarExpression::Source(
                    SourceScalarExpression::new(
                        QueryLocation::new_fake(),
                        ValueAccessor::new_with_selectors(vec![ScalarExpression::Static(
                            StaticScalarExpression::String(StringScalarExpression::new(
                                QueryLocation::new_fake(),
                                "column",
                            )),
                        )]),
                    ),
                )]),
            ));

            run_scalar_expression_test(
                TestRecords::new().with_values(HashMap::from([
                    ("column".into(), columns),
                    (
                        "left".into(),
                        build_dictionary(
                            vec![
                                Some(0),
                                Some(1),
                                Some(0),
                                Some(1),
                                Some(0),
                                None,
                                Some(1),
                                Some(0),
                                Some(1),
                            ],
                            vec![ValueOrRef::Integer(10), ValueOrRef::Integer(20)],
                        ),
                    ),
                    (
                        "right".into(),
                        build_dictionary(
                            vec![
                                Some(0),
                                Some(0),
                                Some(1),
                                Some(1),
                                Some(0),
                                Some(1),
                                Some(0),
                                Some(0),
                                Some(0),
                            ],
                            vec![ValueOrRef::Integer(300), ValueOrRef::Integer(400)],
                        ),
                    ),
                ])),
                expression,
                |result| match result {
                    ResolvedScalarValue::Dictionary(actual) => {
                        assert_eq!(key_type, actual.keys().data_type());
                        let rows: Vec<_> = (0..actual.len())
                            .map(|row| actual.get_value(row).expect("valid selected value"))
                            .collect();
                        assert_eq!(
                            vec![
                                ValueOrRef::Integer(300),
                                ValueOrRef::Integer(20),
                                ValueOrRef::Integer(400),
                                ValueOrRef::Null,
                                ValueOrRef::Null,
                                ValueOrRef::Null,
                                ValueOrRef::Null,
                                ValueOrRef::Null,
                                ValueOrRef::Integer(300),
                            ],
                            rows,
                        );
                    }
                    other => panic!("expected dictionary, got {other:?}"),
                },
            );
        }
    }

    /// Scenario: Repeated narrow dictionary column names select distinct values at key-width boundaries.
    /// Guarantees: Selection widens only when needed and preserves every value and original null row.
    #[test]
    fn test_select_from_source_table_widens_result_keys() {
        for (selector_type, row_count, expected_type, with_nulls) in [
            (DictionaryKeyType::UInt8, 0, DictionaryKeyType::UInt8, false),
            (
                DictionaryKeyType::UInt8,
                256,
                DictionaryKeyType::UInt8,
                false,
            ),
            (
                DictionaryKeyType::UInt8,
                257,
                DictionaryKeyType::UInt16,
                false,
            ),
            (
                DictionaryKeyType::UInt8,
                300,
                DictionaryKeyType::UInt16,
                false,
            ),
            (DictionaryKeyType::Int8, 128, DictionaryKeyType::Int8, false),
            (
                DictionaryKeyType::Int8,
                129,
                DictionaryKeyType::Int16,
                false,
            ),
            (
                DictionaryKeyType::UInt8,
                300,
                DictionaryKeyType::UInt16,
                true,
            ),
        ] {
            let column = build_dictionary(
                (0..row_count)
                    .map(|row| {
                        if with_nulls && row % 7 == 0 {
                            None
                        } else {
                            Some(u16::try_from(row).expect("row index fits source key type"))
                        }
                    })
                    .collect(),
                (0..row_count)
                    .map(|row| ValueOrRef::Integer(i64::try_from(row).expect("row fits integer")))
                    .collect(),
            );
            let selector = Dictionary::new_scalar(
                selector_type.clone(),
                row_count,
                ValueOrRef::String(StringValueOrRef::new_owned("right".into())),
            );
            let expression = SourceScalarExpression::new(
                QueryLocation::new_fake(),
                ValueAccessor::new_with_selectors(vec![ScalarExpression::Source(
                    SourceScalarExpression::new(
                        QueryLocation::new_fake(),
                        ValueAccessor::new_with_selectors(vec![ScalarExpression::Static(
                            StaticScalarExpression::String(StringScalarExpression::new(
                                QueryLocation::new_fake(),
                                "column",
                            )),
                        )]),
                    ),
                )]),
            );
            let records = TestRecords::new().with_values(HashMap::from([
                ("column".into(), selector),
                ("right".into(), column.clone()),
            ]));
            let diagnostics = RefCell::new(vec![]);
            let pipeline = PipelineExpression::default();
            let context = ExecutionContext::new(
                ColumnarEngineDiagnosticLevel::Warn,
                &diagnostics,
                &pipeline,
                Some(records),
            );

            let result = execute_source_scalar_expression(&context, &expression);

            let ResolvedScalarValue::Dictionary(actual) = result else {
                panic!("expected dictionary, got {result:?}");
            };
            assert_eq!(row_count, actual.len());
            for row in 0..row_count {
                assert_eq!(
                    column.get_value(row).expect("valid source value"),
                    actual.get_value(row).expect("valid selected value"),
                    "{selector_type:?} selector at row {row}",
                );
            }
            assert_eq!(expected_type, actual.keys().data_type());
            assert!(
                diagnostics.borrow().is_empty(),
                "{:?}",
                diagnostics.borrow()
            );
        }
    }

    /// Scenario: Narrow row-dependent array indexes select many distinct values with null inputs.
    /// Guarantees: Result keys widen without losing values or changing source and selector null positions.
    #[test]
    fn test_select_array_elements_widens_result_keys() {
        let row_count = 350;
        let values = build_dictionary(
            (0..row_count)
                .map(|row| {
                    if row % 23 == 0 {
                        None
                    } else {
                        Some(u16::try_from(row).expect("row index fits source key type"))
                    }
                })
                .collect(),
            (0..row_count)
                .map(|row| {
                    ValueOrRef::Array(ArrayValueOrRef::from([ValueOrRef::Integer(
                        i64::try_from(row).expect("row fits integer"),
                    )]))
                })
                .collect(),
        );
        let indexes = Dictionary::new(
            UInt8Array::from(
                (0..row_count)
                    .map(|row| (row % 19 != 0).then_some(0))
                    .collect::<Vec<_>>(),
            )
            .into(),
            vec![ValueOrRef::Integer(0)].into(),
        )
        .expect("valid index dictionary");
        let expression = ScalarExpression::Source(SourceScalarExpression::new(
            QueryLocation::new_fake(),
            ValueAccessor::new_with_selectors(vec![
                ScalarExpression::Static(StaticScalarExpression::String(
                    StringScalarExpression::new(QueryLocation::new_fake(), "values"),
                )),
                ScalarExpression::Source(SourceScalarExpression::new(
                    QueryLocation::new_fake(),
                    ValueAccessor::new_with_selectors(vec![ScalarExpression::Static(
                        StaticScalarExpression::String(StringScalarExpression::new(
                            QueryLocation::new_fake(),
                            "index",
                        )),
                    )]),
                )),
            ]),
        ));

        run_scalar_expression_test(
            TestRecords::new().with_values(HashMap::from([
                ("values".into(), values),
                ("index".into(), indexes),
            ])),
            expression,
            |result| {
                let ResolvedScalarValue::Dictionary(actual) = result else {
                    panic!("expected dictionary, got {result:?}");
                };
                assert_eq!(row_count, actual.len());
                for row in 0..row_count {
                    let expected = if row % 19 == 0 || row % 23 == 0 {
                        ValueOrRef::Null
                    } else {
                        ValueOrRef::Integer(i64::try_from(row).expect("row fits integer"))
                    };
                    assert_eq!(
                        expected,
                        actual.get_value(row).expect("valid selected value"),
                        "array selection at row {row}",
                    );
                }
                assert_eq!(DictionaryKeyType::UInt16, actual.keys().data_type());
            },
        );
    }

    /// Scenario: String accessors select existing/missing source keys, nested maps, and the root.
    /// Guarantees: Selected rows stay aligned, missing keys yield nulls, and root access returns a table.
    #[test]
    fn test_select_from_source_table_using_single_string() {
        let values_dictionary = build_dictionary(
            vec![Some(0), Some(0), None, Some(1), Some(2), Some(3)],
            vec![
                ValueOrRef::String(StringValueOrRef::new_owned("hello world".into())),
                ValueOrRef::String(StringValueOrRef::new_owned("goodbye world".into())),
                ValueOrRef::Map(MapValueOrRef::from([(
                    "key1".into(),
                    ValueOrRef::Integer(18),
                )])),
                ValueOrRef::Integer(0),
            ],
        );

        let select_valid_key = SourceScalarExpression::new(
            QueryLocation::new_fake(),
            ValueAccessor::new_with_selectors(vec![ScalarExpression::Static(
                StaticScalarExpression::String(StringScalarExpression::new(
                    QueryLocation::new_fake(),
                    "values",
                )),
            )]),
        );

        run_scalar_expression_test(
            TestRecords::new().with_values(HashMap::from([(
                "values".into(),
                values_dictionary.clone(),
            )])),
            ScalarExpression::Source(select_valid_key),
            |r| match r {
                ResolvedScalarValue::Dictionary(actual) => {
                    assert_eq!(values_dictionary, actual)
                }
                _ => panic!("test failure"),
            },
        );

        let select_invalid_key = SourceScalarExpression::new(
            QueryLocation::new_fake(),
            ValueAccessor::new_with_selectors(vec![ScalarExpression::Static(
                StaticScalarExpression::String(StringScalarExpression::new(
                    QueryLocation::new_fake(),
                    "unknown",
                )),
            )]),
        );

        run_scalar_expression_test(
            TestRecords::new(),
            ScalarExpression::Source(select_invalid_key),
            |r| {
                assert!(matches!(r, ResolvedScalarValue::Single(ValueOrRef::Null)));
            },
        );

        let select_root =
            SourceScalarExpression::new(QueryLocation::new_fake(), ValueAccessor::new());

        run_scalar_expression_test(
            TestRecords::new(),
            ScalarExpression::Source(select_root),
            |r| {
                assert!(matches!(r, ResolvedScalarValue::Table(_)));
            },
        );

        let select_sub_key = SourceScalarExpression::new(
            QueryLocation::new_fake(),
            ValueAccessor::new_with_selectors(vec![
                ScalarExpression::Static(StaticScalarExpression::String(
                    StringScalarExpression::new(QueryLocation::new_fake(), "values"),
                )),
                ScalarExpression::Static(StaticScalarExpression::String(
                    StringScalarExpression::new(QueryLocation::new_fake(), "key1"),
                )),
            ]),
        );

        run_scalar_expression_test(
            TestRecords::new().with_values(HashMap::from([(
                "values".into(),
                values_dictionary.clone(),
            )])),
            ScalarExpression::Source(select_sub_key),
            |r| match r {
                ResolvedScalarValue::Dictionary(actual) => {
                    assert_eq!(
                        build_dictionary(
                            vec![None, None, None, None, Some(0), None],
                            vec![ValueOrRef::Integer(18)]
                        ),
                        actual
                    );
                }
                _ => panic!("test failure"),
            },
        );

        let select_sub_key_invalid = SourceScalarExpression::new(
            QueryLocation::new_fake(),
            ValueAccessor::new_with_selectors(vec![
                ScalarExpression::Static(StaticScalarExpression::String(
                    StringScalarExpression::new(QueryLocation::new_fake(), "values"),
                )),
                ScalarExpression::Static(StaticScalarExpression::String(
                    StringScalarExpression::new(QueryLocation::new_fake(), "invalid"),
                )),
            ]),
        );

        run_scalar_expression_test(
            TestRecords::new().with_values(HashMap::from([(
                "values".into(),
                values_dictionary.clone(),
            )])),
            ScalarExpression::Source(select_sub_key_invalid),
            |r| match r {
                ResolvedScalarValue::Dictionary(actual) => {
                    assert_eq!(Dictionary::new_null(DictionaryKeyType::UInt16, 6), actual);
                }
                _ => panic!("test failure"),
            },
        );
    }

    /// Scenario: Scalar indexes select array elements from mixed-type source rows.
    /// Guarantees: Negative indexes select from the end; invalid indexes and non-arrays yield null.
    #[test]
    fn test_select_from_source_table_using_single_integer() {
        let values_dictionary = build_dictionary(
            vec![Some(0), Some(0), None, Some(1)],
            vec![
                ValueOrRef::Array(ArrayValueOrRef::from([
                    ValueOrRef::Integer(0),
                    ValueOrRef::Integer(1),
                    ValueOrRef::Integer(2),
                ])),
                ValueOrRef::Integer(0),
            ],
        );

        let select_sub_index = SourceScalarExpression::new(
            QueryLocation::new_fake(),
            ValueAccessor::new_with_selectors(vec![
                ScalarExpression::Static(StaticScalarExpression::String(
                    StringScalarExpression::new(QueryLocation::new_fake(), "values"),
                )),
                ScalarExpression::Static(StaticScalarExpression::Integer(
                    IntegerScalarExpression::new(QueryLocation::new_fake(), 0),
                )),
            ]),
        );

        run_scalar_expression_test(
            TestRecords::new().with_values(HashMap::from([(
                "values".into(),
                values_dictionary.clone(),
            )])),
            ScalarExpression::Source(select_sub_index),
            |r| match r {
                ResolvedScalarValue::Dictionary(actual) => {
                    assert_eq!(
                        build_dictionary(
                            vec![Some(0), Some(0), None, None],
                            vec![ValueOrRef::Integer(0)]
                        ),
                        actual
                    );
                }
                _ => panic!("test failure"),
            },
        );

        let select_sub_index_negative = SourceScalarExpression::new(
            QueryLocation::new_fake(),
            ValueAccessor::new_with_selectors(vec![
                ScalarExpression::Static(StaticScalarExpression::String(
                    StringScalarExpression::new(QueryLocation::new_fake(), "values"),
                )),
                ScalarExpression::Static(StaticScalarExpression::Integer(
                    IntegerScalarExpression::new(QueryLocation::new_fake(), -1),
                )),
            ]),
        );

        run_scalar_expression_test(
            TestRecords::new().with_values(HashMap::from([(
                "values".into(),
                values_dictionary.clone(),
            )])),
            ScalarExpression::Source(select_sub_index_negative),
            |r| match r {
                ResolvedScalarValue::Dictionary(actual) => {
                    assert_eq!(
                        build_dictionary(
                            vec![Some(0), Some(0), None, None],
                            vec![ValueOrRef::Integer(2)]
                        ),
                        actual
                    );
                }
                _ => panic!("test failure"),
            },
        );

        let select_sub_index_invalid = SourceScalarExpression::new(
            QueryLocation::new_fake(),
            ValueAccessor::new_with_selectors(vec![
                ScalarExpression::Static(StaticScalarExpression::String(
                    StringScalarExpression::new(QueryLocation::new_fake(), "values"),
                )),
                ScalarExpression::Static(StaticScalarExpression::Integer(
                    IntegerScalarExpression::new(QueryLocation::new_fake(), 100),
                )),
            ]),
        );

        run_scalar_expression_test(
            TestRecords::new().with_values(HashMap::from([(
                "values".into(),
                values_dictionary.clone(),
            )])),
            ScalarExpression::Source(select_sub_index_invalid),
            |r| match r {
                ResolvedScalarValue::Dictionary(actual) => {
                    assert_eq!(Dictionary::new_null(DictionaryKeyType::UInt16, 4), actual);
                }
                _ => panic!("test failure"),
            },
        );
    }

    /// Scenario: Source dictionaries provide per-row map keys and array indexes for mixed values.
    /// Guarantees: Each selector is applied to its matching row while null rows remain aligned.
    #[test]
    fn test_select_from_source_table_using_dictionary() {
        let values_dictionary = build_dictionary(
            vec![
                Some(0), // string value: hello world
                Some(0), // string value: hello world
                None,
                Some(1), // string value: goodbye world
                Some(2), // map value
                Some(2), // map value
                Some(3), // array value
                Some(3), // array value
                Some(4), // integer value
            ],
            vec![
                ValueOrRef::String(StringValueOrRef::new_owned("hello world".into())),
                ValueOrRef::String(StringValueOrRef::new_owned("goodebye world".into())),
                ValueOrRef::Map(MapValueOrRef::from([
                    (
                        "key1".into(),
                        ValueOrRef::String(StringValueOrRef::new_owned("value1".into())),
                    ),
                    (
                        "key2".into(),
                        ValueOrRef::String(StringValueOrRef::new_owned("value2".into())),
                    ),
                ])),
                ValueOrRef::Array(ArrayValueOrRef::from([
                    ValueOrRef::Integer(0),
                    ValueOrRef::Integer(1),
                    ValueOrRef::Integer(2),
                ])),
                ValueOrRef::Integer(0),
            ],
        );

        let keys_dictionary = build_dictionary(
            vec![
                None,
                None,
                None,
                None,
                Some(0), // Should eval as map['key1']
                Some(1), // Should eval as map['key2']
                None,
                None,
                None,
            ],
            vec![
                ValueOrRef::String(StringValueOrRef::new_owned("key1".into())),
                ValueOrRef::String(StringValueOrRef::new_owned("key2".into())),
            ],
        );

        let indicies_dictionary = build_dictionary(
            vec![
                None,
                None,
                None,
                None,
                None,
                None,
                Some(0), // Should eval as array[0]
                Some(1), // Should eval as array[-1]
                None,
            ],
            vec![ValueOrRef::Integer(0), ValueOrRef::Integer(-1)],
        );

        let select_sub_key = SourceScalarExpression::new(
            QueryLocation::new_fake(),
            ValueAccessor::new_with_selectors(vec![
                ScalarExpression::Static(StaticScalarExpression::String(
                    StringScalarExpression::new(QueryLocation::new_fake(), "values"),
                )),
                ScalarExpression::Source(SourceScalarExpression::new(
                    QueryLocation::new_fake(),
                    ValueAccessor::new_with_selectors(vec![ScalarExpression::Static(
                        StaticScalarExpression::String(StringScalarExpression::new(
                            QueryLocation::new_fake(),
                            "keys",
                        )),
                    )]),
                )),
            ]),
        );

        run_scalar_expression_test(
            TestRecords::new().with_values(HashMap::from([
                ("values".into(), values_dictionary.clone()),
                ("keys".into(), keys_dictionary.clone()),
            ])),
            ScalarExpression::Source(select_sub_key),
            |r| match r {
                ResolvedScalarValue::Dictionary(actual) => {
                    assert_eq!(
                        build_dictionary(
                            vec![
                                None,
                                None,
                                None,
                                None,
                                Some(0), // Should eval as map['key1']
                                Some(1), // Should eval as map['key2']
                                None,
                                None,
                                None,
                            ],
                            vec![
                                ValueOrRef::String(StringValueOrRef::new_owned("value1".into())),
                                ValueOrRef::String(StringValueOrRef::new_owned("value2".into())),
                            ],
                        ),
                        actual
                    );
                }
                _ => panic!("test failure"),
            },
        );

        let select_sub_element = SourceScalarExpression::new(
            QueryLocation::new_fake(),
            ValueAccessor::new_with_selectors(vec![
                ScalarExpression::Static(StaticScalarExpression::String(
                    StringScalarExpression::new(QueryLocation::new_fake(), "values"),
                )),
                ScalarExpression::Source(SourceScalarExpression::new(
                    QueryLocation::new_fake(),
                    ValueAccessor::new_with_selectors(vec![ScalarExpression::Static(
                        StaticScalarExpression::String(StringScalarExpression::new(
                            QueryLocation::new_fake(),
                            "indicies",
                        )),
                    )]),
                )),
            ]),
        );

        run_scalar_expression_test(
            TestRecords::new().with_values(HashMap::from([
                ("values".into(), values_dictionary.clone()),
                ("indicies".into(), indicies_dictionary.clone()),
            ])),
            ScalarExpression::Source(select_sub_element),
            |r| match r {
                ResolvedScalarValue::Dictionary(actual) => {
                    assert_eq!(
                        build_dictionary(
                            vec![
                                None,
                                None,
                                None,
                                None,
                                None,
                                None,
                                Some(0), // Should eval as array[0]
                                Some(1), // Should eval as array[-1]
                                None,
                            ],
                            vec![ValueOrRef::Integer(0), ValueOrRef::Integer(2),],
                        ),
                        actual
                    );
                }
                _ => panic!("test failure"),
            },
        );
    }
}
