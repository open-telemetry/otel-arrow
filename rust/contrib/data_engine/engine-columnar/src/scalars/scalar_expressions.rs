// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

#![allow(unused)]

use otel_arrow_contrib_data_engine_expressions::*;

use crate::{
    execution_context::ExecutionContext,
    resolved_value::*,
    scalars::{
        attached_scalar_expression::execute_attached_scalar_expression,
        constant_scalar_expression::execute_constant_scalar_expression,
        source_scalar_expression::execute_source_scalar_expression,
        temporal_scalar_expression::execute_temporal_scalar_expression,
    },
    *,
};

pub fn execute_scalar_expression<'a, 'pipeline, TRecords: ColumnarRecords<'pipeline>>(
    execution_context: &'a ExecutionContext<'a, 'pipeline, TRecords>,
    scalar_expression: &'pipeline ScalarExpression,
) -> ResolvedScalarValue<'pipeline, 'a> {
    let value = match scalar_expression {
        ScalarExpression::Argument(_) => todo!(),
        ScalarExpression::Attached(a) => execute_attached_scalar_expression(execution_context, a),
        ScalarExpression::Case(_) => todo!(),
        ScalarExpression::Coalesce(_) => todo!(),
        ScalarExpression::Collection(_) => todo!(),
        ScalarExpression::Conditional(_) => todo!(),
        ScalarExpression::Constant(c) => execute_constant_scalar_expression(execution_context, c),
        ScalarExpression::Convert(_) => todo!(),
        ScalarExpression::GetType(_) => todo!(),
        ScalarExpression::GetRecordType(_) => todo!(),
        ScalarExpression::InvokeFunction(_) => todo!(),
        ScalarExpression::Length(_) => todo!(),
        ScalarExpression::Logical(_) => todo!(),
        ScalarExpression::Math(_) => todo!(),
        ScalarExpression::Parse(_) => todo!(),
        ScalarExpression::Select(_) => todo!(),
        ScalarExpression::Slice(_) => todo!(),
        ScalarExpression::Source(s) => execute_source_scalar_expression(execution_context, s),
        ScalarExpression::Static(s) => ResolvedScalarValue::new_from_value(s.to_value()),
        ScalarExpression::Temporal(t) => execute_temporal_scalar_expression(execution_context, t),
        ScalarExpression::Text(_) => todo!(),
        ScalarExpression::Variable(_) => todo!(),
    };

    execution_context.add_diagnostic_if_enabled(
        ColumnarEngineDiagnosticLevel::Verbose,
        scalar_expression,
        || format!("Evaluated as: {value}"),
    );

    value
}
