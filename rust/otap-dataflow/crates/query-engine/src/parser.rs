// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Utilities for configuring [`Parser`](data_engine_kql_parser::Parser) to parse program for OTAP
//! query-engine

use data_engine_parser_abstractions::ParserOptions;

use crate::consts::{ENCODE_FUNC_NAME, SHA256_FUNC_NAME};

/// Create parser options that can be used when parsing an expression that will be executed with
/// this query engine
pub fn default_parser_options() -> ParserOptions {
    ParserOptions::new()
        // Add placeholders for scalar UDFs supported by this engine - these are needed because
        // the invoke function expression in our expression AST references the function by an ID,
        // adding these will make a named function with some ID available in the parser. The
        // arguments/return types are not populated, as these are currently not needed by the
        // parser, but they will be validated by the query planner.
        .with_external_function(SHA256_FUNC_NAME, vec![], None)
        .with_external_function(ENCODE_FUNC_NAME, vec![], None)
}
