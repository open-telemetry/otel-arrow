// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use data_engine_expressions::*;

use crate::{execution_context::*, scalars::execute_scalar_expression, *};

pub fn execute_collection_scalar_expression<'a, 'b, 'c, TRecord: Record>(
    execution_context: &'b ExecutionContext<'a, '_, '_, TRecord>,
    collection_scalar_expression: &'a CollectionScalarExpression,
) -> Result<ResolvedValue<'c>, ExpressionError>
where
    'a: 'c,
    'b: 'c,
{
    match collection_scalar_expression {
        CollectionScalarExpression::Concat(c) => {
            match execute_scalar_expression(execution_context, c.get_values_expression())?
                .try_resolve_array()
            {
                Ok(a) => {
                    let values = a.to_vec((..).into(), |i, r| {
                        match r.try_resolve_array() {
                            Ok(v) => Ok(v),
                            Err(v) => {
                                Err(ExpressionError::TypeMismatch(
                                    c.get_values_expression().get_query_location().clone(),
                                    format!(
                                        "Value of '{:?}' type returned by scalar expression at index '{i}' could not be converted to an array",
                                        v.get_value_type()
                                    ),
                                ))
                            }
                        }
                    })?;

                    let r = ResolvedValue::Sequence(Sequence::new(values));

                    execution_context.add_diagnostic_if_enabled(
                        RecordSetEngineDiagnosticLevel::Verbose,
                        collection_scalar_expression,
                        || format!("Evaluated as: '{r}'"),
                    );

                    Ok(r)
                }
                Err(orig) => Err(ExpressionError::TypeMismatch(
                    c.get_values_expression().get_query_location().clone(),
                    format!(
                        "Value of '{:?}' type returned by scalar expression was not an array",
                        orig.get_value_type()
                    ),
                )),
            }
        }
        CollectionScalarExpression::List(c) => {
            let expressions = c.get_value_expressions();

            let mut values = Vec::with_capacity(expressions.len());

            for v in expressions {
                values.push(execute_scalar_expression(execution_context, v)?);
            }

            let r = ResolvedValue::List(List::new(values));

            execution_context.add_diagnostic_if_enabled(
                RecordSetEngineDiagnosticLevel::Verbose,
                collection_scalar_expression,
                || format!("Evaluated as: '{r}'"),
            );

            Ok(r)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn text_execute_list_scalar_expression() {
        fn run_test_success(input: Vec<ScalarExpression>, expected_value: Value) {
            let expression = ScalarExpression::Collection(CollectionScalarExpression::List(
                ListScalarExpression::new(QueryLocation::new_fake(), input),
            ));

            let mut test = TestExecutionContext::new();

            let execution_context = test.create_execution_context();

            let actual_value = execute_scalar_expression(&execution_context, &expression).unwrap();
            assert_eq!(expected_value, actual_value.to_value());
        }

        run_test_success(
            vec![],
            OwnedValue::Array(ArrayValueStorage::new(vec![])).to_value(),
        );

        run_test_success(
            vec![
                ScalarExpression::Static(StaticScalarExpression::Integer(
                    IntegerScalarExpression::new(QueryLocation::new_fake(), 1),
                )),
                ScalarExpression::Static(StaticScalarExpression::Integer(
                    IntegerScalarExpression::new(QueryLocation::new_fake(), 2),
                )),
            ],
            OwnedValue::Array(ArrayValueStorage::new(vec![
                OwnedValue::Integer(IntegerValueStorage::new(1)),
                OwnedValue::Integer(IntegerValueStorage::new(2)),
            ]))
            .to_value(),
        );
    }
}
