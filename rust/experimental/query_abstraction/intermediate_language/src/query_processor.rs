use crate::grammar_objects::*;
use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub enum QueryError {
    ParseError(String),
    ProcessingError(String),
}

impl fmt::Display for QueryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            QueryError::ParseError(msg) => write!(f, "KQL Parse Error: {}", msg),
            QueryError::ProcessingError(msg) => write!(f, "KQL Processing Error: {}", msg),
        }
    }
}

impl Error for QueryError {}

pub type QueryResult<T> = Result<T, QueryError>;

pub trait QueryProcessor {
    fn process_query(input: &str) -> Result<Query, QueryError>;
}
