// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Utilities for configuring [`Parser`](data_engine_kql_parser::Parser) to parse program for OTAP query-engine

use data_engine_expressions::ValueType;
use data_engine_parser_abstractions::ParserOptions;

use crate::consts::{ENCODE_FUNC_NAME, SHA256_FUNC_NAME};

///
pub fn default_parser_options() -> ParserOptions {
    ParserOptions::new()
        // TODO - need a return type for binary
        // TODO - do we actually need to add the param types?
        .with_external_function(SHA256_FUNC_NAME, vec![], None)
        .with_external_function(ENCODE_FUNC_NAME, vec![], Some(ValueType::String))
}
