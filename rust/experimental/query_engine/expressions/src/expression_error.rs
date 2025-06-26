use thiserror::Error;

use crate::QueryLocation;

#[derive(Error, Debug)]
pub enum ExpressionError {
    #[error("{1}")]
    TypeMismatch(QueryLocation, String),
}

impl ExpressionError {
    pub fn get_query_location(&self) -> &QueryLocation {
        match self {
            ExpressionError::TypeMismatch(l, _) => l,
        }
    }
}
