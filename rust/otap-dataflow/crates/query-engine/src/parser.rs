// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Utilities for configuring [`Parser`](data_engine_kql_parser::Parser) to parse program for OTAP
//! query-engine

use data_engine_expressions::{
    IntegerScalarExpression, NullScalarExpression, PipelineFunctionParameter,
    PipelineFunctionParameterType, QueryLocation, ScalarExpression, StaticScalarExpression,
};
use data_engine_parser_abstractions::ParserOptions;

use crate::consts::{
    ENCODE_FUNC_NAME, FORMAT_DATETIME_FUNC_NAME, LOWER_CASE_FUNC_NAME, LTRIM_FUNC_NAME,
    REGEXP_SUBSTR_FUNC_NAME, RTRIM_FUNC_NAME, SHA256_FUNC_NAME, UPPER_CASE_FUNC_NAME,
    UUID_FUNC_NAME, UUIDV7_FUNC_NAME,
};

/// Create parser options that can be used when parsing an expression that will be executed with
/// this query engine
#[must_use]
pub fn default_parser_options() -> ParserOptions {
    ParserOptions::new()
        // Add placeholders for scalar UDFs supported by this engine - these are needed because
        // the invoke function expression in our expression AST references the function by an ID,
        // adding these will make a named function with some ID available in the parser. Only the
        // number of arguments is currently validated by the parser, but the rest of the parameter
        // definitions are left as placeholders. Additional parameter validation happens at query
        // planning time.
        //
        // Note that for functions that take optional parameters, we are currently explicitly
        // filling in the default values, because our expression tree doesn't have the concept of
        // optional parameters (signatures with different arities), even though the underlying
        // function might support this. Eventually we may clean this up with modifications to the
        // expression tree.
        //
        .with_external_function(FORMAT_DATETIME_FUNC_NAME, param_placeholders(2), None)
        .with_external_function(SHA256_FUNC_NAME, param_placeholders(1), None)
        .with_external_function(ENCODE_FUNC_NAME, param_placeholders(2), None)
        .with_external_function(UUID_FUNC_NAME, param_placeholders(0), None)
        .with_external_function(UUIDV7_FUNC_NAME, param_placeholders(0), None)
        .with_external_function(UPPER_CASE_FUNC_NAME, param_placeholders(1), None)
        .with_external_function(LOWER_CASE_FUNC_NAME, param_placeholders(1), None)
        .with_external_function(LTRIM_FUNC_NAME, param_placeholders(2), None)
        .with_external_function(RTRIM_FUNC_NAME, param_placeholders(2), None)
        .with_external_function(
            REGEXP_SUBSTR_FUNC_NAME,
            vec![
                (
                    "",
                    PipelineFunctionParameter::new(
                        QueryLocation::new_fake(),
                        PipelineFunctionParameterType::Scalar(None),
                    ),
                    None,
                ),
                (
                    "",
                    PipelineFunctionParameter::new(
                        QueryLocation::new_fake(),
                        PipelineFunctionParameterType::Scalar(None),
                    ),
                    None,
                ),
                (
                    "start",
                    PipelineFunctionParameter::new(
                        QueryLocation::new_fake(),
                        PipelineFunctionParameterType::Scalar(None),
                    ),
                    Some(ScalarExpression::Static(StaticScalarExpression::Integer(
                        IntegerScalarExpression::new(QueryLocation::new_fake(), 1),
                    ))),
                ),
                (
                    "occurrence",
                    PipelineFunctionParameter::new(
                        QueryLocation::new_fake(),
                        PipelineFunctionParameterType::Scalar(None),
                    ),
                    Some(ScalarExpression::Static(StaticScalarExpression::Integer(
                        IntegerScalarExpression::new(QueryLocation::new_fake(), 1),
                    ))),
                ),
                (
                    "flags",
                    PipelineFunctionParameter::new(
                        QueryLocation::new_fake(),
                        PipelineFunctionParameterType::Scalar(None),
                    ),
                    Some(ScalarExpression::Static(StaticScalarExpression::Null(
                        NullScalarExpression::new(QueryLocation::new_fake()),
                    ))),
                ),
                (
                    "group",
                    PipelineFunctionParameter::new(
                        QueryLocation::new_fake(),
                        PipelineFunctionParameterType::Scalar(None),
                    ),
                    Some(ScalarExpression::Static(StaticScalarExpression::Integer(
                        IntegerScalarExpression::new(QueryLocation::new_fake(), 0),
                    ))),
                ),
            ],
            None,
        )
}

fn param_placeholders(
    num_params: usize,
) -> Vec<(
    &'static str,
    PipelineFunctionParameter,
    Option<ScalarExpression>,
)> {
    let mut params = Vec::with_capacity(num_params);
    for _ in 0..num_params {
        params.push((
            "",
            PipelineFunctionParameter::new(
                QueryLocation::new_fake(),
                PipelineFunctionParameterType::Scalar(None),
            ),
            None,
        ))
    }

    params
}
