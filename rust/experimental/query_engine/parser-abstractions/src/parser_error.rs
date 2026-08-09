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
    /// Converts the pest error to a syntax error, identifying the invalid content from the error
    /// location information.
    pub fn from_pest_error(query: &str, pest_error: Error<impl pest::RuleType>) -> Self {
        let (start, end) = match pest_error.location {
            pest::error::InputLocation::Pos(p) => (0, p),
            pest::error::InputLocation::Span(s) => s,
        };

        let (line, column) = match pest_error.line_col {
            pest::error::LineColLocation::Pos(p) => p,
            pest::error::LineColLocation::Span(l, _) => l,
        };

        // try to identify invalid syntax on the line/column that contains it.
        let invalid_line = query
            .lines()
            .nth(line - 1)
            .and_then(|line| line.get(column - 1..));

        // if for some reason the error's line isn't in the query (which can happen for empty
        // queries, e.g. "", or if we're simply passed an error with a bad line), fall back to
        // identifying the invalid content by start..end range. If the range is also not valid
        // for the query, just fall back to using the query contents.
        let content = invalid_line.or(query.get(start..end)).unwrap_or(query);

        Self::SyntaxNotSupported(
            QueryLocation::new(start, end, line, column)
                .expect("QueryLocation could not be constructed"),
            format!("Syntax '{content}' supplied in query is not supported"),
        )
    }
}

#[cfg(test)]
mod test {
    use pest::Position;
    use pest::error::{ErrorVariant, LineColLocation};

    use super::*;

    /// Scenario: Converting a pest error whose line/column does not exist in the provided query.
    /// Guarantees: `ParserError::from_pest_error` does not panic and falls back to a safe content slice.
    #[test]
    fn test_from_pest_error_invalid_location() {
        let pest_error = Error::<()>::new_from_pos(
            ErrorVariant::CustomError {
                message: "test".into(),
            },
            Position::new("a\nabc", 4).unwrap(),
        );

        // ensure we don't panic when the line doesn't exist
        let (line, col) = match pest_error.line_col {
            LineColLocation::Pos(p) => p,
            _ => {
                panic!("invalid pos")
            }
        };
        assert!(line > 0);
        assert!(col > 0);
        let error = ParserError::from_pest_error("abcde", pest_error.clone());
        assert!(
            error
                .to_string()
                .contains("Syntax 'abcd' supplied in query is not supported"),
        );

        let error = ParserError::from_pest_error("ab\nc", pest_error.clone());
        assert!(
            error
                .to_string()
                .contains("Syntax 'ab\nc' supplied in query is not supported"),
        );

        // ensure we don't panic when the position doesn't exist:
        let (_, end) = match pest_error.location {
            pest::error::InputLocation::Pos(p) => (0, p),
            pest::error::InputLocation::Span(s) => s,
        };
        assert!(end > 1);
        let error = ParserError::from_pest_error("ab", pest_error);
        assert!(
            error
                .to_string()
                .contains("Syntax 'ab' supplied in query is not supported"),
        );
    }
}
