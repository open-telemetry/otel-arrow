use std::num::{ParseFloatError, ParseIntError};

use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("{0}")]
    RegistrationError(&'static str),

    #[error("Expression encountered an error: {1}")]
    ExpressionError(usize, Box<Error>),

    #[error("Operation was not evaluated")]
    NotEvaluated,

    #[error("{0}")]
    NotSupported(&'static str),

    #[error("{0}")]
    InvalidOperation(&'static str),

    #[error("Failed to parse double: {0}")]
    DoubleParseError(#[source] ParseFloatError),

    #[error("Failed to parse long: {0}")]
    LongParseError(#[source] ParseIntError),

    #[error("Failed to parse RegEx: {0}")]
    RegexParseError(#[source] regex::Error),

    #[error("Failed to parse DateTime: {0}")]
    DateTimeParseError(#[source] chrono::format::ParseError),

    #[error("Failed to parse path: {0}")]
    PathParseError(String),

    #[error("Failed to evaluate path: {0}")]
    PathEvaluationError(String),

    #[error("External summary was not found: {0}")]
    ExternalSummaryNotFound(String),
}

impl Error {
    pub(crate) fn new_expression_not_supported(
        expression_id: usize,
        message: &'static str,
    ) -> Error {
        Error::ExpressionError(expression_id, Error::NotSupported(message).into())
    }
}
