use std::collections::HashMap;

use crate::{
    execution_context::ExecutionContext, scalar_expressions::execute_scalar_expression,
    summary::summary::SummaryAggregation, *,
};
use data_engine_expressions::*;

pub fn execute_summary_data_expression<'a, TRecord: Record>(
    execution_context: &ExecutionContext<'a, '_, '_, TRecord>,
    summary_data_expression: &'a SummaryDataExpression,
) -> Result<(), ExpressionError> {
    match summary_data_expression {
        SummaryDataExpression::Flatten(s) => {
            let group_by_expressions = s.get_group_by_expressions();

            let mut group_by_values = HashMap::with_capacity(group_by_expressions.len());

            for (key, expression) in group_by_expressions {
                let value = execute_scalar_expression(execution_context, expression)?;

                group_by_values.insert(key.clone(), value);
            }

            let aggregation_expressions = s.get_aggregation_expressions();

            let mut aggregation_values = HashMap::with_capacity(aggregation_expressions.len());

            for (key, expression) in aggregation_expressions {
                let aggregation_function = expression.get_aggregation_function();

                let mut resolved_aggregate_value = None;

                if !matches!(aggregation_function, AggregationFunction::Count) {
                    if let Some(e) = expression.get_value_expression() {
                        resolved_aggregate_value =
                            Some(execute_scalar_expression(execution_context, e)?);
                    } else {
                        execution_context.add_diagnostic_if_enabled(
                            RecordSetEngineDiagnosticLevel::Warn,
                            expression,
                            || {
                                format!(
                                    "Value expression was not specified for '{:?}' aggregation",
                                    aggregation_function
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

                        if let Value::Integer(i) = aggregate_value {

                        }
                        else if let Value::Integer(i) = aggregate_value {

                        }
                        match aggregate_value.convert_to_double() {
                            Some(v) => {
                                aggregation_values
                                    .insert(key.clone(), SummaryAggregation::Average { count: 1, sum: v });
                            }
                            None => {
                                execution_context.add_diagnostic_if_enabled(
                                    RecordSetEngineDiagnosticLevel::Warn,
                                    expression.get_value_expression().as_ref().unwrap(),
                                    || {
                                        format!(
                                            "Value expression value of '{:?}' type could not be converted to double",
                                            aggregate_value.get_value_type()
                                        )
                                    },
                                );
                            }
                        }
                    }
                    AggregationFunction::Count => {
                        aggregation_values.insert(key.clone(), SummaryAggregation::Count { value: 1 });
                    }
                    AggregationFunction::Maximum => todo!(),
                    AggregationFunction::Minimum => todo!(),
                    AggregationFunction::Sum => todo!(),
                };
            }

            let result = execution_context
                .get_summaries()
                .create_or_update_summary(group_by_values, aggregation_values);

            if result {
                // todo: log created
            } else {
                // todo: log updated
            }
        }
    }

    Ok(())
}
