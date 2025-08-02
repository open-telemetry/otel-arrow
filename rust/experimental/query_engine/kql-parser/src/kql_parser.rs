use data_engine_expressions::*;
use data_engine_parser_abstractions::*;
use pest_derive::Parser;

use crate::query_expression::parse_query;

#[derive(Parser)]
#[grammar = "kql.pest"]
pub(crate) struct KqlPestParser;

pub struct KqlParser {}

impl Parser for KqlParser {
    fn parse_with_options(
        query: &str,
        options: ParserOptions,
    ) -> Result<PipelineExpression, Vec<ParserError>> {
        parse_query(query, options)
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

    #[test]
    pub fn test_parse_case_expressions() {
        // Test simple case expression
        assert!(KqlParser::parse("events | extend status = case(true, \"success\", \"failure\")").is_ok());
        
        // Test case expression with multiple conditions
        assert!(KqlParser::parse("events | extend category = case(level == \"info\", \"information\", level == \"error\", \"problem\", \"other\")").is_ok());
        
        // Test case expression with various types
        assert!(KqlParser::parse("events | extend result = case(count > 10, 1, count > 5, 2, 0)").is_ok());
    }
}
