use data_engine_expressions::QueryLocation;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ParserError {
    #[error("{1}")]
    SyntaxNotSupported(QueryLocation, String),

    #[error("{1}")]
    SyntaxError(QueryLocation, String),

    #[error("{diagnostic_id}: {message}")]
    QueryLanguageDiagnostic {
        location: QueryLocation,
        diagnostic_id: &'static str,
        message: String,
    },
}
