// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use data_engine_expressions::{ExpressionError, QueryLocation};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ParserError {
    #[error("{1}")]
    SyntaxNotSupported(QueryLocation, String),

    #[error("{1}")]
    SyntaxError(QueryLocation, String),

    #[error("{diagnostic_id}: {message}")]
    QueryLanguageDiagnostic {
        location: QueryLocation,
        diagnostic_id: &'static str,
        message: String,
    },
}

impl From<&ExpressionError> for ParserError {
    fn from(value: &ExpressionError) -> Self {
        ParserError::SyntaxError(value.get_query_location().clone(), value.to_string())
    }
}
