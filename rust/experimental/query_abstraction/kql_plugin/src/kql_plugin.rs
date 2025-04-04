use crate::kql_parser::{KqlParser, Rule};
use intermediate_language::{grammar_objects::Query, query_processor::QueryProcessor};
use pest::Parser;

pub struct KqlPlugin;

impl QueryProcessor for KqlPlugin {
    fn process_query(input: &str) -> Query {
        let _ = KqlParser::parse(Rule::query, input);
        todo!()
    }
}
