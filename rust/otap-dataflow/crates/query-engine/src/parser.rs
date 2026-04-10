// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Utilities for configuring [`Parser`](data_engine_kql_parser::Parser) to parse program for OTAP
//! query-engine

use data_engine_expressions::{
    NullScalarExpression, PipelineFunctionParameter, PipelineFunctionParameterType, QueryLocation,
    ScalarExpression, StaticScalarExpression,
};
use data_engine_parser_abstractions::ParserOptions;

use crate::consts::{ENCODE_FUNC_NAME, REGEXP_SUBSTR_FUNC_NAME, SHA256_FUNC_NAME};

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
        .with_external_function(SHA256_FUNC_NAME, param_placeholders(1), None)
        .with_external_function(ENCODE_FUNC_NAME, param_placeholders(2), None)
        .with_external_function(
            REGEXP_SUBSTR_FUNC_NAME,
            param_placeholders_some_optional(2, 4),
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

fn param_placeholders_with_default_value(
    num_params: usize,
) -> Vec<(
    &'static str,
    PipelineFunctionParameter,
    Option<ScalarExpression>,
)> {
    // TODO in the future we will have try to have a better mechanism for specifying
    // the function signature. For now, just define as many unique parameter names as
    // needed. This helper function is only used internally, and gets called during tests
    // so we know this won't panic at runtime.
    static PARAM_NAMES: &[&'static str] = &["1", "2", "3", "4", "5"];
    let mut params = Vec::with_capacity(num_params);

    for i in 0..num_params {
        params.push((
            PARAM_NAMES[i],
            PipelineFunctionParameter::new(
                QueryLocation::new_fake(),
                PipelineFunctionParameterType::Scalar(None),
            ),
            Some(ScalarExpression::Static(StaticScalarExpression::Null(
                NullScalarExpression::new(QueryLocation::new_fake()),
            ))),
        ))
    }

    params
}

fn param_placeholders_some_optional(
    required: usize,
    optional: usize,
) -> Vec<(
    &'static str,
    PipelineFunctionParameter,
    Option<ScalarExpression>,
)> {
    let mut required = param_placeholders(required);
    let mut optional = param_placeholders_with_default_value(optional);
    required.append(&mut optional);

    required
}
