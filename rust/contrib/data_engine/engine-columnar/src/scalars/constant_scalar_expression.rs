// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use otel_arrow_contrib_data_engine_expressions::*;

use crate::{execution_context::ExecutionContext, resolved_value::*, selection::select, *};

pub fn execute_constant_scalar_expression<'a, 'pipeline, TRecords: ColumnarRecords<'pipeline>>(
    execution_context: &'a ExecutionContext<'a, 'pipeline, TRecords>,
    constant_scalar_expression: &'pipeline ReferenceConstantScalarExpression,
) -> ResolvedScalarValue<'pipeline, 'a> {
    let constant_id = constant_scalar_expression.get_constant_id();

    let constant = execution_context
        .get_pipeline()
        .get_constant(constant_id)
        .unwrap_or_else(|| panic!("Constant for id '{constant_id}' was not found on pipeline"));

    if execution_context.is_diagnostic_level_enabled(ColumnarEngineDiagnosticLevel::Verbose) {
        let (line, column) = constant.get_query_location().get_line_and_column_numbers();
        execution_context.add_diagnostic(ColumnarEngineDiagnostic::new(
            ColumnarEngineDiagnosticLevel::Verbose,
            constant_scalar_expression,
            format!("Resolved '{}' constant with id '{constant_id}' defined on line {line} at column {column}", constant.get_value_type()),
        ));
    }

    select(
        execution_context,
        ResolvedScalarValue::Single(constant.to_value().into()),
        constant_scalar_expression
            .get_value_accessor()
            .get_selectors(),
    )
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use crate::test_helpers::*;

    use super::*;

    /// Scenario: Two records index the same constant array using a source column.
    /// Guarantees: Selection broadcasts the constant and returns one value per record.
    #[test]
    fn test_select_constant_array_with_source_index() {
        let pipeline = PipelineExpressionBuilder::new("")
            .with_constants(vec![StaticScalarExpression::Array(
                ArrayScalarExpression::new(
                    QueryLocation::new_fake(),
                    vec![StaticScalarExpression::Integer(
                        IntegerScalarExpression::new(QueryLocation::new_fake(), 99),
                    )],
                ),
            )])
            .build()
            .expect("valid pipeline");

        let index = ScalarExpression::Source(SourceScalarExpression::new(
            QueryLocation::new_fake(),
            ValueAccessor::new_with_selectors(vec![ScalarExpression::Static(
                StaticScalarExpression::String(StringScalarExpression::new(
                    QueryLocation::new_fake(),
                    "index",
                )),
            )]),
        ));

        let expression = ScalarExpression::Constant(ReferenceConstantScalarExpression::new(
            QueryLocation::new_fake(),
            ValueType::Integer,
            0,
            ValueAccessor::new_with_selectors(vec![index]),
        ));

        run_scalar_expression_test_with_state(
            ExecutionContextState::new()
                .with_pipeline(&pipeline)
                .with_records(TestRecords::new().with_values(HashMap::from([(
                    "index".into(),
                    build_dictionary(vec![Some(0), Some(0)], vec![ValueOrRef::Integer(0)]),
                )]))),
            expression,
            |result| match result {
                ResolvedScalarValue::Dictionary(actual) => assert_eq!(
                    build_dictionary(vec![Some(0), Some(0)], vec![ValueOrRef::Integer(99)]),
                    actual,
                ),
                other => panic!("expected dictionary, got {other:?}"),
            },
        );
    }

    /// Scenario: Source rows select present, missing, null, and invalid keys from a constant map.
    /// Guarantees: Scalar maps broadcast per row, preserve row order, and return null for invalid keys.
    #[test]
    fn test_select_constant_map_with_source_key() {
        let pipeline = PipelineExpressionBuilder::new("")
            .with_constants(vec![StaticScalarExpression::Map(MapScalarExpression::new(
                QueryLocation::new_fake(),
                HashMap::from([
                    (
                        "first".into(),
                        StaticScalarExpression::Integer(IntegerScalarExpression::new(
                            QueryLocation::new_fake(),
                            10,
                        )),
                    ),
                    (
                        "second".into(),
                        StaticScalarExpression::Integer(IntegerScalarExpression::new(
                            QueryLocation::new_fake(),
                            20,
                        )),
                    ),
                ]),
            ))])
            .build()
            .expect("valid pipeline");

        let expression = ScalarExpression::Constant(ReferenceConstantScalarExpression::new(
            QueryLocation::new_fake(),
            ValueType::Integer,
            0,
            ValueAccessor::new_with_selectors(vec![ScalarExpression::Source(
                SourceScalarExpression::new(
                    QueryLocation::new_fake(),
                    ValueAccessor::new_with_selectors(vec![ScalarExpression::Static(
                        StaticScalarExpression::String(StringScalarExpression::new(
                            QueryLocation::new_fake(),
                            "key",
                        )),
                    )]),
                ),
            )]),
        ));

        run_scalar_expression_test_with_state(
            ExecutionContextState::new()
                .with_pipeline(&pipeline)
                .with_records(TestRecords::new().with_values(HashMap::from([(
                    "key".into(),
                    build_dictionary(
                        vec![
                            Some(0),
                            Some(1),
                            Some(0),
                            None,
                            Some(2),
                            Some(3),
                            Some(4),
                            Some(5),
                        ],
                        vec![
                            ValueOrRef::String(StringValueOrRef::new_owned("second".into())),
                            ValueOrRef::String(StringValueOrRef::new_owned("first".into())),
                            ValueOrRef::String(StringValueOrRef::new_owned("missing".into())),
                            ValueOrRef::Integer(0),
                            ValueOrRef::Boolean(true),
                            ValueOrRef::Null,
                        ],
                    ),
                )]))),
            expression,
            |result| match result {
                ResolvedScalarValue::Dictionary(actual) => assert_eq!(
                    build_dictionary(
                        vec![Some(0), Some(1), Some(0), None, None, None, None, None],
                        vec![ValueOrRef::Integer(20), ValueOrRef::Integer(10)],
                    ),
                    actual,
                ),
                other => panic!("expected dictionary, got {other:?}"),
            },
        );
    }

    /// Scenario: A nested constant array receives mixed, all-null, or empty row-dependent indexes.
    /// Guarantees: Negative indexes work, invalid indexes yield null, and every input row is preserved.
    #[test]
    fn test_select_constant_array_handles_null_and_invalid_indexes() {
        let pipeline = PipelineExpressionBuilder::new("")
            .with_constants(vec![StaticScalarExpression::Map(MapScalarExpression::new(
                QueryLocation::new_fake(),
                HashMap::from([(
                    "items".into(),
                    StaticScalarExpression::Array(ArrayScalarExpression::new(
                        QueryLocation::new_fake(),
                        vec![
                            StaticScalarExpression::Integer(IntegerScalarExpression::new(
                                QueryLocation::new_fake(),
                                10,
                            )),
                            StaticScalarExpression::Integer(IntegerScalarExpression::new(
                                QueryLocation::new_fake(),
                                20,
                            )),
                        ],
                    )),
                )]),
            ))])
            .build()
            .expect("valid pipeline");

        for (indexes, expected) in [
            (
                build_dictionary(
                    vec![
                        Some(0),
                        Some(1),
                        None,
                        Some(2),
                        Some(3),
                        Some(4),
                        Some(5),
                        Some(6),
                        Some(7),
                        Some(8),
                        Some(0),
                    ],
                    vec![
                        ValueOrRef::Integer(0),
                        ValueOrRef::Integer(-1),
                        ValueOrRef::Integer(-2),
                        ValueOrRef::Integer(2),
                        ValueOrRef::Integer(-3),
                        ValueOrRef::Integer(i64::MIN),
                        ValueOrRef::Integer(i64::MAX),
                        ValueOrRef::Boolean(true),
                        ValueOrRef::Null,
                    ],
                ),
                vec![
                    ValueOrRef::Integer(10),
                    ValueOrRef::Integer(20),
                    ValueOrRef::Null,
                    ValueOrRef::Integer(10),
                    ValueOrRef::Null,
                    ValueOrRef::Null,
                    ValueOrRef::Null,
                    ValueOrRef::Null,
                    ValueOrRef::Null,
                    ValueOrRef::Null,
                    ValueOrRef::Integer(10),
                ],
            ),
            (
                Dictionary::new_null(DictionaryKeyType::UInt16, 3),
                vec![ValueOrRef::Null; 3],
            ),
            (Dictionary::new_null(DictionaryKeyType::UInt16, 0), vec![]),
        ] {
            let expression = ScalarExpression::Constant(ReferenceConstantScalarExpression::new(
                QueryLocation::new_fake(),
                ValueType::Integer,
                0,
                ValueAccessor::new_with_selectors(vec![
                    ScalarExpression::Static(StaticScalarExpression::String(
                        StringScalarExpression::new(QueryLocation::new_fake(), "items"),
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

            run_scalar_expression_test_with_state(
                ExecutionContextState::new()
                    .with_pipeline(&pipeline)
                    .with_records(
                        TestRecords::new().with_values(HashMap::from([("index".into(), indexes)])),
                    ),
                expression,
                |result| match result {
                    ResolvedScalarValue::Dictionary(actual) => {
                        assert_eq!(DictionaryKeyType::UInt16, actual.keys().data_type());
                        let rows: Vec<_> = (0..actual.len())
                            .map(|row| actual.get_value(row).expect("valid selected value"))
                            .collect();
                        assert_eq!(expected, rows);
                    }
                    other => panic!("expected dictionary, got {other:?}"),
                },
            );
        }
    }

    /// Scenario: Constants are accessed directly or through scalar array indexes and map keys.
    /// Guarantees: Constant references resolve the requested value without requiring source records.
    #[test]
    fn test_select_from_constants() {
        let pipeline = PipelineExpressionBuilder::new("")
            .with_constants(vec![
                StaticScalarExpression::Integer(IntegerScalarExpression::new(
                    QueryLocation::new_fake(),
                    18,
                )),
                StaticScalarExpression::Array(ArrayScalarExpression::new(
                    QueryLocation::new_fake(),
                    vec![StaticScalarExpression::Integer(
                        IntegerScalarExpression::new(QueryLocation::new_fake(), 99),
                    )],
                )),
                StaticScalarExpression::Map(MapScalarExpression::new(
                    QueryLocation::new_fake(),
                    HashMap::from([(
                        "key1".into(),
                        StaticScalarExpression::Integer(IntegerScalarExpression::new(
                            QueryLocation::new_fake(),
                            100,
                        )),
                    )]),
                )),
            ])
            .build()
            .expect("valid pipeline");

        let select_valid_constant = ReferenceConstantScalarExpression::new(
            QueryLocation::new_fake(),
            ValueType::Integer,
            0,
            ValueAccessor::new(),
        );

        run_scalar_expression_test_with_state(
            ExecutionContextState::new().with_pipeline(&pipeline),
            ScalarExpression::Constant(select_valid_constant),
            |r| match r {
                ResolvedScalarValue::Single(actual) => {
                    assert_eq!(ValueOrRef::Integer(18), actual)
                }
                _ => panic!("test failure"),
            },
        );

        let select_constant_with_int_path = ReferenceConstantScalarExpression::new(
            QueryLocation::new_fake(),
            ValueType::Integer,
            1,
            ValueAccessor::new_with_selectors(vec![ScalarExpression::Static(
                StaticScalarExpression::Integer(IntegerScalarExpression::new(
                    QueryLocation::new_fake(),
                    0,
                )),
            )]),
        );

        run_scalar_expression_test_with_state(
            ExecutionContextState::new().with_pipeline(&pipeline),
            ScalarExpression::Constant(select_constant_with_int_path),
            |r| match r {
                ResolvedScalarValue::Single(actual) => {
                    assert_eq!(ValueOrRef::Integer(99), actual)
                }
                _ => panic!("test failure"),
            },
        );

        let select_constant_with_string_path = ReferenceConstantScalarExpression::new(
            QueryLocation::new_fake(),
            ValueType::Integer,
            2,
            ValueAccessor::new_with_selectors(vec![ScalarExpression::Static(
                StaticScalarExpression::String(StringScalarExpression::new(
                    QueryLocation::new_fake(),
                    "key1",
                )),
            )]),
        );

        run_scalar_expression_test_with_state(
            ExecutionContextState::new().with_pipeline(&pipeline),
            ScalarExpression::Constant(select_constant_with_string_path),
            |r| match r {
                ResolvedScalarValue::Single(actual) => {
                    assert_eq!(ValueOrRef::Integer(100), actual)
                }
                _ => panic!("test failure"),
            },
        );
    }
}
