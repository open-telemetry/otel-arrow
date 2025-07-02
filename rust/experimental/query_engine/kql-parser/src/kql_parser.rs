use data_engine_expressions::*;
use data_engine_parser_abstractions::*;
use pest_derive::Parser;

use crate::query_expression::parse_query;

#[derive(Parser)]
#[grammar = "kql.pest"]
pub(crate) struct KqlParser;

pub fn parse(query: &str) -> Result<PipelineExpression, Vec<ParserError>> {
    parse_with_options(query, ParserOptions::new())
}

pub fn parse_with_options(
    query: &str,
    options: ParserOptions,
) -> Result<PipelineExpression, Vec<ParserError>> {
    parse_query(query, options)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn test_parse() {
        assert!(parse("a").is_ok());
        assert!(parse("let a = 1").is_err());
        assert!(parse("i | extend a = 1 i | extend b = 2").is_err());
    }
}
