// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

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
            QueryError::ParseError(msg) => write!(f, "Parse Error: {msg}"),
            QueryError::ProcessingError(msg) => write!(f, "Processing Error: {msg}"),
        }
    }
}

impl Error for QueryError {}

pub type QueryResult<T> = Result<T, QueryError>;

pub trait QueryProcessor {
    fn process_query(input: &str) -> Result<Query, QueryError>;
}
