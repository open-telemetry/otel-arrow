// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use otel_arrow_contrib_data_engine_expressions::*;

use crate::{
    execution_context::ExecutionContext, resolved_value::*, selection::select_from_record_table, *,
};

pub fn execute_attached_scalar_expression<'a, 'pipeline, TRecords: ColumnarRecords<'pipeline>>(
    execution_context: &'a ExecutionContext<'a, 'pipeline, TRecords>,
    attached_scalar_expression: &'pipeline AttachedScalarExpression,
) -> ResolvedScalarValue<'pipeline, 'a> {
    let record = match execution_context.get_records() {
        Some(r) => r,
        None => {
            execution_context.add_diagnostic_if_enabled(
                ColumnarEngineDiagnosticLevel::Warn,
                attached_scalar_expression,
                || "Attached data could not be found".into(),
            );
            return ResolvedScalarValue::new_null();
        }
    };

    let name = attached_scalar_expression.get_name().get_value();

    let attached_record = match record.get_attached_records(name) {
        Some(a) => a,
        None => {
            execution_context.add_diagnostic_if_enabled(
                ColumnarEngineDiagnosticLevel::Warn,
                attached_scalar_expression,
                || format!("Attached record matching name '{name}' could not be found"),
            );
            return ResolvedScalarValue::new_null();
        }
    };

    select_from_record_table(
        execution_context,
        attached_record,
        attached_scalar_expression
            .get_value_accessor()
            .get_selectors(),
    )
}

#[cfg(test)]
mod tests {
    use std::{cell::RefCell, collections::HashMap};

    use crate::test_helpers::*;

    use super::*;

    /// Scenario: Attached access is evaluated without source records or without the named attachment.
    /// Guarantees: Each absence yields null and one warning attributed to the attached expression.
    #[test]
    fn test_select_from_missing_attached_data() {
        for (records, expected_message) in [
            (None, "Attached data could not be found"),
            (
                Some(TestRecords::new()),
                "Attached record matching name 'data' could not be found",
            ),
        ] {
            let diagnostics = RefCell::new(vec![]);
            let pipeline = PipelineExpression::default();
            let expression = AttachedScalarExpression::new(
                QueryLocation::new_fake(),
                StringScalarExpression::new(QueryLocation::new_fake(), "data"),
                ValueAccessor::new(),
            );
            let context = ExecutionContext::new(
                ColumnarEngineDiagnosticLevel::Warn,
                &diagnostics,
                &pipeline,
                records,
            );

            let result = execute_attached_scalar_expression(&context, &expression);

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
            assert_eq!(expected_message, diagnostics[0].get_message());
            assert!(std::ptr::addr_eq(
                diagnostics[0].get_expression(),
                &expression as &dyn Expression,
            ));
        }
    }

    /// Scenario: Attached access selects a column, an absent attachment, or an existing attachment root.
    /// Guarantees: Columns retain values, missing attachments yield null, and root access returns a table.
    #[test]
    fn test_select_from_attached_table() {
        let values_dictionary = build_dictionary(
            vec![Some(0), Some(0)],
            vec![ValueOrRef::String(StringValueOrRef::new_owned(
                "hello world".into(),
            ))],
        );

        let select_valid_attached_data = AttachedScalarExpression::new(
            QueryLocation::new_fake(),
            StringScalarExpression::new(QueryLocation::new_fake(), "data"),
            ValueAccessor::new_with_selectors(vec![ScalarExpression::Static(
                StaticScalarExpression::String(StringScalarExpression::new(
                    QueryLocation::new_fake(),
                    "values",
                )),
            )]),
        );

        run_scalar_expression_test(
            TestRecords::new().with_attached_records(HashMap::from([(
                "data".into(),
                TestRecords::new().with_values(HashMap::from([(
                    "values".into(),
                    values_dictionary.clone(),
                )])),
            )])),
            ScalarExpression::Attached(select_valid_attached_data),
            |r| match r {
                ResolvedScalarValue::Dictionary(actual) => {
                    assert_eq!(values_dictionary, actual)
                }
                _ => panic!("test failure"),
            },
        );

        let select_invalid_attached_data = AttachedScalarExpression::new(
            QueryLocation::new_fake(),
            StringScalarExpression::new(QueryLocation::new_fake(), "invalid"),
            ValueAccessor::new_with_selectors(vec![ScalarExpression::Static(
                StaticScalarExpression::String(StringScalarExpression::new(
                    QueryLocation::new_fake(),
                    "unknown",
                )),
            )]),
        );

        run_scalar_expression_test(
            TestRecords::new(),
            ScalarExpression::Attached(select_invalid_attached_data),
            |r| {
                assert!(matches!(r, ResolvedScalarValue::Single(ValueOrRef::Null)));
            },
        );

        let select_root = AttachedScalarExpression::new(
            QueryLocation::new_fake(),
            StringScalarExpression::new(QueryLocation::new_fake(), "data"),
            ValueAccessor::new(),
        );

        run_scalar_expression_test(
            TestRecords::new()
                .with_attached_records(HashMap::from([("data".into(), TestRecords::new())])),
            ScalarExpression::Attached(select_root),
            |r| {
                assert!(matches!(r, ResolvedScalarValue::Table(_)));
            },
        );
    }
}
