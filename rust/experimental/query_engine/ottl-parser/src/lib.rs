// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

pub(crate) mod ottl_parser;
pub(crate) mod scalar_expression;
pub(crate) mod scalar_primitive_expression;

pub use data_engine_parser_abstractions::parse_standard_bool_literal;
pub use data_engine_parser_abstractions::parse_standard_null_literal;
pub use ottl_parser::*;

// Note: Re-export Parser API surface so users don't need to also depend on
// parser-abstractions crate just to parse queries.
pub use data_engine_parser_abstractions::Parser;
pub use data_engine_parser_abstractions::ParserError;
pub use data_engine_parser_abstractions::ParserMapKeySchema;
pub use data_engine_parser_abstractions::ParserMapSchema;
pub use data_engine_parser_abstractions::ParserOptions;
