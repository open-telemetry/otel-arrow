use crate::ottl_parser::{OttlParser, Rule};
use intermediate_language::{
    grammar_objects::Query,
    query_processor::{QueryError, QueryProcessor},
};
use pest::Parser;

pub struct OttlPlugin;

impl QueryProcessor for OttlPlugin {
    fn process_query(input: &str) -> Result<Query, QueryError> {
        let _ = OttlParser::parse(Rule::query, input);
        todo!()
    }
}
