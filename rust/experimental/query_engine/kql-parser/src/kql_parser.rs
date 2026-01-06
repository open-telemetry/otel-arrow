// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use data_engine_parser_abstractions::*;
use data_engine_parser_macros::{BaseRuleCompatible, ScalarExprPrattParser};
use pest_derive::Parser;

use crate::{base_parser, kql_parser, query_expression::parse_query};

#[derive(Parser, BaseRuleCompatible, ScalarExprPrattParser)]
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
