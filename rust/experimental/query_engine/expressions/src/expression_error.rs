use thiserror::Error;

use crate::QueryLocation;

#[derive(Error, Debug)]
pub enum ExpressionError {
    #[error("{1}")]
    TypeMismatch(QueryLocation, String),
}
