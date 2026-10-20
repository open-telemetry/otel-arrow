// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use chrono::Utc;
use otel_arrow_contrib_data_engine_expressions::*;

use crate::{execution_context::ExecutionContext, resolved_value::*, *};

pub fn execute_temporal_scalar_expression<'a, 'pipeline, TRecords: ColumnarRecords<'pipeline>>(
    _execution_context: &'a ExecutionContext<'a, 'pipeline, TRecords>,
    temporal_scalar_expression: &'pipeline TemporalScalarExpression,
) -> ResolvedScalarValue<'pipeline, 'a> {
    match temporal_scalar_expression {
        TemporalScalarExpression::Now(_) => {
            ResolvedScalarValue::Single(ValueOrRef::DateTime(Utc::now().into()))
        }
    }
}

#[cfg(test)]
mod tests {
    use chrono::Offset;

    use crate::test_helpers::*;

    use super::*;

    #[test]
    fn test_execute_now_temporal_scalar_expression() {
        run_scalar_expression_test(
            TestRecords::new(),
            ScalarExpression::Temporal(TemporalScalarExpression::Now(NowScalarExpression::new(
                QueryLocation::new_fake(),
            ))),
            |r| match r {
                ResolvedScalarValue::Single(value) => {
                    assert_eq!(ValueType::DateTime, value.get_value_type());

                    if let Value::DateTime(d) = value.to_value() {
                        assert_eq!(Utc::now().timezone().fix(), d.get_value().timezone());
                    } else {
                        panic!("Value wasn't a DateTime");
                    }
                }
                _ => panic!("test failure"),
            },
        );
    }
}
