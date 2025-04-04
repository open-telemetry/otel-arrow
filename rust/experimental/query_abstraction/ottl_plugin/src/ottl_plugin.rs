use crate::ottl_parser::{OttlParser, Rule};
use intermediate_language::{grammar_objects::Query, query_processor::QueryProcessor};
use pest::Parser;

pub struct OttlPlugin;

impl QueryProcessor for OttlPlugin {
    fn process_query(input: &str) -> Query {
        let _ = OttlParser::parse(Rule::query, input);
        todo!()
    }
}
