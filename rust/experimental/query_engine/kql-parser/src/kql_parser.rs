// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use data_engine_expressions::QueryLocation;
use data_engine_kql_parser_macros::BaseRuleCompatible;
use data_engine_parser_abstractions::*;
use pest::error::Error;
use pest_derive::Parser;

use crate::query_expression::parse_query;

#[derive(Parser, BaseRuleCompatible)]
#[grammar = "base.pest"]
#[grammar = "kql.pest"]
pub(crate) struct KqlPestParser;

pub struct KqlParser {}

impl Parser for KqlParser {
    fn parse_with_options(
        query: &str,
        options: ParserOptions,
    ) -> Result<ParserResult, Vec<ParserError>> {
        let pipeline = parse_query(query, options)?;
        Ok(ParserResult::new(pipeline))
    }
}

pub(crate) fn map_kql_errors(error: ParserError) -> ParserError {
    match error {
        ParserError::KeyNotFound { location, key } => ParserError::QueryLanguageDiagnostic {
            location,
            diagnostic_id: "KS142",
            message: format!(
                "The name '{key}' does not refer to any known column, table, variable or function"
            ),
        },
        e => e,
    }
}

pub fn map_parse_error<R>(query: &str, pest_error: Error<R>) -> ParserError {
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

    ParserError::SyntaxNotSupported(
        QueryLocation::new(start, end, line, column)
            .expect("QueryLocation could not be constructed"),
        format!("Syntax '{content}' supplied in query is not supported"),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn test_parse() {
        assert!(KqlParser::parse("a").is_ok());
        assert!(KqlParser::parse("let a = 1").is_err());
        assert!(KqlParser::parse("i | extend a = 1 i | extend b = 2").is_err());
    }
}
