use data_engine_expressions::*;
use data_engine_parser_abstractions::*;
use pest::Parser;
use pest_derive::Parser;

use crate::query_expression::parse_query;

#[derive(Parser)]
#[grammar = "kql.pest"]
pub(crate) struct KqlParser;

pub fn parse(query: &str) -> Result<PipelineExpression, ParserError> {
    parse_with_options(query, ParserOptions::new())
}

pub fn parse_with_options(
    query: &str,
    options: ParserOptions,
) -> Result<PipelineExpression, ParserError> {
    let mut state = ParserState::new_with_options(query, options);

    let parse_result = KqlParser::parse(Rule::query, query);

    if parse_result.is_err() {
        let pest_error = parse_result.unwrap_err();

        let (start, end) = match pest_error.location {
            pest::error::InputLocation::Pos(p) => (0, p),
            pest::error::InputLocation::Span(s) => s,
        };

        let (line, column) = match pest_error.line_col {
            pest::error::LineColLocation::Pos(p) => p,
            pest::error::LineColLocation::Span(l, _) => l,
        };

        return Err(ParserError::SyntaxNotSupported(
            QueryLocation::new(start, end, line, column),
            pest_error.variant.message().into(),
        ));
    }

    parse_query(parse_result.unwrap().next().unwrap(), &mut state)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn test_parse() {
        assert!(parse("let a = 1").is_err());
        assert!(parse("i | extend a = 1 i | extend b = 2").is_err());
    }
}
