use std::collections::HashMap;

use crate::{
    execution_context::ExecutionContext, scalar_expressions::execute_scalar_expression, *,
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

                group_by_values.insert(key.as_ref().to_string(), value);
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
                                aggregation_values.insert(
                                    key.as_ref().to_string(),
                                    SummaryAggregationUpdate::Average(v),
                                );
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
                        aggregation_values
                            .insert(key.as_ref().to_string(), SummaryAggregationUpdate::Count);
                    }
                    AggregationFunction::Maximum => {
                        let aggregate_value =
                            resolved_aggregate_value.expect("Value was not resolved");

                        aggregation_values.insert(
                            key.as_ref().to_string(),
                            SummaryAggregationUpdate::Maximum(aggregate_value),
                        );
                    }
                    AggregationFunction::Minimum => {
                        let aggregate_value =
                            resolved_aggregate_value.expect("Value was not resolved");

                        aggregation_values.insert(
                            key.as_ref().to_string(),
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
                                aggregation_values.insert(
                                    key.as_ref().to_string(),
                                    SummaryAggregationUpdate::Sum(v),
                                );
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
                s,
                group_by_values,
                aggregation_values,
            );
        }
    }

    Ok(())
}

fn get_summary_value(value: &Value) -> Option<SummaryValue> {
    match value {
        Value::Integer(i) => Some(SummaryValue::Integer(i.get_value())),
        Value::Double(d) => Some(SummaryValue::Double(d.get_value())),
        _ => value.convert_to_double().map(SummaryValue::Double),
    }
}
