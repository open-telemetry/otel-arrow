use intermediate_language::{grammar_objects::Query, query_processor::QueryProcessor};

pub struct OttlPlugin;

impl QueryProcessor for OttlPlugin {
    fn process_query(input: &str) -> Query {
        todo!()
    }
}
