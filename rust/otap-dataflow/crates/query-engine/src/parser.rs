// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Utilities for configuring [`Parser`](data_engine_kql_parser::Parser) to parse program for OTAP
//! query-engine

use data_engine_expressions::{
    PipelineFunctionParameter, PipelineFunctionParameterType, QueryLocation, ScalarExpression,
};
use data_engine_parser_abstractions::ParserOptions;

use crate::consts::{ENCODE_FUNC_NAME, SHA256_FUNC_NAME};

/// Create parser options that can be used when parsing an expression that will be executed with
/// this query engine
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
