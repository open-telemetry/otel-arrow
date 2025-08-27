// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use data_engine_expressions::*;

use chrono::Utc;

use crate::{execution_context::*, *};

pub fn execute_temporal_scalar_expression<'a, 'b, 'c, TRecord: Record>(
    execution_context: &'b ExecutionContext<'a, '_, '_, TRecord>,
    temporal_scalar_expression: &'a TemporalScalarExpression,
) -> Result<ResolvedValue<'c>, ExpressionError>
where
    'a: 'c,
    'b: 'c,
{
    let value = match temporal_scalar_expression {
        TemporalScalarExpression::Now(_) => ResolvedValue::Computed(OwnedValue::DateTime(
            DateTimeValueStorage::new(Utc::now().into()),
        )),
    };

    execution_context.add_diagnostic_if_enabled(
        RecordSetEngineDiagnosticLevel::Verbose,
        temporal_scalar_expression,
        || format!("Evaluated as: '{value}'"),
    );

    Ok(value)
}

#[cfg(test)]
mod tests {
    use chrono::Offset;

    use crate::scalars::execute_scalar_expression;

    use super::*;

    #[test]
    fn test_execute_now_temporal_scalar_expression() {
        let mut test = TestExecutionContext::new();

        let execution_context = test.create_execution_context();

        let scalar_expression = ScalarExpression::Temporal(TemporalScalarExpression::Now(
            NowScalarExpression::new(QueryLocation::new_fake()),
        ));

        let value = execute_scalar_expression(&execution_context, &scalar_expression).unwrap();

        assert_eq!(ValueType::DateTime, value.get_value_type());

        if let Value::DateTime(d) = value.to_value() {
            assert_eq!(Utc::now().timezone().fix(), d.get_value().timezone());
        } else {
            panic!("Value wasn't a DateTime");
        }
    }
}
