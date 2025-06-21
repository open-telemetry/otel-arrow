use data_engine_expressions::QueryLocation;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ParserError {
    #[error("{1}")]
    SyntaxNotSupported(QueryLocation, String),

    #[error("{1}")]
    SyntaxError(QueryLocation, String),

    #[error("{1}: {2}")]
    QueryLanguageDiagnostic(QueryLocation, &'static str, String),
}
