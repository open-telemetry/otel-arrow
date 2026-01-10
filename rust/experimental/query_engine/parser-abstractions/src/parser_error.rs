// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use data_engine_expressions::{ExpressionError, QueryLocation};
use pest::error::Error;
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

    #[error("{0}")]
    SchemaError(String),

    #[error("The name '{key}' does not refer to any known key on the target map")]
    KeyNotFound {
        location: QueryLocation,
        key: String,
    },
}

impl From<&ExpressionError> for ParserError {
    fn from(value: &ExpressionError) -> Self {
        ParserError::SyntaxError(value.get_query_location().clone(), value.to_string())
    }
}

impl ParserError {
    pub fn from_pest_error(query: &str, pest_error: Error<impl pest::RuleType>) -> Self {
        let (start, end) = match pest_error.location {
            pest::error::InputLocation::Pos(p) => (0, p),
            pest::error::InputLocation::Span(s) => s,
        };

        let (line, column) = match pest_error.line_col {
            pest::error::LineColLocation::Pos(p) => p,
            pest::error::LineColLocation::Span(l, _) => l,
        };

        let content = if line > 0 && column > 0 {
            &query
                .lines()
                .nth(line - 1)
                .expect("Query line did not exist")[column - 1..]
        } else {
            &query[start..end]
        };

        Self::SyntaxNotSupported(
            QueryLocation::new(start, end, line, column)
                .expect("QueryLocation could not be constructed"),
            format!("Syntax '{content}' supplied in query is not supported"),
        )
    }
}
