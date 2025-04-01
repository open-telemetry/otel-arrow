use crate::grammar_objects::*;

pub trait QueryProcessor {
    fn process_query(input: &str) -> Query;
}
