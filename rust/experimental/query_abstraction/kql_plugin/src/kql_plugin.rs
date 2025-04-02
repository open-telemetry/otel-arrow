use intermediate_language::{grammar_objects::Query, query_processor::QueryProcessor};

pub struct KqlPlugin;

impl QueryProcessor for KqlPlugin {
    fn process_query(_input: &str) -> Query {
        todo!()
    }
}
