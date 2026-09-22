// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Implementation of OpenTelemetry Transformation Language (OTTL) parser.

pub(crate) mod editor_expression;
pub mod parser;
pub(crate) mod scalar_expression;
pub(crate) mod scalar_primitive_expression;

pub use otel_arrow_contrib_data_engine_parser_abstractions::parse_standard_bool_literal;
pub use otel_arrow_contrib_data_engine_parser_abstractions::parse_standard_null_literal;
pub use parser::{OttlParser, Rule};

// Note: Re-export Parser API surface so users don't need to also depend on
// parser-abstractions crate just to parse queries.
pub use otel_arrow_contrib_data_engine_parser_abstractions::Parser;
pub use otel_arrow_contrib_data_engine_parser_abstractions::ParserError;
pub use otel_arrow_contrib_data_engine_parser_abstractions::ParserMapKeySchema;
pub use otel_arrow_contrib_data_engine_parser_abstractions::ParserMapSchema;
pub use otel_arrow_contrib_data_engine_parser_abstractions::ParserOptions;
pub use otel_arrow_contrib_data_engine_parser_abstractions::ParserResult;
